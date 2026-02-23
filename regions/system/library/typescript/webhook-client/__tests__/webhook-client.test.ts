import { describe, it, expect } from 'vitest';
import {
  generateSignature,
  verifySignature,
  InMemoryWebhookClient,
} from '../src/index.js';
import type { WebhookPayload } from '../src/index.js';

describe('generateSignature', () => {
  it('HMAC-SHA256のhex文字列を返す', () => {
    const sig = generateSignature('secret', '{"data":1}');
    expect(sig).toMatch(/^[0-9a-f]{64}$/);
  });

  it('同じ入力で同じ署名を返す', () => {
    const s1 = generateSignature('key', 'body');
    const s2 = generateSignature('key', 'body');
    expect(s1).toBe(s2);
  });

  it('異なるsecretで異なる署名を返す', () => {
    const s1 = generateSignature('key1', 'body');
    const s2 = generateSignature('key2', 'body');
    expect(s1).not.toBe(s2);
  });
});

describe('verifySignature', () => {
  it('正しい署名で検証成功する', () => {
    const body = '{"event":"test"}';
    const sig = generateSignature('secret', body);
    expect(verifySignature('secret', body, sig)).toBe(true);
  });

  it('不正な署名で検証失敗する', () => {
    const body = '{"event":"test"}';
    const sig = generateSignature('secret', body);
    expect(verifySignature('wrong-secret', body, sig)).toBe(false);
  });
});

describe('InMemoryWebhookClient', () => {
  it('送信ペイロードを記録する', async () => {
    const client = new InMemoryWebhookClient();
    const payload: WebhookPayload = {
      eventType: 'user.created',
      timestamp: new Date().toISOString(),
      data: { userId: '123' },
    };
    const status = await client.send('https://example.com/webhook', payload);
    expect(status).toBe(200);
    expect(client.getSent()).toHaveLength(1);
    expect(client.getSent()[0].payload.eventType).toBe('user.created');
  });
});
