import { vi, describe, it, expect } from 'vitest';
import {
  withRetry,
  computeDelay,
  RetryError,
  CircuitBreaker,
  defaultRetryConfig,
  type RetryConfig,
} from '../src/index.js';

describe('computeDelay', () => {
  it('jitter無しで指数バックオフを計算する', () => {
    const config: RetryConfig = { ...defaultRetryConfig, jitter: false };
    expect(computeDelay(config, 0)).toBe(100);
    expect(computeDelay(config, 1)).toBe(200);
    expect(computeDelay(config, 2)).toBe(400);
  });

  it('maxDelayMsで上限制限される', () => {
    const config: RetryConfig = { ...defaultRetryConfig, jitter: false, maxDelayMs: 150 };
    expect(computeDelay(config, 0)).toBe(100);
    expect(computeDelay(config, 1)).toBe(150);
    expect(computeDelay(config, 5)).toBe(150);
  });

  it('jitter有りで近似値を返す', () => {
    const config: RetryConfig = { ...defaultRetryConfig, jitter: true };
    const delay = computeDelay(config, 0);
    expect(delay).toBeGreaterThanOrEqual(90);
    expect(delay).toBeLessThanOrEqual(110);
  });
});

describe('withRetry', () => {
  it('成功時に結果を返す', async () => {
    const op = vi.fn().mockResolvedValue('ok');
    const config: RetryConfig = { ...defaultRetryConfig, maxAttempts: 3 };
    const result = await withRetry(config, op);
    expect(result).toBe('ok');
    expect(op).toHaveBeenCalledTimes(1);
  });

  it('リトライ後に成功する', async () => {
    const op = vi
      .fn()
      .mockRejectedValueOnce(new Error('fail-1'))
      .mockResolvedValueOnce('ok');
    const config: RetryConfig = { ...defaultRetryConfig, maxAttempts: 3, initialDelayMs: 1, jitter: false };
    const result = await withRetry(config, op);
    expect(result).toBe('ok');
    expect(op).toHaveBeenCalledTimes(2);
  });

  it('全リトライ失敗でRetryErrorを投げる', async () => {
    const op = vi.fn().mockRejectedValue(new Error('always-fail'));
    const config: RetryConfig = { ...defaultRetryConfig, maxAttempts: 3, initialDelayMs: 1, jitter: false };
    await expect(withRetry(config, op)).rejects.toThrow(RetryError);
    expect(op).toHaveBeenCalledTimes(3);
  });

  it('RetryErrorにattemptsとlastErrorが含まれる', async () => {
    const lastErr = new Error('last');
    const op = vi.fn().mockRejectedValue(lastErr);
    const config: RetryConfig = { ...defaultRetryConfig, maxAttempts: 2, initialDelayMs: 1, jitter: false };
    try {
      await withRetry(config, op);
    } catch (e) {
      expect(e).toBeInstanceOf(RetryError);
      const retryErr = e as RetryError;
      expect(retryErr.attempts).toBe(2);
      expect(retryErr.lastError).toBe(lastErr);
    }
  });
});

describe('CircuitBreaker', () => {
  it('初期状態はclosed', () => {
    const cb = new CircuitBreaker();
    expect(cb.getState()).toBe('closed');
    expect(cb.isOpen()).toBe(false);
  });

  it('失敗がthresholdに達するとopenになる', () => {
    const cb = new CircuitBreaker({ failureThreshold: 3 });
    cb.recordFailure();
    cb.recordFailure();
    expect(cb.getState()).toBe('closed');
    cb.recordFailure();
    expect(cb.getState()).toBe('open');
    expect(cb.isOpen()).toBe(true);
  });

  it('成功で失敗カウントがリセットされる', () => {
    const cb = new CircuitBreaker({ failureThreshold: 3 });
    cb.recordFailure();
    cb.recordFailure();
    cb.recordSuccess();
    cb.recordFailure();
    cb.recordFailure();
    expect(cb.getState()).toBe('closed');
  });

  it('タイムアウト後にhalf-openに遷移する', () => {
    const cb = new CircuitBreaker({ failureThreshold: 1, timeoutMs: 50 });
    cb.recordFailure();
    expect(cb.getState()).toBe('open');

    vi.useFakeTimers();
    vi.advanceTimersByTime(60);
    expect(cb.getState()).toBe('half-open');
    vi.useRealTimers();
  });

  it('half-openで成功がthresholdに達するとclosedになる', () => {
    const cb = new CircuitBreaker({
      failureThreshold: 1,
      successThreshold: 2,
      timeoutMs: 50,
    });
    cb.recordFailure();
    expect(cb.getState()).toBe('open');

    vi.useFakeTimers();
    vi.advanceTimersByTime(60);
    expect(cb.getState()).toBe('half-open');

    cb.recordSuccess();
    expect(cb.getState()).toBe('half-open');
    cb.recordSuccess();
    expect(cb.getState()).toBe('closed');
    vi.useRealTimers();
  });
});
