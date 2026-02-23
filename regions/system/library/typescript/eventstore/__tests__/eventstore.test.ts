import { describe, it, expect } from 'vitest';
import {
  InMemoryEventStore,
  InMemorySnapshotStore,
  VersionConflictError,
  type StreamId,
  type Snapshot,
} from '../src/index.js';

function makeEvent(streamId: StreamId, eventType: string) {
  return { streamId, eventType, payload: { key: 'value' } };
}

describe('InMemoryEventStore', () => {
  it('append/loadでイベントを保存・取得できる', async () => {
    const store = new InMemoryEventStore();
    const sid: StreamId = 'order-123';
    const events = [makeEvent(sid, 'OrderCreated'), makeEvent(sid, 'OrderConfirmed')];

    await store.append(sid, events);
    const loaded = await store.load(sid);
    expect(loaded).toHaveLength(2);
    expect(loaded[0].eventType).toBe('OrderCreated');
    expect(loaded[1].eventType).toBe('OrderConfirmed');
    expect(loaded[0].version).toBe(1);
    expect(loaded[1].version).toBe(2);
  });

  it('append前はexistsがfalseを返す', async () => {
    const store = new InMemoryEventStore();
    expect(await store.exists('order-999')).toBe(false);
  });

  it('append後はexistsがtrueを返す', async () => {
    const store = new InMemoryEventStore();
    const sid: StreamId = 'order-100';
    await store.append(sid, [makeEvent(sid, 'OrderCreated')]);
    expect(await store.exists(sid)).toBe(true);
  });

  it('loadFromで指定バージョン以降のみ取得できる', async () => {
    const store = new InMemoryEventStore();
    const sid: StreamId = 'order-200';
    await store.append(sid, [
      makeEvent(sid, 'OrderCreated'),
      makeEvent(sid, 'OrderConfirmed'),
      makeEvent(sid, 'OrderShipped'),
    ]);

    const loaded = await store.loadFrom(sid, 2);
    expect(loaded).toHaveLength(2);
    expect(loaded[0].eventType).toBe('OrderConfirmed');
    expect(loaded[0].version).toBe(2);
    expect(loaded[1].eventType).toBe('OrderShipped');
    expect(loaded[1].version).toBe(3);
  });

  it('間違ったexpectedVersionでVersionConflictErrorを投げる', async () => {
    const store = new InMemoryEventStore();
    const sid: StreamId = 'order-300';
    await store.append(sid, [makeEvent(sid, 'OrderCreated')]);

    await expect(
      store.append(sid, [makeEvent(sid, 'OrderConfirmed')], 0),
    ).rejects.toThrow(VersionConflictError);
  });

  it('正しいexpectedVersionで成功する', async () => {
    const store = new InMemoryEventStore();
    const sid: StreamId = 'order-400';
    await store.append(sid, [makeEvent(sid, 'OrderCreated')]);

    const version = await store.append(sid, [makeEvent(sid, 'OrderConfirmed')], 1);
    expect(version).toBe(2);
  });

  it('currentVersionが正しく更新される', async () => {
    const store = new InMemoryEventStore();
    const sid: StreamId = 'order-500';
    expect(await store.currentVersion(sid)).toBe(0);

    await store.append(sid, [makeEvent(sid, 'OrderCreated')]);
    expect(await store.currentVersion(sid)).toBe(1);

    await store.append(sid, [makeEvent(sid, 'OrderConfirmed'), makeEvent(sid, 'OrderShipped')], 1);
    expect(await store.currentVersion(sid)).toBe(3);
  });

  it('未存在ストリームのloadは空配列を返す', async () => {
    const store = new InMemoryEventStore();
    const events = await store.load('nonexistent');
    expect(events).toHaveLength(0);
  });

  it('複数イベントでバージョンが連続増加する', async () => {
    const store = new InMemoryEventStore();
    const sid: StreamId = 'order-700';
    const events = [
      makeEvent(sid, 'OrderCreated'),
      makeEvent(sid, 'ItemAdded'),
      makeEvent(sid, 'ItemAdded'),
      makeEvent(sid, 'OrderConfirmed'),
    ];

    const finalVersion = await store.append(sid, events);
    expect(finalVersion).toBe(4);

    const loaded = await store.load(sid);
    expect(loaded).toHaveLength(4);
    loaded.forEach((event, i) => {
      expect(event.version).toBe(i + 1);
    });
  });

  it('VersionConflictErrorにexpectedとactualが含まれる', async () => {
    const store = new InMemoryEventStore();
    const sid: StreamId = 'order-800';
    await store.append(sid, [makeEvent(sid, 'OrderCreated')]);

    try {
      await store.append(sid, [makeEvent(sid, 'Fail')], 5);
    } catch (e) {
      expect(e).toBeInstanceOf(VersionConflictError);
      const err = e as VersionConflictError;
      expect(err.expected).toBe(5);
      expect(err.actual).toBe(1);
    }
  });
});

describe('InMemorySnapshotStore', () => {
  it('スナップショットを保存・読み込みできる', async () => {
    const store = new InMemorySnapshotStore();
    const snapshot: Snapshot = {
      streamId: 'order-600',
      version: 5,
      state: { status: 'shipped' },
      createdAt: new Date(),
    };

    await store.saveSnapshot(snapshot);
    const loaded = await store.loadSnapshot('order-600');
    expect(loaded).not.toBeNull();
    expect(loaded!.version).toBe(5);
    expect(loaded!.state).toEqual({ status: 'shipped' });
  });

  it('存在しないスナップショットはnullを返す', async () => {
    const store = new InMemorySnapshotStore();
    expect(await store.loadSnapshot('nonexistent')).toBeNull();
  });
});
