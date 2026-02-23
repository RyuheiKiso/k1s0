import { describe, it, expect } from 'vitest';
import { InMemorySessionClient } from '../src/index.js';

describe('InMemorySessionClient', () => {
  it('セッションを作成できる', async () => {
    const client = new InMemorySessionClient();
    const session = await client.create({
      userId: 'user-1',
      ttlSeconds: 3600,
      metadata: { device: 'mobile' },
    });
    expect(session.id).toBeDefined();
    expect(session.token).toBeDefined();
    expect(session.userId).toBe('user-1');
    expect(session.revoked).toBe(false);
    expect(session.metadata.device).toBe('mobile');
    expect(session.expiresAt.getTime()).toBeGreaterThan(Date.now());
  });

  it('セッションを取得できる', async () => {
    const client = new InMemorySessionClient();
    const created = await client.create({ userId: 'user-1', ttlSeconds: 3600 });

    const got = await client.get(created.id);
    expect(got).not.toBeNull();
    expect(got!.id).toBe(created.id);
  });

  it('存在しないセッションはnullを返す', async () => {
    const client = new InMemorySessionClient();
    const got = await client.get('nonexistent');
    expect(got).toBeNull();
  });

  it('セッションをリフレッシュできる', async () => {
    const client = new InMemorySessionClient();
    const created = await client.create({ userId: 'user-1', ttlSeconds: 60 });
    const oldToken = created.token;

    const refreshed = await client.refresh({ id: created.id, ttlSeconds: 7200 });
    expect(refreshed.token).not.toBe(oldToken);
    expect(refreshed.expiresAt.getTime()).toBeGreaterThan(Date.now() + 3600 * 1000);
  });

  it('存在しないセッションのリフレッシュでエラーになる', async () => {
    const client = new InMemorySessionClient();
    await expect(
      client.refresh({ id: 'nonexistent', ttlSeconds: 3600 }),
    ).rejects.toThrow('Session not found');
  });

  it('セッションを無効化できる', async () => {
    const client = new InMemorySessionClient();
    const created = await client.create({ userId: 'user-1', ttlSeconds: 3600 });

    await client.revoke(created.id);
    const got = await client.get(created.id);
    expect(got!.revoked).toBe(true);
  });

  it('存在しないセッションの無効化でエラーになる', async () => {
    const client = new InMemorySessionClient();
    await expect(client.revoke('nonexistent')).rejects.toThrow('Session not found');
  });

  it('ユーザーのセッション一覧を取得できる', async () => {
    const client = new InMemorySessionClient();
    await client.create({ userId: 'user-1', ttlSeconds: 3600 });
    await client.create({ userId: 'user-1', ttlSeconds: 3600 });
    await client.create({ userId: 'user-2', ttlSeconds: 3600 });

    const sessions = await client.listUserSessions('user-1');
    expect(sessions).toHaveLength(2);
  });

  it('ユーザーの全セッションを無効化できる', async () => {
    const client = new InMemorySessionClient();
    await client.create({ userId: 'user-1', ttlSeconds: 3600 });
    await client.create({ userId: 'user-1', ttlSeconds: 3600 });
    await client.create({ userId: 'user-2', ttlSeconds: 3600 });

    const count = await client.revokeAll('user-1');
    expect(count).toBe(2);

    const sessions = await client.listUserSessions('user-1');
    for (const s of sessions) {
      expect(s.revoked).toBe(true);
    }
  });
});
