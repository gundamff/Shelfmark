use crate::types::AppId;

/// Result of attempting to fetch/resolve metadata for one app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchStep {
    /// Resolved from cache or local data without a remote store request.
    Cached,
    /// Fetched successfully from the remote store.
    Fetched,
    /// Metadata was already sufficient; no work needed.
    AlreadyPresent,
    /// This app could not be fetched (non-fatal for the loop).
    AppFailed,
}

/// Progress snapshot for a metadata fetch loop.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FetchProgress {
    pub total: usize,
    pub done: usize,
    pub current_app: Option<AppId>,
    pub cached: usize,
    pub fetched: usize,
    pub failed: usize,
    /// Real error text from the last batch/app failure that should be surfaced.
    pub last_error: Option<String>,
    pub cancelled: bool,
    pub running: bool,
}

impl FetchProgress {
    pub fn remaining(&self) -> usize {
        self.total.saturating_sub(self.done)
    }
}

/// Outcome of one loop run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FetchLoopReport {
    pub cached: usize,
    pub fetched: usize,
    pub failed: usize,
    pub already_present: usize,
    pub cancelled: bool,
    /// If set, a batch failed with a real error that was surfaced.
    pub batch_error: Option<String>,
}

/// Drives metadata fetching in batches until the library is covered or cancelled.
///
/// - `app_ids`: full set to consider, in order.
/// - `batch_cap`: safety limit per batch (use 40).
/// - `should_stop`: returns true when cancellation is requested.
/// - `on_progress`: called with progress snapshots (including `current_app`).
/// - `fetch_one`: resolves one app; `Err` surfaces a real error for that batch
///   and stops the loop (so the UI can show it instead of stalling silently).
///   Return `Ok(FetchStep::AppFailed)` for per-app failures that should not
///   abort the whole run.
pub fn run_fetch_loop<F, P, S>(
    app_ids: &[AppId],
    batch_cap: usize,
    mut should_stop: S,
    mut on_progress: P,
    mut fetch_one: F,
) -> FetchLoopReport
where
    F: FnMut(AppId) -> Result<FetchStep, String>,
    P: FnMut(&FetchProgress),
    S: FnMut() -> bool,
{
    let batch_cap = batch_cap.max(1);
    let mut progress = FetchProgress {
        total: app_ids.len(),
        running: true,
        ..FetchProgress::default()
    };
    let mut report = FetchLoopReport::default();
    on_progress(&progress);

    let mut index = 0usize;
    while index < app_ids.len() {
        if should_stop() {
            progress.cancelled = true;
            report.cancelled = true;
            break;
        }

        let end = (index + batch_cap).min(app_ids.len());
        for &app_id in &app_ids[index..end] {
            if should_stop() {
                progress.cancelled = true;
                report.cancelled = true;
                break;
            }
            progress.current_app = Some(app_id);
            on_progress(&progress);

            match fetch_one(app_id) {
                Ok(FetchStep::Cached) => {
                    progress.cached += 1;
                    report.cached += 1;
                }
                Ok(FetchStep::Fetched) => {
                    progress.fetched += 1;
                    report.fetched += 1;
                }
                Ok(FetchStep::AlreadyPresent) => {
                    report.already_present += 1;
                }
                Ok(FetchStep::AppFailed) => {
                    progress.failed += 1;
                    report.failed += 1;
                }
                Err(err) => {
                    progress.failed += 1;
                    report.failed += 1;
                    progress.last_error = Some(err.clone());
                    report.batch_error = Some(err);
                    progress.current_app = None;
                    progress.done += 1;
                    progress.running = false;
                    on_progress(&progress);
                    return report;
                }
            }
            progress.done += 1;
            progress.last_error = None;
            on_progress(&progress);
        }

        if progress.cancelled {
            break;
        }
        index = end;
    }

    progress.current_app = None;
    progress.running = false;
    on_progress(&progress);
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn ids(n: u32) -> Vec<AppId> {
        (1..=n).collect()
    }

    #[test]
    fn loops_in_batches_until_complete() {
        let mut seen_batches: Vec<Vec<AppId>> = Vec::new();
        let mut current_batch: Vec<AppId> = Vec::new();
        let mut in_batch = 0usize;
        let report = run_fetch_loop(
            &ids(5),
            2,
            || false,
            |_| {},
            |id| {
                current_batch.push(id);
                in_batch += 1;
                if in_batch == 2 || id == 5 {
                    seen_batches.push(std::mem::take(&mut current_batch));
                    in_batch = 0;
                }
                Ok(FetchStep::Fetched)
            },
        );
        assert_eq!(report.fetched, 5);
        assert!(!report.cancelled);
        assert_eq!(seen_batches, vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn cancel_stops_promptly_and_reports_progress() {
        let stop_after = Cell::new(2usize);
        let mut snapshots = Vec::new();
        let report = run_fetch_loop(
            &ids(10),
            4,
            || stop_after.get() == 0,
            |p| snapshots.push((p.done, p.current_app, p.cancelled)),
            |_| {
                stop_after.set(stop_after.get().saturating_sub(1));
                Ok(FetchStep::Cached)
            },
        );
        assert!(report.cancelled);
        assert!(report.cached < 10);
        assert!(snapshots.iter().any(|s| s.2));
        let last = snapshots.last().unwrap();
        assert!(!last.1.is_some() || last.2);
    }

    #[test]
    fn surfaces_batch_error_text_and_stops() {
        let report = run_fetch_loop(
            &ids(5),
            3,
            || false,
            |_| {},
            |id| {
                if id == 2 {
                    Err("proxy required: connect timed out".to_string())
                } else {
                    Ok(FetchStep::Fetched)
                }
            },
        );
        assert_eq!(
            report.batch_error.as_deref(),
            Some("proxy required: connect timed out")
        );
        assert_eq!(report.fetched, 1);
        assert_eq!(report.failed, 1);
        assert_eq!(report.cached + report.fetched + report.failed + report.already_present, 2);
    }

    #[test]
    fn progress_reports_done_total_and_current_app() {
        let mut currents = Vec::new();
        let mut totals = Vec::new();
        run_fetch_loop(
            &ids(3),
            40,
            || false,
            |p| {
                totals.push(p.total);
                if let Some(c) = p.current_app {
                    if currents.last().copied() != Some(c) {
                        currents.push(c);
                    }
                }
            },
            |_| Ok(FetchStep::AlreadyPresent),
        );
        assert!(totals.iter().all(|&t| t == 3));
        assert_eq!(currents, vec![1, 2, 3]);
    }

    #[test]
    fn app_failed_does_not_abort_loop() {
        let report = run_fetch_loop(
            &ids(3),
            40,
            || false,
            |_| {},
            |id| {
                if id == 2 {
                    Ok(FetchStep::AppFailed)
                } else {
                    Ok(FetchStep::Fetched)
                }
            },
        );
        assert_eq!(report.fetched, 2);
        assert_eq!(report.failed, 1);
        assert!(report.batch_error.is_none());
    }
}
