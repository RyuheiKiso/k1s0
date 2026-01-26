import { describe, it, expect } from "vitest";
import {
  deepMerge,
  mergeConfigs,
  mergeEnvironmentConfig,
  getNestedValue,
  setNestedValue,
} from "../src/merge.js";

describe("deepMerge", () => {
  it("should merge two flat objects", () => {
    const target = { a: 1, b: 2 };
    const source = { b: 3, c: 4 };
    const result = deepMerge(target, source);
    expect(result).toEqual({ a: 1, b: 3, c: 4 });
  });

  it("should recursively merge nested objects", () => {
    const target = {
      api: { baseUrl: "http://localhost", timeout: 5000 },
      auth: { enabled: true },
    };
    const source = {
      api: { baseUrl: "https://api.example.com" },
    };
    const result = deepMerge(target, source);
    expect(result).toEqual({
      api: { baseUrl: "https://api.example.com", timeout: 5000 },
      auth: { enabled: true },
    });
  });

  it("should replace arrays instead of merging", () => {
    const target = { items: [1, 2, 3] };
    const source = { items: [4, 5] };
    const result = deepMerge(target, source);
    expect(result.items).toEqual([4, 5]);
  });

  it("should not modify original objects", () => {
    const target = { a: 1, nested: { b: 2 } };
    const source = { nested: { c: 3 } };
    const result = deepMerge(target, source);
    expect(target.nested).toEqual({ b: 2 });
    expect(result.nested).toEqual({ b: 2, c: 3 });
  });
});

describe("mergeConfigs", () => {
  it("should merge multiple configs in order", () => {
    const config1 = { a: 1 };
    const config2 = { b: 2 };
    const config3 = { a: 3, c: 4 };
    const result = mergeConfigs(config1, config2, config3);
    expect(result).toEqual({ a: 3, b: 2, c: 4 });
  });

  it("should return empty object for no configs", () => {
    const result = mergeConfigs();
    expect(result).toEqual({});
  });
});

describe("mergeEnvironmentConfig", () => {
  it("should merge dev config with default", () => {
    const configs = {
      default: { api: { baseUrl: "http://localhost", timeout: 5000 } },
      dev: { api: { timeout: 10000 } },
      prod: { api: { baseUrl: "https://api.example.com" } },
    };
    const result = mergeEnvironmentConfig(configs, "dev");
    expect(result).toEqual({
      api: { baseUrl: "http://localhost", timeout: 10000 },
    });
  });

  it("should return default when env config is missing", () => {
    const configs = {
      default: { api: { baseUrl: "http://localhost" } },
    };
    const result = mergeEnvironmentConfig(configs, "stg");
    expect(result).toEqual({ api: { baseUrl: "http://localhost" } });
  });
});

describe("getNestedValue", () => {
  it("should get value at nested path", () => {
    const config = {
      api: { baseUrl: "http://localhost", auth: { token: "secret" } },
    };
    expect(getNestedValue(config, "api.baseUrl")).toBe("http://localhost");
    expect(getNestedValue(config, "api.auth.token")).toBe("secret");
  });

  it("should return undefined for missing path", () => {
    const config = { api: { baseUrl: "http://localhost" } };
    expect(getNestedValue(config, "api.missing")).toBeUndefined();
    expect(getNestedValue(config, "missing.path")).toBeUndefined();
  });
});

describe("setNestedValue", () => {
  it("should set value at nested path", () => {
    const config = { api: { baseUrl: "http://localhost" } };
    const result = setNestedValue(config, "api.timeout", 5000);
    expect(result).toEqual({
      api: { baseUrl: "http://localhost", timeout: 5000 },
    });
  });

  it("should create intermediate objects", () => {
    const config = {};
    const result = setNestedValue(config, "api.auth.token", "secret");
    expect(result).toEqual({ api: { auth: { token: "secret" } } });
  });

  it("should not modify original object", () => {
    const config = { api: { baseUrl: "http://localhost" } };
    setNestedValue(config, "api.timeout", 5000);
    expect(config.api).toEqual({ baseUrl: "http://localhost" });
  });
});
