import { describe, it, expect, vi } from 'vitest';
import type { Message, MessageHandler, PubSub } from '../src/pubsub.js';
import type { ComponentStatus } from '../src/component.js';

describe('Message', () => {
  it('should create a message with all required fields', () => {
    const msg: Message = {
      topic: 'orders',
      data: new Uint8Array([1, 2, 3]),
      metadata: { source: 'test' },
      id: 'msg-001',
    };

    expect(msg.topic).toBe('orders');
    expect(msg.data).toBeInstanceOf(Uint8Array);
    expect(msg.data).toHaveLength(3);
    expect(msg.metadata).toEqual({ source: 'test' });
    expect(msg.id).toBe('msg-001');
  });

  it('should support empty metadata', () => {
    const msg: Message = {
      topic: 'events',
      data: new Uint8Array(),
      metadata: {},
      id: 'msg-002',
    };

    expect(msg.metadata).toEqual({});
    expect(msg.data).toHaveLength(0);
  });
});

describe('MessageHandler', () => {
  it('should be implementable as an async handler', async () => {
    const received: Message[] = [];
    const handler: MessageHandler = {
      handle: vi.fn().mockImplementation(async (msg: Message) => {
        received.push(msg);
      }),
    };

    const msg: Message = {
      topic: 'test',
      data: new Uint8Array([42]),
      metadata: {},
      id: 'msg-100',
    };

    await handler.handle(msg);
    expect(handler.handle).toHaveBeenCalledWith(msg);
    expect(received).toHaveLength(1);
    expect(received[0].id).toBe('msg-100');
  });
});

describe('PubSub interface', () => {
  it('should be implementable with publish, subscribe, and unsubscribe', async () => {
    const subscriptions = new Map<string, MessageHandler>();

    const mock: PubSub = {
      name: 'test-pubsub',
      componentType: 'pubsub',
      init: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      status: vi.fn().mockResolvedValue('ready' as ComponentStatus),
      metadata: () => ({}),
      publish: vi.fn().mockResolvedValue(undefined),
      subscribe: vi.fn().mockImplementation(async (topic: string, handler: MessageHandler) => {
        const id = `sub-${topic}`;
        subscriptions.set(id, handler);
        return id;
      }),
      unsubscribe: vi.fn().mockImplementation(async (id: string) => {
        subscriptions.delete(id);
      }),
    };

    await mock.publish('orders', new Uint8Array([1]), { key: 'val' });
    expect(mock.publish).toHaveBeenCalledWith('orders', expect.any(Uint8Array), { key: 'val' });

    const handler: MessageHandler = { handle: vi.fn() };
    const subId = await mock.subscribe('orders', handler);
    expect(subId).toBe('sub-orders');
    expect(subscriptions.size).toBe(1);

    await mock.unsubscribe(subId);
    expect(subscriptions.size).toBe(0);
  });

  it('should support publish without optional metadata', async () => {
    const mock: PubSub = {
      name: 'pubsub-no-meta',
      componentType: 'pubsub',
      init: vi.fn(),
      close: vi.fn(),
      status: vi.fn().mockResolvedValue('ready' as ComponentStatus),
      metadata: () => ({}),
      publish: vi.fn().mockResolvedValue(undefined),
      subscribe: vi.fn().mockResolvedValue('sub-1'),
      unsubscribe: vi.fn().mockResolvedValue(undefined),
    };

    await mock.publish('topic', new Uint8Array());
    expect(mock.publish).toHaveBeenCalledWith('topic', expect.any(Uint8Array));
  });
});
