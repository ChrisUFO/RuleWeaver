import { describe, it, expect } from "vitest";
import {
  toggleInArray,
  togglePathInSet,
  toggleInSet,
  sortItems,
  filterByQuery,
} from "@/lib/collection-utils";

describe("toggleInArray", () => {
  it("adds item when not present", () => {
    expect(toggleInArray(["a", "b"], "c", true)).toEqual(["a", "b", "c"]);
  });

  it("removes item when present and not including", () => {
    expect(toggleInArray(["a", "b", "c"], "b", false)).toEqual(["a", "c"]);
  });

  it("returns same array if item already present and including", () => {
    const result = toggleInArray(["a", "b"], "b", true);
    expect(result).toEqual(["a", "b"]);
    expect(result).not.toBe(["a", "b"]);
  });

  it("returns filtered array if item not present and not including", () => {
    expect(toggleInArray(["a", "b"], "c", false)).toEqual(["a", "b"]);
  });

  it("handles empty array", () => {
    expect(toggleInArray([], "a", true)).toEqual(["a"]);
    expect(toggleInArray([], "a", false)).toEqual([]);
  });
});

describe("togglePathInSet", () => {
  it("adds path when checked", () => {
    expect(togglePathInSet(["/a", "/b"], "/c", true)).toEqual(["/a", "/b", "/c"]);
  });

  it("removes path when unchecked", () => {
    expect(togglePathInSet(["/a", "/b", "/c"], "/b", false)).toEqual(["/a", "/c"]);
  });

  it("does not duplicate paths", () => {
    const result = togglePathInSet(["/a", "/b"], "/b", true);
    expect(result).toEqual(["/a", "/b"]);
  });
});

describe("toggleInSet", () => {
  it("adds item to set", () => {
    const set = toggleInSet(new Set(["a"]), "b", true);
    expect(set.has("a")).toBe(true);
    expect(set.has("b")).toBe(true);
  });

  it("removes item from set", () => {
    const set = toggleInSet(new Set(["a", "b"]), "b", false);
    expect(set.has("a")).toBe(true);
    expect(set.has("b")).toBe(false);
  });

  it("returns new set instance", () => {
    const original = new Set(["a"]);
    const result = toggleInSet(original, "b", true);
    expect(result).not.toBe(original);
  });
});

describe("sortItems", () => {
  const items = [
    { name: "Charlie", count: 3 },
    { name: "Alpha", count: 1 },
    { name: "Bravo", count: 2 },
  ];

  it("sorts by string field ascending", () => {
    const sorted = sortItems(items, "name", "asc");
    expect(sorted.map((i) => i.name)).toEqual(["Alpha", "Bravo", "Charlie"]);
  });

  it("sorts by string field descending", () => {
    const sorted = sortItems(items, "name", "desc");
    expect(sorted.map((i) => i.name)).toEqual(["Charlie", "Bravo", "Alpha"]);
  });

  it("sorts by number field ascending", () => {
    const sorted = sortItems(items, "count", "asc");
    expect(sorted.map((i) => i.count)).toEqual([1, 2, 3]);
  });

  it("sorts by number field descending", () => {
    const sorted = sortItems(items, "count", "desc");
    expect(sorted.map((i) => i.count)).toEqual([3, 2, 1]);
  });

  it("does not mutate original array", () => {
    sortItems(items, "name", "asc");
    expect(items[0].name).toBe("Charlie");
  });
});

describe("filterByQuery", () => {
  const items = [
    { name: "Apple", description: "Red fruit" },
    { name: "Banana", description: "Yellow fruit" },
    { name: "Cherry", description: "Red berry" },
  ];

  it("filters by single field", () => {
    const filtered = filterByQuery(items, "apple", ["name"]);
    expect(filtered).toHaveLength(1);
    expect(filtered[0].name).toBe("Apple");
  });

  it("filters by multiple fields", () => {
    const filtered = filterByQuery(items, "red", ["name", "description"]);
    expect(filtered).toHaveLength(2);
  });

  it("is case-insensitive", () => {
    const filtered = filterByQuery(items, "APPLE", ["name"]);
    expect(filtered).toHaveLength(1);
  });

  it("returns all items for empty query", () => {
    const filtered = filterByQuery(items, "", ["name"]);
    expect(filtered).toHaveLength(3);
  });

  it("returns all items for whitespace-only query", () => {
    const filtered = filterByQuery(items, "   ", ["name"]);
    expect(filtered).toHaveLength(3);
  });
});
