import { vi, describe, it, expect } from 'vitest';
import { InMemoryCacheClient, RedisCacheClient } from '../src/index.js';

describe('InMemoryCacheClient', () => {
  it('set/getで値を保存・取得できる', async () => {
    const cache = new InMemoryCacheClient();
    await cache.set('key1', 'value1');
    expect(await cache.get('key1')).toBe('value1');
  });

  it('存在しないキーでnullを返す', async () => {
    const cache = new InMemoryCacheClient();
    expect(await cache.get('nonexistent')).toBeNull();
  });

  it('deleteで値を削除できる', async () => {
    const cache = new InMemoryCacheClient();
    await cache.set('key1', 'value1');
    expect(await cache.delete('key1')).toBe(true);
    expect(await cache.get('key1')).toBeNull();
  });

  it('存在しないキーのdeleteはfalseを返す', async () => {
    const cache = new InMemoryCacheClient();
    expect(await cache.delete('nonexistent')).toBe(false);
  });

  it('existsが正しく動作する', async () => {
    const cache = new InMemoryCacheClient();
    expect(await cache.exists('key1')).toBe(false);
    await cache.set('key1', 'value1');
    expect(await cache.exists('key1')).toBe(true);
  });

  it('TTL期限切れで値がnullになる', async () => {
    vi.useFakeTimers();
    const cache = new InMemoryCacheClient();
    await cache.set('key1', 'value1', 1000);
    expect(await cache.get('key1')).toBe('value1');

    vi.advanceTimersByTime(1001);
    expect(await cache.get('key1')).toBeNull();
    vi.useRealTimers();
  });

  it('TTLなしの値は期限切れにならない', async () => {
    vi.useFakeTimers();
    const cache = new InMemoryCacheClient();
    await cache.set('key1', 'value1');

    vi.advanceTimersByTime(999_999);
    expect(await cache.get('key1')).toBe('value1');
    vi.useRealTimers();
  });

  it('setNXは存在しないキーでtrueを返す', async () => {
    const cache = new InMemoryCacheClient();
    expect(await cache.setNX('key1', 'value1', 5000)).toBe(true);
    expect(await cache.get('key1')).toBe('value1');
  });

  it('setNXは存在するキーでfalseを返す', async () => {
    const cache = new InMemoryCacheClient();
    await cache.set('key1', 'value1');
    expect(await cache.setNX('key1', 'value2', 5000)).toBe(false);
    expect(await cache.get('key1')).toBe('value1');
  });

  it('setNXは期限切れキーでtrueを返す', async () => {
    vi.useFakeTimers();
    const cache = new InMemoryCacheClient();
    await cache.set('key1', 'value1', 100);
    vi.advanceTimersByTime(101);
    expect(await cache.setNX('key1', 'value2', 5000)).toBe(true);
    expect(await cache.get('key1')).toBe('value2');
    vi.useRealTimers();
  });
});

describe('RedisCacheClient', () => {
  it('prefixes keys and delegates to redis client', async () => {
    const redis = {
      get: vi.fn().mockResolvedValue('value1'),
      set: vi.fn().mockResolvedValue('OK'),
      del: vi.fn().mockResolvedValue(1),
      exists: vi.fn().mockResolvedValue(1),
    };

    const cache = RedisCacheClient.fromClient(redis, 'app');
    await cache.set('key1', 'value1', 1000);
    await cache.get('key1');
    const deleted = await cache.delete('key1');
    const exists = await cache.exists('key1');
    const setNx = await cache.setNX('key1', 'value2', 1000);

    expect(redis.set).toHaveBeenCalledWith('app:key1', 'value1', 'PX', 1000);
    expect(redis.get).toHaveBeenCalledWith('app:key1');
    expect(redis.del).toHaveBeenCalledWith('app:key1');
    expect(redis.exists).toHaveBeenCalledWith('app:key1');
    expect(redis.set).toHaveBeenCalledWith('app:key1', 'value2', 'PX', 1000, 'NX');
    expect(deleted).toBe(true);
    expect(exists).toBe(true);
    expect(setNx).toBe(true);
  });
});
