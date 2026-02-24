export function toggleInArray<T>(arr: readonly T[], item: T, include: boolean): T[] {
  if (include) {
    if (arr.includes(item)) return [...arr];
    return [...arr, item];
  }
  return arr.filter((x) => x !== item);
}

export function togglePathInSet(prev: readonly string[], path: string, checked: boolean): string[] {
  if (checked) {
    if (prev.includes(path)) return [...prev];
    return [...prev, path];
  }
  return prev.filter((p) => p !== path);
}

export function toggleInSet<T>(set: Set<T>, item: T, include: boolean): Set<T> {
  const next = new Set(set);
  if (include) {
    next.add(item);
  } else {
    next.delete(item);
  }
  return next;
}

export function getAdapterBadge(
  adapterId: string,
  adapters: readonly { id: string; name: string }[]
): string {
  const adapter = adapters.find((a) => a.id === adapterId);
  return adapter?.name ?? adapterId;
}

export function sortItems<T>(
  items: readonly T[],
  field: keyof T,
  direction: "asc" | "desc",
  compareFn?: (a: T, b: T, field: keyof T) => number
): T[] {
  const sorted = [...items];
  sorted.sort((a, b) => {
    let comparison: number;
    if (compareFn) {
      comparison = compareFn(a, b, field);
    } else {
      const aVal = a[field];
      const bVal = b[field];
      if (typeof aVal === "string" && typeof bVal === "string") {
        comparison = aVal.localeCompare(bVal);
      } else if (typeof aVal === "number" && typeof bVal === "number") {
        comparison = aVal - bVal;
      } else if (typeof aVal === "boolean" && typeof bVal === "boolean") {
        comparison = (aVal ? 1 : 0) - (bVal ? 1 : 0);
      } else {
        comparison = 0;
      }
    }
    return direction === "asc" ? comparison : -comparison;
  });
  return sorted;
}

export function filterByQuery<T>(
  items: readonly T[],
  query: string,
  fields: readonly (keyof T)[]
): T[] {
  const q = query.toLowerCase().trim();
  if (!q) return [...items];
  return items.filter((item) =>
    fields.some((field) => {
      const val = item[field];
      if (typeof val === "string") {
        return val.toLowerCase().includes(q);
      }
      return false;
    })
  );
}
