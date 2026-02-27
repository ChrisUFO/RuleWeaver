import { describe, it, expect } from "vitest";
import { cn, generateDuplicateName } from "@/lib/utils";

describe("cn utility", () => {
  it("should merge class names", () => {
    expect(cn("foo", "bar")).toBe("foo bar");
  });

  it("should handle conditional classes", () => {
    const condition = false;
    expect(cn("foo", condition && "bar", "baz")).toBe("foo baz");
    const trueCondition = true;
    expect(cn("foo", trueCondition && "bar", "baz")).toBe("foo bar baz");
  });

  it("should merge tailwind classes correctly", () => {
    expect(cn("p-4", "p-2")).toBe("p-2");
  });
});

describe("generateDuplicateName utility", () => {
  it("should append (Copy) to a name", () => {
    expect(generateDuplicateName("Rule", [])).toBe("Rule (Copy)");
  });

  it("should increment counter for already copied names", () => {
    expect(generateDuplicateName("Rule (Copy)", [])).toBe("Rule (Copy) 2");
    expect(generateDuplicateName("Rule (Copy) 2", [])).toBe("Rule (Copy) 3");
  });

  it("should handle names that already exist", () => {
    const existing = ["Rule (Copy)", "Rule (Copy) 2"];
    // Since "Rule" -> "Rule (Copy)" which exists, it should try "Rule (Copy) 2" which also exists, then "Rule (Copy) 3"
    expect(generateDuplicateName("Rule", existing)).toBe("Rule (Copy) 3");
  });

  it("should truncate very long names to stay under 100 characters", () => {
    const longName = "A".repeat(110);
    const result = generateDuplicateName(longName, []);
    expect(result.length).toBeLessThanOrEqual(100);
    expect(result).toContain("(Copy)");
  });
});
