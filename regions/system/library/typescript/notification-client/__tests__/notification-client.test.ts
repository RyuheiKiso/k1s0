import { describe, it, expect } from 'vitest';
import { InMemoryNotificationClient } from '../src/index.js';
import type { NotificationRequest } from '../src/index.js';

function makeRequest(overrides: Partial<NotificationRequest> = {}): NotificationRequest {
  return {
    id: 'notif-1',
    channel: 'email',
    recipient: 'user@example.com',
    subject: 'Test',
    body: 'Hello',
    ...overrides,
  };
}

describe('InMemoryNotificationClient', () => {
  it('通知を送信しレスポンスを返す', async () => {
    const client = new InMemoryNotificationClient();
    const resp = await client.send(makeRequest());
    expect(resp.id).toBe('notif-1');
    expect(resp.status).toBe('sent');
    expect(resp.messageId).toBeTruthy();
  });

  it('送信済み通知を取得できる', async () => {
    const client = new InMemoryNotificationClient();
    await client.send(makeRequest({ channel: 'sms', recipient: '+1234567890' }));
    await client.send(makeRequest({ channel: 'push', recipient: 'device-token' }));
    const sent = client.getSent();
    expect(sent).toHaveLength(2);
    expect(sent[0].channel).toBe('sms');
    expect(sent[1].channel).toBe('push');
  });

  it('各レスポンスに異なるmessageIdが含まれる', async () => {
    const client = new InMemoryNotificationClient();
    const r1 = await client.send(makeRequest({ id: 'n1' }));
    const r2 = await client.send(makeRequest({ id: 'n2' }));
    expect(r1.messageId).not.toBe(r2.messageId);
  });

  it('全チャネル型を受け入れる', async () => {
    const client = new InMemoryNotificationClient();
    for (const ch of ['email', 'sms', 'push', 'webhook'] as const) {
      const resp = await client.send(makeRequest({ id: `n-${ch}`, channel: ch }));
      expect(resp.status).toBe('sent');
    }
    expect(client.getSent()).toHaveLength(4);
  });
});
