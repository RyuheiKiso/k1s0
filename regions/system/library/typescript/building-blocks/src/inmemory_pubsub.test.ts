import { describe, it, expect, beforeEach } from 'vitest';
import { InMemoryPubSub } from './inmemory_pubsub.js';
import type { MessageHandler, Message } from './pubsub.js';

/** テスト用: 受信したメッセージを記録するシンプルなハンドラー。 */
function makeHandler(): { handler: MessageHandler; messages: Message[] } {
  const messages: Message[] = [];
  return {
    handler: { handle: async (msg) => { messages.push(msg); } },
    messages,
  };
}

describe('InMemoryPubSub', () => {
  let ps: InMemoryPubSub;

  beforeEach(() => {
    ps = new InMemoryPubSub();
  });

  it('初期状態は uninitialized', async () => {
    expect(await ps.status()).toBe('uninitialized');
  });

  it('init 後は ready になる', async () => {
    await ps.init();
    expect(await ps.status()).toBe('ready');
  });

  it('close 後は closed になりサブスクリプションがクリアされる', async () => {
    await ps.init();
    await ps.close();
    expect(await ps.status()).toBe('closed');
  });

  it('デフォルト name は inmemory-pubsub', () => {
    expect(ps.name).toBe('inmemory-pubsub');
    expect(ps.componentType).toBe('pubsub');
  });

  it('コンストラクタで name を指定できる', () => {
    const named = new InMemoryPubSub('custom-ps');
    expect(named.name).toBe('custom-ps');
  });

  it('metadata は backend=memory を返す', () => {
    expect(ps.metadata()).toEqual({ backend: 'memory' });
  });

  it('publish したメッセージが subscribe で受信できる', async () => {
    await ps.init();
    const { handler, messages } = makeHandler();
    await ps.subscribe('orders', handler);

    await ps.publish('orders', new Uint8Array([1, 2, 3]));

    expect(messages).toHaveLength(1);
    expect(messages[0].topic).toBe('orders');
    expect(messages[0].data).toEqual(new Uint8Array([1, 2, 3]));
  });

  it('メッセージにユニークなIDが付与される', async () => {
    await ps.init();
    const { handler, messages } = makeHandler();
    await ps.subscribe('t', handler);

    await ps.publish('t', new Uint8Array([1]));
    await ps.publish('t', new Uint8Array([2]));

    expect(messages[0].id).toBeTruthy();
    expect(messages[1].id).toBeTruthy();
    expect(messages[0].id).not.toBe(messages[1].id);
  });

  it('メタデータ付きで publish できる', async () => {
    await ps.init();
    const { handler, messages } = makeHandler();
    await ps.subscribe('t', handler);

    await ps.publish('t', new Uint8Array([]), { key: 'value' });

    expect(messages[0].metadata).toEqual({ key: 'value' });
  });

  it('複数のサブスクライバーが同じトピックを受信できる', async () => {
    await ps.init();
    const h1 = makeHandler();
    const h2 = makeHandler();
    await ps.subscribe('events', h1.handler);
    await ps.subscribe('events', h2.handler);

    await ps.publish('events', new Uint8Array([42]));

    expect(h1.messages).toHaveLength(1);
    expect(h2.messages).toHaveLength(1);
  });

  it('別トピックのメッセージは受信しない', async () => {
    await ps.init();
    const { handler, messages } = makeHandler();
    await ps.subscribe('topic-a', handler);

    await ps.publish('topic-b', new Uint8Array([1]));

    expect(messages).toHaveLength(0);
  });

  it('購読者がいないトピックへの publish はエラーにならない', async () => {
    await ps.init();
    await expect(ps.publish('empty', new Uint8Array([]))).resolves.toBeUndefined();
  });

  it('unsubscribe 後はメッセージを受信しない', async () => {
    await ps.init();
    const { handler, messages } = makeHandler();
    const subId = await ps.subscribe('t', handler);
    await ps.unsubscribe(subId);

    await ps.publish('t', new Uint8Array([1]));

    expect(messages).toHaveLength(0);
  });

  it('存在しないサブスクリプションIDの unsubscribe はエラーにならない', async () => {
    await ps.init();
    await expect(ps.unsubscribe('nonexistent-id')).resolves.toBeUndefined();
  });
});
