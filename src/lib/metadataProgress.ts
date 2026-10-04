export function coveragePercent(have: number, total: number): number {
  if (total <= 0) return 0;
  return Math.min(100, Math.round((have / total) * 100));
}
