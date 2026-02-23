import { describe, it, expect } from 'vitest';
import { InMemoryVaultClient, VaultError } from '../src/index.js';
import type { Secret, VaultClientConfig } from '../src/index.js';

function makeConfig(): VaultClientConfig {
  return { serverUrl: 'http://localhost:8080', cacheTtlMs: 600000 };
}

function makeSecret(path: string): Secret {
  return {
    path,
    data: { password: 's3cr3t', username: 'admin' },
    version: 1,
    createdAt: new Date(),
  };
}

describe('InMemoryVaultClient', () => {
  it('シークレットを取得できること', async () => {
    const client = new InMemoryVaultClient(makeConfig());
    client.putSecret(makeSecret('system/db/primary'));
    const secret = await client.getSecret('system/db/primary');
    expect(secret.path).toBe('system/db/primary');
    expect(secret.data['password']).toBe('s3cr3t');
  });

  it('存在しないシークレットでNOT_FOUNDエラーが返ること', async () => {
    const client = new InMemoryVaultClient(makeConfig());
    await expect(client.getSecret('missing/path')).rejects.toThrow(VaultError);
    try {
      await client.getSecret('missing/path');
    } catch (e) {
      expect((e as VaultError).code).toBe('NOT_FOUND');
    }
  });

  it('シークレットの値を直接取得できること', async () => {
    const client = new InMemoryVaultClient(makeConfig());
    client.putSecret(makeSecret('system/db'));
    const value = await client.getSecretValue('system/db', 'password');
    expect(value).toBe('s3cr3t');
  });

  it('存在しないキーでNOT_FOUNDエラーが返ること', async () => {
    const client = new InMemoryVaultClient(makeConfig());
    client.putSecret(makeSecret('system/db'));
    await expect(client.getSecretValue('system/db', 'missing')).rejects.toThrow(VaultError);
  });

  it('プレフィックスでシークレット一覧を取得できること', async () => {
    const client = new InMemoryVaultClient(makeConfig());
    client.putSecret(makeSecret('system/db/primary'));
    client.putSecret(makeSecret('system/db/replica'));
    client.putSecret(makeSecret('business/api/key'));
    const paths = await client.listSecrets('system/');
    expect(paths).toHaveLength(2);
    expect(paths.every((p) => p.startsWith('system/'))).toBe(true);
  });

  it('一致しないプレフィックスで空配列を返すこと', async () => {
    const client = new InMemoryVaultClient(makeConfig());
    const paths = await client.listSecrets('nothing/');
    expect(paths).toHaveLength(0);
  });

  it('watchSecretがAsyncIterableを返すこと', async () => {
    const client = new InMemoryVaultClient(makeConfig());
    const watcher = client.watchSecret('system/db');
    expect(watcher).toBeDefined();
  });

  it('VaultErrorにcodeが設定されていること', () => {
    const err = new VaultError('test', 'NOT_FOUND');
    expect(err.code).toBe('NOT_FOUND');
    expect(err.message).toBe('test');
    expect(err.name).toBe('VaultError');
  });

  it('設定値が正しく保持されること', () => {
    const config = makeConfig();
    const client = new InMemoryVaultClient(config);
    expect(client.getConfig().serverUrl).toBe('http://localhost:8080');
    expect(client.getConfig().cacheTtlMs).toBe(600000);
  });

  it('SecretRotatedEventの型が正しいこと', async () => {
    const client = new InMemoryVaultClient(makeConfig());
    client.putSecret(makeSecret('system/db'));
    const secret = await client.getSecret('system/db');
    expect(secret.version).toBe(1);
    expect(secret.createdAt).toBeInstanceOf(Date);
  });
});
