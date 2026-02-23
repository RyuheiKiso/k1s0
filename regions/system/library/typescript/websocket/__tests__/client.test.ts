import { describe, it, expect } from 'vitest';
import { InMemoryWsClient, defaultConfig } from '../src/index.js';
import type { WsMessage } from '../src/index.js';

describe('InMemoryWsClient', () => {
  it('connect/disconnect で状態が変わる', async () => {
    const client = new InMemoryWsClient();
    expect(client.state).toBe('disconnected');

    await client.connect();
    expect(client.state).toBe('connected');

    await client.disconnect();
    expect(client.state).toBe('disconnected');
  });

  it('二重接続でエラーになる', async () => {
    const client = new InMemoryWsClient();
    await client.connect();
    await expect(client.connect()).rejects.toThrow('Already connected');
  });

  it('二重切断でエラーになる', async () => {
    const client = new InMemoryWsClient();
    await expect(client.disconnect()).rejects.toThrow('Already disconnected');
  });

  it('メッセージを送受信できる', async () => {
    const client = new InMemoryWsClient();
    await client.connect();

    const sendMsg: WsMessage = { type: 'text', payload: 'hello' };
    await client.send(sendMsg);

    const sent = client.getSentMessages();
    expect(sent).toHaveLength(1);
    expect(sent[0].payload).toBe('hello');

    const injected: WsMessage = { type: 'text', payload: 'world' };
    client.injectMessage(injected);

    const received = await client.receive();
    expect(received.payload).toBe('world');
  });

  it('未接続で送信するとエラーになる', async () => {
    const client = new InMemoryWsClient();
    await expect(
      client.send({ type: 'text', payload: 'hello' }),
    ).rejects.toThrow('Not connected');
  });

  it('未接続で受信するとエラーになる', async () => {
    const client = new InMemoryWsClient();
    await expect(client.receive()).rejects.toThrow('Not connected');
  });

  it('defaultConfig が正しい値を返す', () => {
    const cfg = defaultConfig();
    expect(cfg.url).toBe('ws://localhost');
    expect(cfg.reconnect).toBe(true);
    expect(cfg.maxReconnectAttempts).toBe(5);
    expect(cfg.reconnectDelayMs).toBe(1000);
    expect(cfg.pingIntervalMs).toBeUndefined();
  });

  it('injectMessage で待機中の receive が解決される', async () => {
    const client = new InMemoryWsClient();
    await client.connect();

    const receivePromise = client.receive();
    client.injectMessage({ type: 'text', payload: 'async' });

    const msg = await receivePromise;
    expect(msg.payload).toBe('async');
  });
});
