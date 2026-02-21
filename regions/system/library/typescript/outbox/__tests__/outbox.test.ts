import { describe, it, expect } from 'vitest';
import {
  OutboxMessage,
  OutboxStatus,
  createOutboxMessage,
  nextScheduledAt,
  canTransitionTo,
  OutboxStore,
  OutboxPublisher,
  OutboxProcessor,
  OutboxError,
} from '../src/index.js';

const UUID_V4_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;

// --- インラインモック ---

class MockStore implements OutboxStore {
  messages: OutboxMessage[] = [];
  savedMessages: OutboxMessage[] = [];
  statusUpdates: Array<{ id: string; status: OutboxStatus; retryCount?: number; scheduledAt?: Date }> = [];
  saveErr?: Error;
  getErr?: Error;
  updateErr?: Error;

  async saveMessage(msg: OutboxMessage): Promise<void> {
    if (this.saveErr) throw this.saveErr;
    this.savedMessages.push(msg);
  }

  async getPendingMessages(limit: number): Promise<OutboxMessage[]> {
    if (this.getErr) throw this.getErr;
    return this.messages.slice(0, limit);
  }

  async updateStatus(id: string, status: OutboxStatus): Promise<void> {
    if (this.updateErr) throw this.updateErr;
    this.statusUpdates.push({ id, status });
  }

  async updateStatusWithRetry(id: string, status: OutboxStatus, retryCount: number, scheduledAt: Date): Promise<void> {
    if (this.updateErr) throw this.updateErr;
    this.statusUpdates.push({ id, status, retryCount, scheduledAt });
  }
}

class MockPublisher implements OutboxPublisher {
  published: OutboxMessage[] = [];
  error?: Error;

  async publish(msg: OutboxMessage): Promise<void> {
    if (this.error) throw this.error;
    this.published.push(msg);
  }
}

// --- テスト ---

describe('createOutboxMessage', () => {
  it('has correct fields', () => {
    const msg = createOutboxMessage('k1s0.system.user.created.v1', 'user.created.v1', '{"id":"1"}', 'corr-123');
    expect(msg.id).toMatch(UUID_V4_REGEX);
    expect(msg.topic).toBe('k1s0.system.user.created.v1');
    expect(msg.eventType).toBe('user.created.v1');
    expect(msg.payload).toBe('{"id":"1"}');
    expect(msg.status).toBe('PENDING');
    expect(msg.retryCount).toBe(0);
    expect(msg.correlationId).toBe('corr-123');
    expect(msg.createdAt).toBeInstanceOf(Date);
    expect(msg.updatedAt).toBeInstanceOf(Date);
    expect(msg.scheduledAt).toBeInstanceOf(Date);
  });

  it('generates unique IDs', () => {
    const msg1 = createOutboxMessage('topic', 'event.v1', '{}', 'corr-1');
    const msg2 = createOutboxMessage('topic', 'event.v1', '{}', 'corr-1');
    expect(msg1.id).not.toBe(msg2.id);
  });
});

describe('nextScheduledAt', () => {
  it.each([
    { retryCount: 0, expectedMinutes: 1 },
    { retryCount: 1, expectedMinutes: 2 },
    { retryCount: 2, expectedMinutes: 4 },
    { retryCount: 3, expectedMinutes: 8 },
    { retryCount: 6, expectedMinutes: 60 },
    { retryCount: 7, expectedMinutes: 60 },
  ])('returns ~$expectedMinutes minutes for retryCount=$retryCount', ({ retryCount, expectedMinutes }) => {
    const before = Date.now();
    const scheduled = nextScheduledAt(retryCount);
    const after = Date.now();
    const delayMs = scheduled.getTime() - before;
    const expectedMs = expectedMinutes * 60 * 1000;
    expect(delayMs).toBeGreaterThanOrEqual(expectedMs - 1000);
    expect(delayMs).toBeLessThanOrEqual(expectedMs + (after - before) + 1000);
  });
});

describe('canTransitionTo', () => {
  it('PENDING -> PROCESSING is valid', () => {
    expect(canTransitionTo('PENDING', 'PROCESSING')).toBe(true);
  });

  it('PROCESSING -> DELIVERED is valid', () => {
    expect(canTransitionTo('PROCESSING', 'DELIVERED')).toBe(true);
  });

  it('PROCESSING -> FAILED is valid', () => {
    expect(canTransitionTo('PROCESSING', 'FAILED')).toBe(true);
  });

  it('FAILED -> PENDING is valid', () => {
    expect(canTransitionTo('FAILED', 'PENDING')).toBe(true);
  });

  it('DELIVERED -> PENDING is invalid', () => {
    expect(canTransitionTo('DELIVERED', 'PENDING')).toBe(false);
  });

  it('DELIVERED -> FAILED is invalid', () => {
    expect(canTransitionTo('DELIVERED', 'FAILED')).toBe(false);
  });

  it('PENDING -> DELIVERED is invalid', () => {
    expect(canTransitionTo('PENDING', 'DELIVERED')).toBe(false);
  });

  it('PENDING -> FAILED is invalid', () => {
    expect(canTransitionTo('PENDING', 'FAILED')).toBe(false);
  });
});

describe('OutboxProcessor.processBatch', () => {
  it('processes successful messages', async () => {
    const msg = createOutboxMessage('topic', 'event.v1', '{"key":"value"}', 'corr-1');
    const store = new MockStore();
    store.messages = [msg];
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 10);
    const count = await processor.processBatch();
    expect(count).toBe(1);
    expect(publisher.published).toHaveLength(1);
    expect(publisher.published[0].topic).toBe('topic');

    // Processing -> Delivered
    expect(store.statusUpdates).toHaveLength(2);
    expect(store.statusUpdates[0].status).toBe('PROCESSING');
    expect(store.statusUpdates[1].status).toBe('DELIVERED');
  });

  it('updates to Failed when publish fails', async () => {
    const msg = createOutboxMessage('topic', 'event.v1', '{}', 'corr-1');
    const store = new MockStore();
    store.messages = [msg];
    const publisher = new MockPublisher();
    publisher.error = new Error('kafka unavailable');

    const processor = new OutboxProcessor(store, publisher, 10);
    const count = await processor.processBatch();
    expect(count).toBe(0);

    // Processing -> Failed (with retry)
    expect(store.statusUpdates).toHaveLength(2);
    expect(store.statusUpdates[0].status).toBe('PROCESSING');
    expect(store.statusUpdates[1].status).toBe('FAILED');
    expect(store.statusUpdates[1].retryCount).toBe(1);
  });

  it('throws when store getPendingMessages fails', async () => {
    const store = new MockStore();
    store.getErr = new Error('db error');
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 10);
    await expect(processor.processBatch()).rejects.toThrow('db error');
  });

  it('returns 0 when no messages', async () => {
    const store = new MockStore();
    store.messages = [];
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 10);
    const count = await processor.processBatch();
    expect(count).toBe(0);
  });

  it('respects batchSize', async () => {
    const store = new MockStore();
    for (let i = 0; i < 5; i++) {
      store.messages.push(createOutboxMessage('topic', 'event.v1', '{}', 'corr-1'));
    }
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 3);
    const count = await processor.processBatch();
    expect(count).toBe(3);
    expect(publisher.published).toHaveLength(3);
  });

  it('processes multiple messages', async () => {
    const store = new MockStore();
    store.messages = [
      createOutboxMessage('topic', 'event.v1', '{"n":1}', 'corr-1'),
      createOutboxMessage('topic', 'event.v1', '{"n":2}', 'corr-2'),
      createOutboxMessage('topic', 'event.v1', '{"n":3}', 'corr-3'),
    ];
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 10);
    const count = await processor.processBatch();
    expect(count).toBe(3);
    expect(publisher.published).toHaveLength(3);
    // 各メッセージが Processing -> Delivered の 2 回ずつ更新される
    expect(store.statusUpdates).toHaveLength(6);
  });

  it('uses default batchSize of 100 when 0 is provided', async () => {
    const store = new MockStore();
    for (let i = 0; i < 5; i++) {
      store.messages.push(createOutboxMessage('topic', 'event.v1', '{}', 'corr'));
    }
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 0);
    const count = await processor.processBatch();
    expect(count).toBe(5); // 全件処理される（100 > 5）
  });
});

describe('OutboxError', () => {
  it('has correct message with cause', () => {
    const cause = new Error('connection failed');
    const err = new OutboxError('SaveMessage', cause);
    expect(err.message).toContain('SaveMessage');
    expect(err.message).toContain('connection failed');
    expect(err.op).toBe('SaveMessage');
  });

  it('has correct message without cause', () => {
    const err = new OutboxError('SaveMessage');
    expect(err.message).toContain('SaveMessage');
    expect(err.name).toBe('OutboxError');
  });
});
