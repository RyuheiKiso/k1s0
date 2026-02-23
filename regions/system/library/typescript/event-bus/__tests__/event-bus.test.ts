import { vi, describe, it, expect } from 'vitest';
import { InMemoryEventBus } from '../src/index.js';
import type { Event } from '../src/index.js';

function makeEvent(eventType: string): Event {
  return {
    id: 'evt-1',
    eventType,
    payload: { key: 'value' },
    timestamp: new Date().toISOString(),
  };
}

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
