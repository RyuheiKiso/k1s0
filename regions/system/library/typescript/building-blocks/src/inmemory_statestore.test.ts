import { describe, it, expect, beforeEach } from 'vitest';
import { InMemoryStateStore } from './inmemory_statestore.js';
import { ETagMismatchError } from './errors.js';

// InMemoryStateStore のテスト: ETag を使った楽観的ロック、一括操作、削除など状態ストアの全機能を検証する。
describe('InMemoryStateStore', () => {
  let store: InMemoryStateStore;

  beforeEach(() => {
    store = new InMemoryStateStore();
  });

  it('初期状態は uninitialized', async () => {
    expect(await store.status()).toBe('uninitialized');
  });

  it('init 後は ready になる', async () => {
    await store.init();
    expect(await store.status()).toBe('ready');
  });

  it('close 後は closed になりエントリがクリアされる', async () => {
    await store.init();
    await store.set('k', new Uint8Array([1]));
    await store.close();
    expect(await store.status()).toBe('closed');
    // クローズ後はエントリが消えていることを確認する
    expect(await store.get('k')).toBeNull();
  });

  it('デフォルト name は inmemory-statestore', () => {
    expect(store.name).toBe('inmemory-statestore');
    expect(store.componentType).toBe('statestore');
  });

  it('コンストラクタで name を指定できる', () => {
    const named = new InMemoryStateStore('custom-store');
    expect(named.name).toBe('custom-store');
  });

  it('metadata は backend=memory を返す', () => {
    expect(store.metadata()).toEqual({ backend: 'memory' });
  });

  it('set した値を get で取得できる', async () => {
    await store.init();
    const etag = await store.set('key', new Uint8Array([10, 20, 30]));
    const entry = await store.get('key');

    expect(entry).not.toBeNull();
    expect(entry!.key).toBe('key');
    expect(entry!.value).toEqual(new Uint8Array([10, 20, 30]));
    expect(entry!.etag).toBe(etag);
  });

  it('存在しないキーの get は null を返す', async () => {
    await store.init();
    expect(await store.get('missing')).toBeNull();
  });

  it('set は毎回新しい ETag を返す', async () => {
    await store.init();
    const etag1 = await store.set('k', new Uint8Array([1]));
    const etag2 = await store.set('k', new Uint8Array([2]));
    expect(etag1).not.toBe(etag2);
  });

  it('正しい ETag で set すると成功する', async () => {
    await store.init();
    const etag = await store.set('k', new Uint8Array([1]));
    const newEtag = await store.set('k', new Uint8Array([2]), etag);
    expect(newEtag).not.toBe(etag);

    const entry = await store.get('k');
    expect(entry!.value).toEqual(new Uint8Array([2]));
  });

  it('古い ETag で set すると ETagMismatchError になる', async () => {
    await store.init();
    await store.set('k', new Uint8Array([1]));
    await expect(store.set('k', new Uint8Array([2]), 'stale-etag')).rejects.toBeInstanceOf(ETagMismatchError);
  });

  it('存在しないキーに ETag 付きで set すると ETagMismatchError になる', async () => {
    await store.init();
    await expect(store.set('missing', new Uint8Array([1]), 'any-etag')).rejects.toBeInstanceOf(ETagMismatchError);
  });

  it('delete でエントリを削除できる', async () => {
    await store.init();
    const etag = await store.set('k', new Uint8Array([1]));
    await store.delete('k', etag);
    expect(await store.get('k')).toBeNull();
  });

  it('存在しないキーの delete はエラーにならない', async () => {
    await store.init();
    await expect(store.delete('missing')).resolves.toBeUndefined();
  });

  it('ETag なしで delete すると無条件削除される', async () => {
    await store.init();
    await store.set('k', new Uint8Array([1]));
    await store.delete('k');
    expect(await store.get('k')).toBeNull();
  });

  it('古い ETag で delete すると ETagMismatchError になる', async () => {
    await store.init();
    await store.set('k', new Uint8Array([1]));
    await expect(store.delete('k', 'stale-etag')).rejects.toBeInstanceOf(ETagMismatchError);
  });

  it('bulkGet で複数エントリを取得できる', async () => {
    await store.init();
    await store.set('a', new Uint8Array([1]));
    await store.set('b', new Uint8Array([2]));

    const entries = await store.bulkGet(['a', 'b']);
    expect(entries).toHaveLength(2);
    expect(entries[0].value).toEqual(new Uint8Array([1]));
    expect(entries[1].value).toEqual(new Uint8Array([2]));
  });

  it('bulkGet は存在しないキーをスキップする', async () => {
    await store.init();
    await store.set('a', new Uint8Array([1]));

    const entries = await store.bulkGet(['a', 'missing']);
    expect(entries).toHaveLength(1);
    expect(entries[0].key).toBe('a');
  });

  it('bulkSet で複数エントリを一括保存できる', async () => {
    await store.init();
    const etags = await store.bulkSet([
      { key: 'x', value: new Uint8Array([10]) },
      { key: 'y', value: new Uint8Array([20]) },
    ]);
    expect(etags).toHaveLength(2);

    const x = await store.get('x');
    const y = await store.get('y');
    expect(x!.value).toEqual(new Uint8Array([10]));
    expect(y!.value).toEqual(new Uint8Array([20]));
  });
});
