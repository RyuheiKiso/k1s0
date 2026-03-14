import { describe, it, expect, beforeEach } from 'vitest';
import { InMemorySecretStore } from './inmemory_secretstore.js';
import { ComponentError } from './errors.js';

describe('InMemorySecretStore', () => {
  let store: InMemorySecretStore;

  beforeEach(() => {
    store = new InMemorySecretStore();
  });

  it('初期状態は uninitialized', async () => {
    expect(await store.status()).toBe('uninitialized');
  });

  it('init 後は ready になる', async () => {
    await store.init();
    expect(await store.status()).toBe('ready');
  });

  it('close 後は closed になりシークレットがクリアされる', async () => {
    await store.init();
    store.put('k', 'v');
    await store.close();
    expect(await store.status()).toBe('closed');
    await expect(store.getSecret('k')).rejects.toBeInstanceOf(ComponentError);
  });

  it('デフォルト name は inmemory-secretstore', () => {
    expect(store.name).toBe('inmemory-secretstore');
    expect(store.componentType).toBe('secretstore');
  });

  it('コンストラクタで name を指定できる', () => {
    const named = new InMemorySecretStore('custom-secrets');
    expect(named.name).toBe('custom-secrets');
  });

  it('metadata は backend=memory を返す', () => {
    expect(store.metadata()).toEqual({ backend: 'memory' });
  });

  it('put したシークレットを getSecret で取得できる', async () => {
    await store.init();
    store.put('db-password', 'secret123');

    const secret = await store.getSecret('db-password');

    expect(secret.key).toBe('db-password');
    expect(secret.value).toBe('secret123');
  });

  it('存在しないキーの getSecret は ComponentError をスローする', async () => {
    await store.init();
    await expect(store.getSecret('missing')).rejects.toBeInstanceOf(ComponentError);
  });

  it('bulkGet で複数シークレットを取得できる', async () => {
    await store.init();
    store.put('k1', 'v1');
    store.put('k2', 'v2');

    const result = await store.bulkGet(['k1', 'k2']);

    expect(result['k1'].value).toBe('v1');
    expect(result['k2'].value).toBe('v2');
  });

  it('bulkGet でいずれかのキーが存在しない場合は ComponentError をスローする', async () => {
    await store.init();
    store.put('k1', 'v1');

    await expect(store.bulkGet(['k1', 'missing'])).rejects.toBeInstanceOf(ComponentError);
  });

  it('put で既存の値を上書きできる', async () => {
    await store.init();
    store.put('key', 'old');
    store.put('key', 'new');

    const secret = await store.getSecret('key');
    expect(secret.value).toBe('new');
  });
});
