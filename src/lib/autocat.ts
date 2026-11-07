export type AutoCatSummary = {
  matchedGames: number;
  skippedMissingMeta: number;
  createCount: number;
  addMemberOps: number;
  renameCount?: number;
  deleteCount?: number;
  removeMemberOps?: number;
};
