import { describe, expect, it } from "vitest";

/**
 * Formatting utilities used throughout the frontend.
 * These are extracted and tested independently of components.
 */

// Number formatting (used in filters, admin, analytics pages)
function formatNumber(num: number): string {
  if (num >= 1_000_000_000) return `${(num / 1_000_000_000).toFixed(1)}B`;
  if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(1)}M`;
  if (num >= 1_000) return `${(num / 1_000).toFixed(1)}K`;
  return num.toString();
}

// Byte formatting (used in analytics, backends pages)
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${Number.parseFloat((bytes / k ** i).toFixed(2))} ${sizes[i]}`;
}

// Duration formatting (used in metrics, analytics)
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  if (ms < 3600000)
    return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
  return `${Math.floor(ms / 3600000)}h ${Math.floor((ms % 3600000) / 60000)}m`;
}

// Percentage formatting
function formatPercentage(value: number, decimals = 1): string {
  return `${value.toFixed(decimals)}%`;
}

// IP address validation (used in IP lookup, filters)
function isValidIPv4(ip: string): boolean {
  const parts = ip.split(".");
  if (parts.length !== 4) return false;
  return parts.every((part) => {
    const num = Number.parseInt(part, 10);
    return (
      !Number.isNaN(num) && num >= 0 && num <= 255 && part === num.toString()
    );
  });
}

function isValidIPv6(ip: string): boolean {
  // Simplified IPv6 validation
  const ipv6Regex = /^([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$/;
  const ipv6CompressedRegex =
    /^(([0-9a-fA-F]{1,4}:)*)?::?(([0-9a-fA-F]{1,4}:)*[0-9a-fA-F]{1,4})?$/;
  return ipv6Regex.test(ip) || ipv6CompressedRegex.test(ip);
}

// CIDR notation validation
function isValidCIDR(cidr: string): boolean {
  const parts = cidr.split("/");
  if (parts.length !== 2) return false;
  const ip = parts[0];
  const prefix = Number.parseInt(parts[1], 10);

  if (isValidIPv4(ip)) {
    return prefix >= 0 && prefix <= 32;
  }
  if (isValidIPv6(ip)) {
    return prefix >= 0 && prefix <= 128;
  }
  return false;
}

describe("formatNumber", () => {
  it("formats small numbers as-is", () => {
    expect(formatNumber(0)).toBe("0");
    expect(formatNumber(1)).toBe("1");
    expect(formatNumber(500)).toBe("500");
    expect(formatNumber(999)).toBe("999");
  });

  it("formats thousands with K suffix", () => {
    expect(formatNumber(1000)).toBe("1.0K");
    expect(formatNumber(1500)).toBe("1.5K");
    expect(formatNumber(10000)).toBe("10.0K");
    expect(formatNumber(999999)).toBe("1000.0K");
  });

  it("formats millions with M suffix", () => {
    expect(formatNumber(1000000)).toBe("1.0M");
    expect(formatNumber(1500000)).toBe("1.5M");
    expect(formatNumber(10000000)).toBe("10.0M");
  });

  it("formats billions with B suffix", () => {
    expect(formatNumber(1000000000)).toBe("1.0B");
    expect(formatNumber(1500000000)).toBe("1.5B");
  });
});

describe("formatBytes", () => {
  it("formats zero bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
  });

  it("formats bytes", () => {
    expect(formatBytes(500)).toBe("500 B");
  });

  it("formats kilobytes", () => {
    expect(formatBytes(1024)).toBe("1 KB");
    expect(formatBytes(1536)).toBe("1.5 KB");
  });

  it("formats megabytes", () => {
    expect(formatBytes(1048576)).toBe("1 MB");
    expect(formatBytes(1572864)).toBe("1.5 MB");
  });

  it("formats gigabytes", () => {
    expect(formatBytes(1073741824)).toBe("1 GB");
  });

  it("formats terabytes", () => {
    expect(formatBytes(1099511627776)).toBe("1 TB");
  });
});

describe("formatDuration", () => {
  it("formats milliseconds", () => {
    expect(formatDuration(50)).toBe("50ms");
    expect(formatDuration(999)).toBe("999ms");
  });

  it("formats seconds", () => {
    expect(formatDuration(1000)).toBe("1.0s");
    expect(formatDuration(5500)).toBe("5.5s");
  });

  it("formats minutes and seconds", () => {
    expect(formatDuration(60000)).toBe("1m 0s");
    expect(formatDuration(90000)).toBe("1m 30s");
    expect(formatDuration(125000)).toBe("2m 5s");
  });

  it("formats hours and minutes", () => {
    expect(formatDuration(3600000)).toBe("1h 0m");
    expect(formatDuration(3660000)).toBe("1h 1m");
    expect(formatDuration(7200000)).toBe("2h 0m");
  });
});

describe("formatPercentage", () => {
  it("formats percentages with default decimals", () => {
    expect(formatPercentage(50)).toBe("50.0%");
    expect(formatPercentage(99.5)).toBe("99.5%");
  });

  it("formats percentages with custom decimals", () => {
    expect(formatPercentage(50, 0)).toBe("50%");
    expect(formatPercentage(99.123, 2)).toBe("99.12%");
  });
});

describe("IP validation", () => {
  describe("isValidIPv4", () => {
    it("validates correct IPv4 addresses", () => {
      expect(isValidIPv4("192.168.1.1")).toBe(true);
      expect(isValidIPv4("10.0.0.1")).toBe(true);
      expect(isValidIPv4("0.0.0.0")).toBe(true);
      expect(isValidIPv4("255.255.255.255")).toBe(true);
    });

    it("rejects invalid IPv4 addresses", () => {
      expect(isValidIPv4("192.168.1")).toBe(false);
      expect(isValidIPv4("192.168.1.1.1")).toBe(false);
      expect(isValidIPv4("256.1.1.1")).toBe(false);
      expect(isValidIPv4("192.168.1.01")).toBe(false); // Leading zero
      expect(isValidIPv4("invalid")).toBe(false);
      expect(isValidIPv4("")).toBe(false);
    });
  });

  describe("isValidIPv6", () => {
    it("validates correct IPv6 addresses", () => {
      expect(isValidIPv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334")).toBe(true);
      expect(isValidIPv6("::1")).toBe(true);
      expect(isValidIPv6("::")).toBe(true);
    });

    it("rejects invalid IPv6 addresses", () => {
      expect(isValidIPv6("192.168.1.1")).toBe(false);
      expect(isValidIPv6("invalid")).toBe(false);
    });
  });
});

describe("CIDR validation", () => {
  it("validates correct CIDR notation for IPv4", () => {
    expect(isValidCIDR("192.168.1.0/24")).toBe(true);
    expect(isValidCIDR("10.0.0.0/8")).toBe(true);
    expect(isValidCIDR("0.0.0.0/0")).toBe(true);
    expect(isValidCIDR("192.168.1.1/32")).toBe(true);
  });

  it("rejects invalid CIDR for IPv4", () => {
    expect(isValidCIDR("192.168.1.0/33")).toBe(false);
    expect(isValidCIDR("192.168.1.0/-1")).toBe(false);
    expect(isValidCIDR("192.168.1.0")).toBe(false);
  });

  it("validates correct CIDR notation for IPv6", () => {
    expect(isValidCIDR("::/0")).toBe(true);
    expect(isValidCIDR("::1/128")).toBe(true);
  });
});
