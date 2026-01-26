import { describe, it, expect } from "vitest";
import {
  apiConfigSchema,
  authConfigSchema,
  appConfigSchema,
  validateConfig,
  validatePartialConfig,
} from "../src/schema.js";

describe("apiConfigSchema", () => {
  it("should validate valid API config", () => {
    const config = {
      baseUrl: "https://api.example.com",
      timeout: 5000,
      retryCount: 3,
      retryDelay: 1000,
    };
    const result = apiConfigSchema.safeParse(config);
    expect(result.success).toBe(true);
  });

  it("should apply defaults for optional fields", () => {
    const config = {
      baseUrl: "https://api.example.com",
    };
    const result = apiConfigSchema.parse(config);
    expect(result.timeout).toBe(30000);
    expect(result.retryCount).toBe(3);
    expect(result.retryDelay).toBe(1000);
  });

  it("should reject invalid URL", () => {
    const config = {
      baseUrl: "not-a-url",
    };
    const result = apiConfigSchema.safeParse(config);
    expect(result.success).toBe(false);
  });

  it("should reject negative timeout", () => {
    const config = {
      baseUrl: "https://api.example.com",
      timeout: -1000,
    };
    const result = apiConfigSchema.safeParse(config);
    expect(result.success).toBe(false);
  });
});

describe("authConfigSchema", () => {
  it("should validate valid auth config", () => {
    const config = {
      enabled: true,
      provider: "jwt",
      tokenRefreshThreshold: 300,
      storage: "localStorage",
    };
    const result = authConfigSchema.safeParse(config);
    expect(result.success).toBe(true);
  });

  it("should apply defaults", () => {
    const config = {};
    const result = authConfigSchema.parse(config);
    expect(result.enabled).toBe(true);
    expect(result.provider).toBe("jwt");
    expect(result.storage).toBe("localStorage");
  });

  it("should reject invalid provider", () => {
    const config = {
      provider: "invalid",
    };
    const result = authConfigSchema.safeParse(config);
    expect(result.success).toBe(false);
  });
});

describe("appConfigSchema", () => {
  it("should validate complete app config", () => {
    const config = {
      env: "prod",
      appName: "my-app",
      version: "1.0.0",
      api: {
        baseUrl: "https://api.example.com",
      },
      auth: {
        enabled: true,
        provider: "oauth2",
      },
      features: {
        darkMode: true,
        experimentalFeature: false,
      },
    };
    const result = appConfigSchema.safeParse(config);
    expect(result.success).toBe(true);
  });

  it("should apply defaults for minimal config", () => {
    const config = {};
    const result = appConfigSchema.parse(config);
    expect(result.env).toBe("dev");
    expect(result.appName).toBe("k1s0-app");
  });
});

describe("validateConfig", () => {
  it("should return success with data for valid config", () => {
    const config = { baseUrl: "https://api.example.com" };
    const result = validateConfig(apiConfigSchema, config);
    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.baseUrl).toBe("https://api.example.com");
    }
  });

  it("should return errors for invalid config", () => {
    const config = { baseUrl: "invalid" };
    const result = validateConfig(apiConfigSchema, config);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.errors.issues.length).toBeGreaterThan(0);
    }
  });
});

describe("validatePartialConfig", () => {
  it("should validate partial config", () => {
    const config = { timeout: 5000 };
    const result = validatePartialConfig(apiConfigSchema, config);
    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.timeout).toBe(5000);
      expect(result.data.baseUrl).toBeUndefined();
    }
  });
});
