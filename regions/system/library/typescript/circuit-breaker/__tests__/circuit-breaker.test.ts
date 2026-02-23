import { vi, describe, it, expect } from 'vitest';
import { CircuitBreaker, CircuitBreakerError } from '../src/index.js';

describe('CircuitBreaker', () => {
  it('初期状態はclosed', () => {
    const cb = new CircuitBreaker({ failureThreshold: 3, successThreshold: 2, timeoutMs: 1000 });
    expect(cb.state).toBe('closed');
    expect(cb.isOpen()).toBe(false);
  });

  it('失敗がthresholdに達するとopenになる', () => {
    const cb = new CircuitBreaker({ failureThreshold: 3, successThreshold: 1, timeoutMs: 1000 });
    cb.recordFailure();
    cb.recordFailure();
    expect(cb.state).toBe('closed');
    cb.recordFailure();
    expect(cb.state).toBe('open');
    expect(cb.isOpen()).toBe(true);
  });

  it('open状態でcallするとCircuitBreakerErrorを投げる', async () => {
    const cb = new CircuitBreaker({ failureThreshold: 1, successThreshold: 1, timeoutMs: 1000 });
    cb.recordFailure();
    await expect(cb.call(async () => 'ok')).rejects.toThrow(CircuitBreakerError);
  });

  it('タイムアウト後にhalf-openに遷移する', () => {
    const cb = new CircuitBreaker({ failureThreshold: 1, successThreshold: 1, timeoutMs: 50 });
    cb.recordFailure();
    expect(cb.state).toBe('open');

    vi.useFakeTimers();
    vi.advanceTimersByTime(60);
    expect(cb.state).toBe('half-open');
    vi.useRealTimers();
  });

  it('half-openで成功するとclosedに戻る', () => {
    const cb = new CircuitBreaker({ failureThreshold: 1, successThreshold: 1, timeoutMs: 50 });
    cb.recordFailure();

    vi.useFakeTimers();
    vi.advanceTimersByTime(60);
    expect(cb.state).toBe('half-open');
    cb.recordSuccess();
    expect(cb.state).toBe('closed');
    vi.useRealTimers();
  });

  it('callが成功するとsuccessを記録する', async () => {
    const cb = new CircuitBreaker({ failureThreshold: 3, successThreshold: 1, timeoutMs: 1000 });
    const result = await cb.call(async () => 42);
    expect(result).toBe(42);
  });
});
