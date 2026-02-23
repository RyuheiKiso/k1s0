import { describe, it, expect } from 'vitest';
import { HealthChecker } from '../src/index.js';
import type { HealthCheck } from '../src/index.js';

describe('HealthChecker', () => {
  it('チェックがない場合はhealthyを返す', async () => {
    const checker = new HealthChecker();
    const resp = await checker.runAll();
    expect(resp.status).toBe('healthy');
    expect(Object.keys(resp.checks)).toHaveLength(0);
    expect(resp.timestamp).toBeTruthy();
  });

  it('全チェック成功でhealthyを返す', async () => {
    const checker = new HealthChecker();
    const dbCheck: HealthCheck = {
      name: 'database',
      check: async () => {},
    };
    const cacheCheck: HealthCheck = {
      name: 'cache',
      check: async () => {},
    };
    checker.add(dbCheck);
    checker.add(cacheCheck);

    const resp = await checker.runAll();
    expect(resp.status).toBe('healthy');
    expect(resp.checks['database'].status).toBe('healthy');
    expect(resp.checks['cache'].status).toBe('healthy');
  });

  it('チェック失敗でunhealthyを返す', async () => {
    const checker = new HealthChecker();
    const failCheck: HealthCheck = {
      name: 'database',
      check: async () => { throw new Error('connection refused'); },
    };
    checker.add(failCheck);

    const resp = await checker.runAll();
    expect(resp.status).toBe('unhealthy');
    expect(resp.checks['database'].status).toBe('unhealthy');
    expect(resp.checks['database'].message).toBe('connection refused');
  });

  it('timestampがISO文字列であること', async () => {
    const checker = new HealthChecker();
    const resp = await checker.runAll();
    expect(() => new Date(resp.timestamp)).not.toThrow();
  });
});
