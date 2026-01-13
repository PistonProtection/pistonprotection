import { describe, expect, it } from "vitest";
import { z } from "zod";

/**
 * Validation tests for filter rules and backend configurations.
 * These mirror the validation schemas used in the frontend forms.
 */

// Filter schema (from dashboard/filters.tsx)
const filterTypes = [
  "tcp",
  "udp",
  "http",
  "quic",
  "minecraft_java",
  "minecraft_bedrock",
] as const;

const filterSchema = z.object({
  name: z.string().min(1, "Name is required").max(100, "Name too long"),
  type: z.enum(filterTypes),
  action: z.enum(["allow", "block", "rate_limit", "challenge"]),
  priority: z
    .number()
    .min(0, "Priority must be >= 0")
    .max(1000, "Priority must be <= 1000"),
  enabled: z.boolean(),
  conditions: z.string().optional(),
  rateLimit: z.number().positive().optional(),
  rateLimitWindow: z.number().positive().optional(),
});

// Backend schema
const backendSchema = z.object({
  name: z.string().min(1, "Name is required").max(100, "Name too long"),
  slug: z
    .string()
    .min(1)
    .max(50)
    .regex(/^[a-z0-9-]+$/, "Slug must be lowercase alphanumeric with hyphens"),
  protocol: z.enum([
    "tcp",
    "udp",
    "http",
    "https",
    "minecraft_java",
    "minecraft_bedrock",
    "quic",
  ]),
  origins: z
    .array(
      z.object({
        host: z.string().min(1),
        port: z.number().min(1).max(65535),
      }),
    )
    .min(1, "At least one origin required"),
  enabled: z.boolean(),
});

// Rate limit validation
const rateLimitSchema = z
  .object({
    tokensPerSecond: z.number().positive("Rate must be positive"),
    bucketSize: z.number().positive("Bucket size must be positive"),
  })
  .refine((data) => data.bucketSize >= data.tokensPerSecond, {
    message: "Bucket size should be >= tokens per second",
  });

describe("Filter Schema Validation", () => {
  describe("valid filters", () => {
    it("accepts valid TCP filter", () => {
      const result = filterSchema.safeParse({
        name: "Block SYN Flood",
        type: "tcp",
        action: "block",
        priority: 100,
        enabled: true,
      });
      expect(result.success).toBe(true);
    });

    it("accepts filter with rate limit", () => {
      const result = filterSchema.safeParse({
        name: "Rate Limit API",
        type: "http",
        action: "rate_limit",
        priority: 50,
        enabled: true,
        rateLimit: 1000,
        rateLimitWindow: 60,
      });
      expect(result.success).toBe(true);
    });

    it("accepts filter with conditions", () => {
      const result = filterSchema.safeParse({
        name: "Allow Internal",
        type: "tcp",
        action: "allow",
        priority: 200,
        enabled: true,
        conditions: '{"source_ip": {"in": ["10.0.0.0/8"]}}',
      });
      expect(result.success).toBe(true);
    });

    it("accepts all filter types", () => {
      for (const type of filterTypes) {
        const result = filterSchema.safeParse({
          name: `Test ${type}`,
          type,
          action: "block",
          priority: 100,
          enabled: true,
        });
        expect(result.success).toBe(true);
      }
    });

    it("accepts all actions", () => {
      for (const action of ["allow", "block", "rate_limit", "challenge"]) {
        const result = filterSchema.safeParse({
          name: `Test ${action}`,
          type: "tcp",
          action,
          priority: 100,
          enabled: true,
        });
        expect(result.success).toBe(true);
      }
    });
  });

  describe("invalid filters", () => {
    it("rejects empty name", () => {
      const result = filterSchema.safeParse({
        name: "",
        type: "tcp",
        action: "block",
        priority: 100,
        enabled: true,
      });
      expect(result.success).toBe(false);
    });

    it("rejects invalid type", () => {
      const result = filterSchema.safeParse({
        name: "Test",
        type: "invalid_type",
        action: "block",
        priority: 100,
        enabled: true,
      });
      expect(result.success).toBe(false);
    });

    it("rejects negative priority", () => {
      const result = filterSchema.safeParse({
        name: "Test",
        type: "tcp",
        action: "block",
        priority: -1,
        enabled: true,
      });
      expect(result.success).toBe(false);
    });

    it("rejects priority > 1000", () => {
      const result = filterSchema.safeParse({
        name: "Test",
        type: "tcp",
        action: "block",
        priority: 1001,
        enabled: true,
      });
      expect(result.success).toBe(false);
    });

    it("rejects missing enabled field", () => {
      const result = filterSchema.safeParse({
        name: "Test",
        type: "tcp",
        action: "block",
        priority: 100,
      });
      expect(result.success).toBe(false);
    });
  });
});

describe("Backend Schema Validation", () => {
  describe("valid backends", () => {
    it("accepts valid HTTP backend", () => {
      const result = backendSchema.safeParse({
        name: "Production API",
        slug: "production-api",
        protocol: "https",
        origins: [{ host: "api.example.com", port: 443 }],
        enabled: true,
      });
      expect(result.success).toBe(true);
    });

    it("accepts backend with multiple origins", () => {
      const result = backendSchema.safeParse({
        name: "Load Balanced API",
        slug: "lb-api",
        protocol: "https",
        origins: [
          { host: "api-1.example.com", port: 443 },
          { host: "api-2.example.com", port: 443 },
        ],
        enabled: true,
      });
      expect(result.success).toBe(true);
    });

    it("accepts Minecraft backend", () => {
      const result = backendSchema.safeParse({
        name: "Game Server",
        slug: "game-server",
        protocol: "minecraft_java",
        origins: [{ host: "mc.example.com", port: 25565 }],
        enabled: true,
      });
      expect(result.success).toBe(true);
    });
  });

  describe("invalid backends", () => {
    it("rejects invalid slug format", () => {
      const result = backendSchema.safeParse({
        name: "Test",
        slug: "Invalid Slug!",
        protocol: "https",
        origins: [{ host: "example.com", port: 443 }],
        enabled: true,
      });
      expect(result.success).toBe(false);
    });

    it("rejects empty origins array", () => {
      const result = backendSchema.safeParse({
        name: "Test",
        slug: "test",
        protocol: "https",
        origins: [],
        enabled: true,
      });
      expect(result.success).toBe(false);
    });

    it("rejects invalid port", () => {
      const result = backendSchema.safeParse({
        name: "Test",
        slug: "test",
        protocol: "https",
        origins: [{ host: "example.com", port: 70000 }],
        enabled: true,
      });
      expect(result.success).toBe(false);
    });

    it("rejects zero port", () => {
      const result = backendSchema.safeParse({
        name: "Test",
        slug: "test",
        protocol: "https",
        origins: [{ host: "example.com", port: 0 }],
        enabled: true,
      });
      expect(result.success).toBe(false);
    });
  });
});

describe("Rate Limit Schema Validation", () => {
  it("accepts valid rate limit config", () => {
    const result = rateLimitSchema.safeParse({
      tokensPerSecond: 1000,
      bucketSize: 2000,
    });
    expect(result.success).toBe(true);
  });

  it("accepts equal tokens and bucket size", () => {
    const result = rateLimitSchema.safeParse({
      tokensPerSecond: 1000,
      bucketSize: 1000,
    });
    expect(result.success).toBe(true);
  });

  it("rejects bucket smaller than rate", () => {
    const result = rateLimitSchema.safeParse({
      tokensPerSecond: 1000,
      bucketSize: 500,
    });
    expect(result.success).toBe(false);
  });

  it("rejects zero tokens per second", () => {
    const result = rateLimitSchema.safeParse({
      tokensPerSecond: 0,
      bucketSize: 1000,
    });
    expect(result.success).toBe(false);
  });

  it("rejects negative values", () => {
    const result = rateLimitSchema.safeParse({
      tokensPerSecond: -100,
      bucketSize: 1000,
    });
    expect(result.success).toBe(false);
  });
});
