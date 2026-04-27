// @k1s0/config の単体テスト。

import { describe, it, expect } from 'vitest';
import { loadConfig, stubConfig } from '../index';

describe('loadConfig', () => {
  it('BFF_URL が設定されていれば読み込める', () => {
    const cfg = loadConfig({ BFF_URL: 'http://localhost:9090', TENANT_ID: 'tenant-X' });
    expect(cfg.bffUrl).toBe('http://localhost:9090');
    expect(cfg.tenantId).toBe('tenant-X');
    expect(cfg.environment).toBe('dev');
  });

  it('VITE_ プレフィックスを優先する', () => {
    const cfg = loadConfig({
      VITE_BFF_URL: 'http://vite-host:8080',
      BFF_URL: 'http://node-host:8080',
    });
    expect(cfg.bffUrl).toBe('http://vite-host:8080');
  });

  it('BFF_URL なしは throw', () => {
    expect(() => loadConfig({})).toThrow(/BFF_URL is required/);
  });

  it('未知の environment は dev フォールバック', () => {
    const cfg = loadConfig({ BFF_URL: 'http://x', ENVIRONMENT: 'something-else' });
    expect(cfg.environment).toBe('dev');
  });
});

describe('stubConfig', () => {
  it('overrides を受ける', () => {
    const cfg = stubConfig({ tenantId: 'tenant-Y' });
    expect(cfg.tenantId).toBe('tenant-Y');
    expect(cfg.bffUrl).toBe('http://localhost:8080');
  });
});
