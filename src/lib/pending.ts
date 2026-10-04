export type CollectionChangeset = {
  create: unknown[];
  rename: unknown[];
  delete: unknown[];
  add_members: unknown[];
  remove_members: unknown[];
};

export function formatPendingSummary(
  pending: CollectionChangeset,
  translate?: (
    key: "pendingSummary",
    params: Record<string, number>,
  ) => string,
): string {
  const params = {
    create: pending.create.length,
    rename: pending.rename.length,
    delete: pending.delete.length,
    add: pending.add_members.length,
    remove: pending.remove_members.length,
  };
  if (translate) {
    return translate("pendingSummary", params);
  }
  return `${params.create} create · ${params.rename} rename · ${params.delete} delete · ${params.add} add-ops · ${params.remove} remove-ops`;
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
