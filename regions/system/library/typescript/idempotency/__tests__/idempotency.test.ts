import { vi, describe, it, expect } from 'vitest';
import {
  InMemoryIdempotencyStore,
  IdempotencyError,
  DuplicateKeyError,
  createIdempotencyRecord,
} from '../src/index.js';

describe('InMemoryIdempotencyStore', () => {
  it('insert/getでレコードを保存・取得できる', async () => {
    const store = new InMemoryIdempotencyStore();
    const record = createIdempotencyRecord('req-1');
    await store.insert(record);

    const result = await store.get('req-1');
    expect(result).not.toBeNull();
    expect(result!.key).toBe('req-1');
    expect(result!.status).toBe('pending');
  });

  it('存在しないキーでnullを返す', async () => {
    const store = new InMemoryIdempotencyStore();
    expect(await store.get('nonexistent')).toBeNull();
  });

  it('重複insertでDuplicateKeyErrorを投げる', async () => {
    const store = new InMemoryIdempotencyStore();
    await store.insert(createIdempotencyRecord('req-1'));
    await expect(store.insert(createIdempotencyRecord('req-1'))).rejects.toThrow(
      DuplicateKeyError,
    );
  });

  it('DuplicateKeyErrorにkeyが含まれる', async () => {
    const store = new InMemoryIdempotencyStore();
    await store.insert(createIdempotencyRecord('req-dup'));
    try {
      await store.insert(createIdempotencyRecord('req-dup'));
    } catch (e) {
      expect(e).toBeInstanceOf(DuplicateKeyError);
      expect((e as DuplicateKeyError).key).toBe('req-dup');
    }
  });

  it('updateでステータスを更新できる', async () => {
    const store = new InMemoryIdempotencyStore();
    await store.insert(createIdempotencyRecord('req-1'));
    await store.update('req-1', 'completed', '{"ok":true}', 200);

    const result = await store.get('req-1');
    expect(result!.status).toBe('completed');
    expect(result!.responseBody).toBe('{"ok":true}');
    expect(result!.statusCode).toBe(200);
    expect(result!.completedAt).toBeInstanceOf(Date);
  });

  it('存在しないキーのupdateでエラーを投げる', async () => {
    const store = new InMemoryIdempotencyStore();
    await expect(store.update('nonexistent', 'completed')).rejects.toThrow(IdempotencyError);
  });

  it('deleteでレコードを削除できる', async () => {
    const store = new InMemoryIdempotencyStore();
    await store.insert(createIdempotencyRecord('req-1'));
    expect(await store.delete('req-1')).toBe(true);
    expect(await store.get('req-1')).toBeNull();
  });

  it('存在しないキーのdeleteはfalseを返す', async () => {
    const store = new InMemoryIdempotencyStore();
    expect(await store.delete('nonexistent')).toBe(false);
  });

  it('TTL期限切れのレコードはgetでnullになる', async () => {
    vi.useFakeTimers();
    const store = new InMemoryIdempotencyStore();
    const record = createIdempotencyRecord('req-1', 1);
    await store.insert(record);

    vi.advanceTimersByTime(1001);
    expect(await store.get('req-1')).toBeNull();
    vi.useRealTimers();
  });

  it('TTL期限切れ後は同じキーでinsertできる', async () => {
    vi.useFakeTimers();
    const store = new InMemoryIdempotencyStore();
    await store.insert(createIdempotencyRecord('req-1', 1));

    vi.advanceTimersByTime(1001);
    await expect(store.insert(createIdempotencyRecord('req-1'))).resolves.not.toThrow();
    vi.useRealTimers();
  });

  it('createIdempotencyRecordでexpiresAtが正しく設定される', () => {
    const record = createIdempotencyRecord('req-1', 60);
    expect(record.expiresAt).toBeInstanceOf(Date);
    expect(record.expiresAt!.getTime() - record.createdAt.getTime()).toBe(60_000);
  });
});
