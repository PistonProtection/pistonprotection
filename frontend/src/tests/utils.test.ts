import { describe, expect, it } from "vitest";
import { cn } from "../lib/utils";

describe("cn (className utility)", () => {
  it("merges class names correctly", () => {
    expect(cn("foo", "bar")).toBe("foo bar");
  });

  it("handles conditional classes", () => {
    expect(cn("base", true && "active")).toBe("base active");
    expect(cn("base", false && "active")).toBe("base");
  });

  it("handles undefined and null values", () => {
    expect(cn("base", undefined, null, "end")).toBe("base end");
  });

  it("merges tailwind classes correctly", () => {
    // twMerge should resolve conflicts
    expect(cn("px-2", "px-4")).toBe("px-4");
    expect(cn("text-red-500", "text-blue-500")).toBe("text-blue-500");
  });

  it("handles array inputs", () => {
    expect(cn(["foo", "bar"])).toBe("foo bar");
  });

  it("handles object inputs", () => {
    expect(cn({ foo: true, bar: false, baz: true })).toBe("foo baz");
  });

  it("handles mixed inputs", () => {
    expect(cn("base", ["array"], { object: true })).toBe("base array object");
  });
});
