import { describe, it, expect, vi } from 'vitest';
import type { StateEntry, StateStore } from '../src/statestore.js';
import type { ComponentStatus } from '../src/component.js';

describe('StateEntry', () => {
  it('should create a state entry with key, value, and etag', () => {
    const entry: StateEntry = {
      key: 'user:123',
      value: new Uint8Array([10, 20, 30]),
      etag: 'etag-abc',
    };

    expect(entry.key).toBe('user:123');
    expect(entry.value).toBeInstanceOf(Uint8Array);
    expect(entry.value).toHaveLength(3);
    expect(entry.etag).toBe('etag-abc');
  });

  it('should support empty value', () => {
    const entry: StateEntry = {
      key: 'empty',
      value: new Uint8Array(),
      etag: '',
    };

    expect(entry.value).toHaveLength(0);
    expect(entry.etag).toBe('');
  });
});

describe('StateStore interface', () => {
  function createMockStore(): StateStore {
    const store = new Map<string, StateEntry>();

    return {
      name: 'test-statestore',
      componentType: 'statestore',
      init: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      status: vi.fn().mockResolvedValue('ready' as ComponentStatus),
      metadata: () => ({}),
      get: vi.fn().mockImplementation(async (key: string) => store.get(key) ?? null),
      set: vi.fn().mockImplementation(async (key: string, value: Uint8Array, _etag?: string) => {
        const etag = `etag-${Date.now()}`;
        store.set(key, { key, value, etag });
        return etag;
      }),
      delete: vi.fn().mockImplementation(async (key: string) => {
        store.delete(key);
      }),
      bulkGet: vi.fn().mockImplementation(async (keys: string[]) =>
        keys.map(k => store.get(k)).filter((e): e is StateEntry => e !== undefined),
      ),
      bulkSet: vi.fn().mockImplementation(async (entries: Array<{ key: string; value: Uint8Array }>) =>
        entries.map(e => {
          const etag = `etag-${e.key}`;
          store.set(e.key, { key: e.key, value: e.value, etag });
          return etag;
        }),
      ),
    };
  }

  it('should get and set state entries', async () => {
    const store = createMockStore();
    const value = new Uint8Array([1, 2, 3]);

    const etag = await store.set('key1', value);
    expect(etag).toBeTruthy();

    const entry = await store.get('key1');
    expect(entry).not.toBeNull();
    expect(entry!.key).toBe('key1');
    expect(entry!.value).toEqual(value);
  });

  it('should return null for non-existent keys', async () => {
    const store = createMockStore();
    const entry = await store.get('missing');
    expect(entry).toBeNull();
  });

  it('should delete state entries', async () => {
    const store = createMockStore();
    await store.set('key1', new Uint8Array([1]));
    await store.delete('key1');
    const entry = await store.get('key1');
    expect(entry).toBeNull();
  });

  it('should support set with optional etag', async () => {
    const store = createMockStore();
    await store.set('key1', new Uint8Array([1]), 'etag-old');
    expect(store.set).toHaveBeenCalledWith('key1', expect.any(Uint8Array), 'etag-old');
  });

  it('should bulk get multiple entries', async () => {
    const store = createMockStore();
    await store.set('a', new Uint8Array([1]));
    await store.set('b', new Uint8Array([2]));

    const entries = await store.bulkGet(['a', 'b', 'c']);
    expect(entries).toHaveLength(2);
  });

  it('should bulk set multiple entries and return etags', async () => {
    const store = createMockStore();
    const etags = await store.bulkSet([
      { key: 'x', value: new Uint8Array([10]) },
      { key: 'y', value: new Uint8Array([20]) },
    ]);
    expect(etags).toHaveLength(2);
    expect(etags[0]).toBe('etag-x');
    expect(etags[1]).toBe('etag-y');
  });
});
