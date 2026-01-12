import { describe, expect, it } from "vitest";

describe("PistonProtection Frontend", () => {
  it("placeholder test - verifies test infrastructure works", () => {
    expect(true).toBe(true);
  });

  it("validates IP address format check logic", () => {
    // IPv4 validation
    const ipv4Regex = /^(\d{1,3}\.){3}\d{1,3}$/;
    expect(ipv4Regex.test("192.168.1.1")).toBe(true);
    expect(ipv4Regex.test("10.0.0.1")).toBe(true);
    expect(ipv4Regex.test("invalid")).toBe(false);
    expect(ipv4Regex.test("192.168.1")).toBe(false);
    expect(ipv4Regex.test("192.168.1.1.1")).toBe(false);
  });

  it("formats bytes correctly", () => {
    const formatBytes = (bytes: number): string => {
      if (bytes === 0) return "0 B";
      const k = 1024;
      const sizes = ["B", "KB", "MB", "GB", "TB"];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return `${Number.parseFloat((bytes / k ** i).toFixed(2))} ${sizes[i]}`;
    };

    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1 KB");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(1048576)).toBe("1 MB");
    expect(formatBytes(1073741824)).toBe("1 GB");
  });

  it("formats numbers correctly", () => {
    const formatNumber = (num: number): string => {
      if (num >= 1_000_000_000) {
        return `${(num / 1_000_000_000).toFixed(1)}B`;
      }
      if (num >= 1_000_000) {
        return `${(num / 1_000_000).toFixed(1)}M`;
      }
      if (num >= 1_000) {
        return `${(num / 1_000).toFixed(1)}K`;
      }
      return num.toString();
    };

    expect(formatNumber(500)).toBe("500");
    expect(formatNumber(1500)).toBe("1.5K");
    expect(formatNumber(1500000)).toBe("1.5M");
    expect(formatNumber(1500000000)).toBe("1.5B");
  });
});
