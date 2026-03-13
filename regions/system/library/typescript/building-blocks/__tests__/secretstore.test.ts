import { describe, it, expect, vi } from 'vitest';
import type { SecretValue, SecretStore } from '../src/secretstore.js';
import type { ComponentStatus } from '../src/component.js';

describe('SecretValue', () => {
  it('should create a secret with key, value, and metadata', () => {
    const secret: SecretValue = {
      key: 'db-password',
      value: 's3cret!',
      metadata: { store: 'vault', version: '2' },
    };

    expect(secret.key).toBe('db-password');
    expect(secret.value).toBe('s3cret!');
    expect(secret.metadata).toEqual({ store: 'vault', version: '2' });
  });

  it('should support empty metadata', () => {
    const secret: SecretValue = {
      key: 'api-key',
      value: 'abc123',
      metadata: {},
    };

    expect(secret.metadata).toEqual({});
  });
});

describe('SecretStore interface', () => {
  function createMockStore(): SecretStore {
    const secrets = new Map<string, SecretValue>([
      ['db-password', { key: 'db-password', value: 's3cret', metadata: {} }],
      ['api-key', { key: 'api-key', value: 'key-123', metadata: { env: 'prod' } }],
    ]);

    return {
      name: 'test-secretstore',
      componentType: 'secretstore',
      init: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      status: vi.fn().mockResolvedValue('ready' as ComponentStatus),
      metadata: () => ({}),
      getSecret: vi.fn().mockImplementation(async (key: string) => {
        const secret = secrets.get(key);
        if (!secret) throw new Error(`secret not found: ${key}`);
        return secret;
      }),
      bulkGet: vi.fn().mockImplementation(async (keys: string[]) => {
        const result: Record<string, SecretValue> = {};
        for (const key of keys) {
          const secret = secrets.get(key);
          if (secret) result[key] = secret;
        }
        return result;
      }),
    };
  }

  it('should get a single secret by key', async () => {
    const store = createMockStore();
    const secret = await store.getSecret('db-password');

    expect(secret.key).toBe('db-password');
    expect(secret.value).toBe('s3cret');
  });

  it('should throw for non-existent secret', async () => {
    const store = createMockStore();
    await expect(store.getSecret('missing')).rejects.toThrow('secret not found: missing');
  });

  it('should bulk get multiple secrets', async () => {
    const store = createMockStore();
    const result = await store.bulkGet(['db-password', 'api-key', 'missing']);

    expect(Object.keys(result)).toHaveLength(2);
    expect(result['db-password'].value).toBe('s3cret');
    expect(result['api-key'].metadata).toEqual({ env: 'prod' });
  });
});
