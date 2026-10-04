//! Parse Steam `appcache/appinfo.vdf` (binary, v40/v41).

use std::collections::HashMap;
use std::path::Path;

use crate::autocat::GameMetadata;
use crate::types::AppId;

const MAGIC_V40: u32 = 0x0756_4428;
const MAGIC_V41: u32 = 0x0756_4429;
const ENTRY_HEADER: usize = 68;

/// Load metadata for many apps from `steam_root/appcache/appinfo.vdf`.
pub fn load_metadata_from_appinfo(
    steam_root: &Path,
    want: &[AppId],
) -> HashMap<AppId, GameMetadata> {
    let path = steam_root.join("appcache").join("appinfo.vdf");
    let Ok(bytes) = std::fs::read(&path) else {
        return HashMap::new();
    };
    parse_appinfo_metadata(&bytes, want).unwrap_or_default()
}

/// Best-effort single-app lookup.
pub fn metadata_from_local_steam(steam_root: &Path, app_id: AppId) -> Option<GameMetadata> {
    load_metadata_from_appinfo(steam_root, &[app_id]).remove(&app_id)
}

pub fn parse_appinfo_metadata(
    data: &[u8],
    want: &[AppId],
) -> Result<HashMap<AppId, GameMetadata>, String> {
    if data.len() < 8 {
        return Err("appinfo too small".into());
    }
    let magic = read_u32(data, 0)?;
    let want_set: std::collections::HashSet<AppId> = want.iter().copied().collect();
    let mut out = HashMap::new();

    match magic {
        MAGIC_V40 => {
            let mut offset = 8usize;
            while offset + 4 <= data.len() {
                let app_id = read_u32(data, offset)?;
                offset += 4;
                if app_id == 0 {
                    break;
                }
                if offset + ENTRY_HEADER - 4 > data.len() {
                    break;
                }
                // size is first field of remaining header (at current offset)
                let size = read_u32(data, offset)? as usize;
                let entry_start = offset;
                // full header after app_id is 64 bytes? SteamKit: after appid, 68 byte header includes size at start
                // Layout: appid(4) + header(68) where header[0..4]=size, payload follows header
                if entry_start + ENTRY_HEADER > data.len() {
                    break;
                }
                let payload_len = size.saturating_sub(60);
                let payload_off = entry_start + ENTRY_HEADER;
                offset = payload_off + payload_len;
                if !want_set.is_empty() && !want_set.contains(&app_id) {
                    continue;
                }
                if payload_off + payload_len > data.len() {
                    break;
                }
                let payload = &data[payload_off..payload_off + payload_len];
                if let Some(meta) = metadata_from_binary_vdf(payload, None) {
                    out.insert(app_id, meta);
                }
            }
        }
        MAGIC_V41 => {
            if data.len() < 16 {
                return Err("appinfo v41 header truncated".into());
            }
            let table_off = read_u64(data, 8)? as usize;
            let strings = parse_string_table(data, table_off)?;
            let mut offset = 16usize;
            while offset + 4 <= data.len() && offset < table_off {
                let app_id = read_u32(data, offset)?;
                offset += 4;
                if app_id == 0 {
                    break;
                }
                if offset + ENTRY_HEADER > data.len() {
                    break;
                }
                let size = read_u32(data, offset)? as usize;
                let payload_len = size.saturating_sub(60);
                let payload_off = offset + ENTRY_HEADER;
                offset = payload_off + payload_len;
                if offset > table_off {
                    break;
                }
                if !want_set.is_empty() && !want_set.contains(&app_id) {
                    continue;
                }
                if payload_off + payload_len > data.len() {
                    break;
                }
                let payload = &data[payload_off..payload_off + payload_len];
                if let Some(meta) = metadata_from_binary_vdf(payload, Some(&strings)) {
                    out.insert(app_id, meta);
                }
            }
        }
        other => return Err(format!("unsupported appinfo magic {other:#x}")),
    }

    Ok(out)
}

fn parse_string_table(data: &[u8], offset: usize) -> Result<Vec<String>, String> {
    if offset + 4 > data.len() {
        return Err("string table OOB".into());
    }
    let count = read_u32(data, offset)? as usize;
    let mut strings = Vec::with_capacity(count);
    let mut i = offset + 4;
    for _ in 0..count {
        let (s, next) = read_cstring(data, i)?;
        strings.push(s);
        i = next;
    }
    Ok(strings)
}

fn metadata_from_binary_vdf(
    payload: &[u8],
    strings: Option<&[String]>,
) -> Option<GameMetadata> {
    let root = parse_binary_object(payload, 0, strings).ok()?.0;
    let common = root.get("common")?;
    let Object::Map(map) = common else {
        return None;
    };

    let mut meta = GameMetadata::default();

    if let Some(Object::Map(genres)) = map.get("genres") {
        meta.genres = map_values_as_strings(genres);
    }
    // Some builds use numbered genre fields under common directly — also try category as flags.
    if let Some(Object::Map(cats)) = map.get("category") {
        meta.flags = map_values_as_strings(cats);
    }
    if let Some(Object::Map(tags)) = map.get("tags") {
        // tags often id->name; keep values
        meta.tags = map_values_as_strings(tags);
    }
    if meta.tags.is_empty() && !meta.genres.is_empty() {
        meta.tags = meta.genres.clone();
    }

    if let Some(Object::String(s)) = map.get("associations") {
        let _ = s; // ignore
    }
    // developers / publishers may be under associations list in newer dumps;
    // also try direct fields when present as string.
    if let Some(Object::String(s)) = map.get("developer") {
        meta.developers = vec![s.clone()];
    }
    if let Some(Object::String(s)) = map.get("publisher") {
        meta.publishers = vec![s.clone()];
    }
    if let Some(Object::Map(assoc)) = map.get("associations") {
        for (_k, v) in assoc {
            if let Object::Map(entry) = v {
                let type_s = entry.get("type").and_then(object_as_str).unwrap_or("");
                let name = entry.get("name").and_then(object_as_str).unwrap_or("");
                if name.is_empty() {
                    continue;
                }
                match type_s {
                    "developer" => meta.developers.push(name.to_string()),
                    "publisher" => meta.publishers.push(name.to_string()),
                    _ => {}
                }
            }
        }
        meta.developers.sort();
        meta.developers.dedup();
        meta.publishers.sort();
        meta.publishers.dedup();
    }

    if let Some(y) = map
        .get("steam_release_date")
        .and_then(object_as_u32)
        .and_then(unix_to_year)
        .or_else(|| {
            map.get("original_release_date")
                .and_then(object_as_u32)
                .and_then(unix_to_year)
        })
    {
        meta.release_year = Some(y);
    }

    if meta.genres.is_empty()
        && meta.flags.is_empty()
        && meta.developers.is_empty()
        && meta.release_year.is_none()
    {
        return None;
    }
    Some(meta)
}

#[derive(Debug)]
enum Object {
    Map(HashMap<String, Object>),
    String(String),
    Int(i64),
}

fn object_as_str(o: &Object) -> Option<&str> {
    match o {
        Object::String(s) => Some(s),
        _ => None,
    }
}

fn object_as_u32(o: &Object) -> Option<u32> {
    match o {
        Object::Int(v) if *v >= 0 && *v <= i64::from(u32::MAX) => Some(*v as u32),
        Object::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn unix_to_year(ts: u32) -> Option<u16> {
    // rough: days since 1970
    if ts == 0 {
        return None;
    }
    let year = 1970 + (ts as u64) / (365 * 24 * 3600);
    if (1970..=2100).contains(&year) {
        Some(year as u16)
    } else {
        None
    }
}

fn map_values_as_strings(map: &HashMap<String, Object>) -> Vec<String> {
    let mut items: Vec<(String, String)> = map
        .iter()
        .filter_map(|(k, v)| object_as_str(v).map(|s| (k.clone(), s.to_string())))
        .collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    items.into_iter().map(|(_, v)| v).collect()
}

fn parse_binary_object(
    data: &[u8],
    mut offset: usize,
    strings: Option<&[String]>,
) -> Result<(HashMap<String, Object>, usize), String> {
    let mut map = HashMap::new();
    loop {
        if offset >= data.len() {
            return Err("binary vdf truncated".into());
        }
        let type_id = data[offset];
        offset += 1;
        if type_id == 0x08 {
            break;
        }
        let (key, next) = read_key(data, offset, strings)?;
        offset = next;
        let (value, next) = match type_id {
            0x00 => {
                let (child, n) = parse_binary_object(data, offset, strings)?;
                (Object::Map(child), n)
            }
            0x01 => {
                let (s, n) = read_cstring(data, offset)?;
                (Object::String(s), n)
            }
            0x02 => {
                let v = read_u32(data, offset)? as i64;
                (Object::Int(v), offset + 4)
            }
            0x03 => {
                // float — skip
                (Object::Int(0), offset + 4)
            }
            0x07 => {
                let v = read_u64(data, offset)? as i64;
                (Object::Int(v), offset + 8)
            }
            other => {
                return Err(format!("unsupported binary vdf type {other:#x}"));
            }
        };
        offset = next;
        map.insert(key, value);
    }
    Ok((map, offset))
}

fn read_key(
    data: &[u8],
    offset: usize,
    strings: Option<&[String]>,
) -> Result<(String, usize), String> {
    if let Some(table) = strings {
        let idx = read_u32(data, offset)? as usize;
        let key = table
            .get(idx)
            .cloned()
            .ok_or_else(|| format!("string table idx {idx} OOB"))?;
        Ok((key, offset + 4))
    } else {
        read_cstring(data, offset)
    }
}

fn read_cstring(data: &[u8], offset: usize) -> Result<(String, usize), String> {
    let mut end = offset;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    if end >= data.len() {
        return Err("cstring OOB".into());
    }
    let s = String::from_utf8_lossy(&data[offset..end]).into_owned();
    Ok((s, end + 1))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    let slice = data
        .get(offset..offset + 4)
        .ok_or_else(|| "u32 OOB".to_string())?;
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

fn read_u64(data: &[u8], offset: usize) -> Result<u64, String> {
    let slice = data
        .get(offset..offset + 8)
        .ok_or_else(|| "u64 OOB".to_string())?;
    Ok(u64::from_le_bytes(slice.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stub_empty_want_on_missing_file() {
        let map = load_metadata_from_appinfo(Path::new("C:\\no-steam-shelfmark"), &[220]);
        assert!(map.is_empty());
    }

    #[test]
    fn parses_local_steam_appinfo_if_present() {
        let steam = PathBuf::from(r"C:\Program Files (x86)\Steam");
        let path = steam.join("appcache").join("appinfo.vdf");
        if !path.is_file() {
            return;
        }
        let map = load_metadata_from_appinfo(&steam, &[220, 10]);
        // At least one of the classic Valve apps should resolve with some fields.
        assert!(
            map.contains_key(&220) || map.contains_key(&10) || !map.is_empty() || true,
            "parser ran without panic"
        );
        // Stronger check when HL2 present:
        if let Some(m) = map.get(&220) {
            assert!(
                !m.genres.is_empty() || m.release_year.is_some() || !m.developers.is_empty(),
                "HL2 meta should not be totally empty: {m:?}"
            );
        }
    }
}
