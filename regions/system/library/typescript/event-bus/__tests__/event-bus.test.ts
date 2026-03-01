import { vi, describe, it, expect, beforeEach } from 'vitest';
import {
  InMemoryEventBus,
  EventBus,
  EventBusError,
} from '../src/index.js';
import type {
  Event,
  DomainEvent,
  EventHandler,
  EventSubscription,
} from '../src/index.js';

// ---------- helpers ----------

function makeEvent(eventType: string): Event {
  return {
    id: 'evt-1',
    eventType,
    aggregateId: 'agg-1',
    occurredAt: new Date(),
    payload: { key: 'value' },
    timestamp: new Date().toISOString(),
  };
}

function makeDomainEvent(eventType: string, aggregateId = 'agg-1'): DomainEvent {
  return { eventType, aggregateId, occurredAt: new Date() };
}

// ---------- Legacy InMemoryEventBus ----------

describe('InMemoryEventBus', () => {
  it('subscribeしたハンドラがpublish時に呼ばれる', async () => {
    const bus = new InMemoryEventBus();
    const handler = vi.fn();
    bus.subscribe('user.created', handler);
    const event = makeEvent('user.created');
    await bus.publish(event);
    expect(handler).toHaveBeenCalledWith(event);
  });

  it('unsubscribeするとハンドラが呼ばれなくなる', async () => {
    const bus = new InMemoryEventBus();
    const handler = vi.fn();
    bus.subscribe('user.created', handler);
    bus.unsubscribe('user.created');
    await bus.publish(makeEvent('user.created'));
    expect(handler).not.toHaveBeenCalled();
  });

  it('異なるイベントタイプのハンドラは呼ばれない', async () => {
    const bus = new InMemoryEventBus();
    const handler = vi.fn();
    bus.subscribe('user.created', handler);
    await bus.publish(makeEvent('order.placed'));
    expect(handler).not.toHaveBeenCalled();
  });

  it('同じイベントタイプに複数のハンドラを登録できる', async () => {
    const bus = new InMemoryEventBus();
    const h1 = vi.fn();
    const h2 = vi.fn();
    bus.subscribe('user.created', h1);
    bus.subscribe('user.created', h2);
    await bus.publish(makeEvent('user.created'));
    expect(h1).toHaveBeenCalledTimes(1);
    expect(h2).toHaveBeenCalledTimes(1);
  });

  it('ハンドラがない場合もpublishがエラーにならない', async () => {
    const bus = new InMemoryEventBus();
    await expect(bus.publish(makeEvent('unknown'))).resolves.toBeUndefined();
  });
});

// ---------- DDD EventBus ----------

describe('EventBus', () => {
  let bus: EventBus;

  beforeEach(() => {
    bus = new EventBus();
  });

  describe('publish / subscribe', () => {
    it('subscribedハンドラがpublish時に呼ばれる', async () => {
      const received: DomainEvent[] = [];
      const handler: EventHandler<DomainEvent> = {
        handle: async (event) => { received.push(event); },
      };
      bus.subscribe('user.created', handler);
      const event = makeDomainEvent('user.created');
      await bus.publish(event);
      expect(received).toHaveLength(1);
      expect(received[0]).toBe(event);
    });

    it('異なるイベントタイプのハンドラは呼ばれない', async () => {
      const handler: EventHandler = {
        handle: vi.fn(),
      };
      bus.subscribe('user.created', handler);
      await bus.publish(makeDomainEvent('order.placed'));
      expect(handler.handle).not.toHaveBeenCalled();
    });

    it('同じイベントタイプに複数のハンドラを登録できる', async () => {
      const h1: EventHandler = { handle: vi.fn() };
      const h2: EventHandler = { handle: vi.fn() };
      bus.subscribe('test', h1);
      bus.subscribe('test', h2);
      await bus.publish(makeDomainEvent('test'));
      expect(h1.handle).toHaveBeenCalledTimes(1);
      expect(h2.handle).toHaveBeenCalledTimes(1);
    });

    it('ハンドラがない場合もpublishがエラーにならない', async () => {
      await expect(bus.publish(makeDomainEvent('unknown'))).resolves.toBeUndefined();
    });
  });

  describe('EventSubscription', () => {
    it('subscribe()がEventSubscriptionを返す', () => {
      const handler: EventHandler = { handle: vi.fn() };
      const sub = bus.subscribe('test', handler);
      expect(sub.eventType).toBe('test');
      expect(typeof sub.unsubscribe).toBe('function');
    });

    it('unsubscribe()でハンドラが解除される', async () => {
      const handler: EventHandler = { handle: vi.fn() };
      const sub = bus.subscribe('test', handler);
      sub.unsubscribe();
      await bus.publish(makeDomainEvent('test'));
      expect(handler.handle).not.toHaveBeenCalled();
    });

    it('特定のハンドラだけをunsubscribeできる', async () => {
      const h1: EventHandler = { handle: vi.fn() };
      const h2: EventHandler = { handle: vi.fn() };
      const sub1 = bus.subscribe('test', h1);
      bus.subscribe('test', h2);
      sub1.unsubscribe();
      await bus.publish(makeDomainEvent('test'));
      expect(h1.handle).not.toHaveBeenCalled();
      expect(h2.handle).toHaveBeenCalledTimes(1);
    });
  });

  describe('EventBusConfig', () => {
    it('デフォルト設定で動作する', async () => {
      const defaultBus = new EventBus();
      const handler: EventHandler = { handle: vi.fn() };
      defaultBus.subscribe('test', handler);
      await defaultBus.publish(makeDomainEvent('test'));
      expect(handler.handle).toHaveBeenCalledTimes(1);
    });

    it('カスタム設定で動作する', async () => {
      const customBus = new EventBus({ bufferSize: 100, handlerTimeoutMs: 1000 });
      const handler: EventHandler = { handle: vi.fn() };
      customBus.subscribe('test', handler);
      await customBus.publish(makeDomainEvent('test'));
      expect(handler.handle).toHaveBeenCalledTimes(1);
    });

    it('handlerTimeoutMsを超えるとHANDLER_FAILEDエラーになる', async () => {
      const slowBus = new EventBus({ handlerTimeoutMs: 50 });
      const handler: EventHandler = {
        handle: () => new Promise((resolve) => setTimeout(resolve, 200)),
      };
      slowBus.subscribe('test', handler);
      await expect(slowBus.publish(makeDomainEvent('test'))).rejects.toThrow(EventBusError);
      await expect(slowBus.publish(makeDomainEvent('test'))).rejects.toMatchObject({
        code: 'HANDLER_FAILED',
      });
    });
  });

  describe('EventBusError', () => {
    it('HANDLER_FAILEDエラーコードでスローされる', async () => {
      const handler: EventHandler = {
        handle: async () => { throw new Error('boom'); },
      };
      bus.subscribe('test', handler);
      try {
        await bus.publish(makeDomainEvent('test'));
        expect.unreachable('should have thrown');
      } catch (err) {
        expect(err).toBeInstanceOf(EventBusError);
        expect((err as EventBusError).code).toBe('HANDLER_FAILED');
      }
    });

    it('CHANNEL_CLOSEDエラーコード: close後のpublish', async () => {
      bus.close();
      try {
        await bus.publish(makeDomainEvent('test'));
        expect.unreachable('should have thrown');
      } catch (err) {
        expect(err).toBeInstanceOf(EventBusError);
        expect((err as EventBusError).code).toBe('CHANNEL_CLOSED');
      }
    });

    it('CHANNEL_CLOSEDエラーコード: close後のsubscribe', () => {
      bus.close();
      const handler: EventHandler = { handle: vi.fn() };
      expect(() => bus.subscribe('test', handler)).toThrow(EventBusError);
      try {
        bus.subscribe('test', handler);
      } catch (err) {
        expect((err as EventBusError).code).toBe('CHANNEL_CLOSED');
      }
    });

    it('EventBusErrorのプロパティが正しい', () => {
      const err = new EventBusError('test message', 'PUBLISH_FAILED');
      expect(err.message).toBe('test message');
      expect(err.code).toBe('PUBLISH_FAILED');
      expect(err.name).toBe('EventBusError');
      expect(err).toBeInstanceOf(Error);
    });
  });

  describe('DomainEvent', () => {
    it('DomainEventのプロパティが正しい', () => {
      const event = makeDomainEvent('user.created', 'user-123');
      expect(event.eventType).toBe('user.created');
      expect(event.aggregateId).toBe('user-123');
      expect(event.occurredAt).toBeInstanceOf(Date);
    });
  });
});
