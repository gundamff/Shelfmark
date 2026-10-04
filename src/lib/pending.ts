export type CollectionChangeset = {
  create: unknown[];
  rename: unknown[];
  delete: unknown[];
  add_members: unknown[];
  remove_members: unknown[];
};

export function formatPendingSummary(pending: CollectionChangeset): string {
  const parts = [
    `${pending.create.length} create`,
    `${pending.rename.length} rename`,
    `${pending.delete.length} delete`,
    `${pending.add_members.length} add-ops`,
    `${pending.remove_members.length} remove-ops`,
  ];
  return parts.join(" · ");
}

export function isPendingEmpty(pending: CollectionChangeset): boolean {
  return (
    pending.create.length === 0 &&
    pending.rename.length === 0 &&
    pending.delete.length === 0 &&
    pending.add_members.length === 0 &&
    pending.remove_members.length === 0
  );
}
