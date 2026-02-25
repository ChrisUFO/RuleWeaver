export type SortField = "name" | "createdAt" | "updatedAt" | "enabled";
export type SortDirection = "asc" | "desc";

export const SORT_OPTIONS = [
  { value: "name-asc", label: "Name (A-Z)" },
  { value: "name-desc", label: "Name (Z-A)" },
  { value: "createdAt-desc", label: "Newest First" },
  { value: "createdAt-asc", label: "Oldest First" },
  { value: "updatedAt-desc", label: "Recently Updated" },
  { value: "updatedAt-asc", label: "Least Recently Updated" },
  { value: "enabled-desc", label: "Enabled First" },
  { value: "enabled-asc", label: "Disabled First" },
];

export function parseSortValue(sortValue: string): {
  sortField: SortField;
  sortDirection: SortDirection;
} {
  const [field, dir] = sortValue.split("-") as [SortField, SortDirection];
  return { sortField: field, sortDirection: dir };
}
