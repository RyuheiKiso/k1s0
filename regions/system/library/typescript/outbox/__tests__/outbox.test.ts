import { describe, it, expect } from 'vitest';
import {
  OutboxMessage,
  OutboxStatus,
  OutboxErrorCode,
  createOutboxMessage,
  markProcessing,
  markDelivered,
  markFailed,
  isProcessable,
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
  updatedMessages: OutboxMessage[] = [];
  deletedCount = 0;
  saveErr?: Error;
  fetchErr?: Error;
  updateErr?: Error;

  async save(msg: OutboxMessage): Promise<void> {
    if (this.saveErr) throw this.saveErr;
    this.savedMessages.push({ ...msg });
  }

  async fetchPending(limit: number): Promise<OutboxMessage[]> {
    if (this.fetchErr) throw this.fetchErr;
    return this.messages.slice(0, limit);
  }

  async update(msg: OutboxMessage): Promise<void> {
    if (this.updateErr) throw this.updateErr;
    this.updatedMessages.push({ ...msg });
  }

  async deleteDelivered(olderThanDays: number): Promise<number> {
    return this.deletedCount;
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
    const msg = createOutboxMessage('k1s0.system.user.created.v1', 'user-001', '{"id":"1"}');
    expect(msg.id).toMatch(UUID_V4_REGEX);
    expect(msg.topic).toBe('k1s0.system.user.created.v1');
    expect(msg.partitionKey).toBe('user-001');
    expect(msg.payload).toBe('{"id":"1"}');
    expect(msg.status).toBe('PENDING');
    expect(msg.retryCount).toBe(0);
    expect(msg.maxRetries).toBe(3);
    expect(msg.lastError).toBeNull();
    expect(msg.createdAt).toBeInstanceOf(Date);
    expect(msg.processAfter).toBeInstanceOf(Date);
  });

  it('generates unique IDs', () => {
    const msg1 = createOutboxMessage('topic', 'key', '{}');
    const msg2 = createOutboxMessage('topic', 'key', '{}');
    expect(msg1.id).not.toBe(msg2.id);
  });
});

describe('markProcessing', () => {
  it('sets status to PROCESSING', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    markProcessing(msg);
    expect(msg.status).toBe('PROCESSING');
  });
});

describe('markDelivered', () => {
  it('sets status to DELIVERED', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    markProcessing(msg);
    markDelivered(msg);
    expect(msg.status).toBe('DELIVERED');
    expect(isProcessable(msg)).toBe(false);
  });
});

describe('markFailed', () => {
  it('increments retryCount and sets FAILED status', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    markFailed(msg, 'kafka error');
    expect(msg.retryCount).toBe(1);
    expect(msg.status).toBe('FAILED');
    expect(msg.lastError).toBe('kafka error');
  });

  it('sets DEAD_LETTER on max retries', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    msg.maxRetries = 3;
    markFailed(msg, 'error 1');
    markFailed(msg, 'error 2');
    markFailed(msg, 'error 3');
    expect(msg.status).toBe('DEAD_LETTER');
    expect(msg.retryCount).toBe(3);
  });

  it('uses exponential backoff in seconds', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    const before = Date.now();
    markFailed(msg, 'error');
    // retryCount is now 1, so delay = 2^1 = 2 seconds
    const expectedDelay = 2 * 1000;
    const actualDelay = msg.processAfter.getTime() - before;
    expect(actualDelay).toBeGreaterThanOrEqual(expectedDelay - 100);
    expect(actualDelay).toBeLessThanOrEqual(expectedDelay + 1000);
  });
});

describe('isProcessable', () => {
  it('returns true for PENDING with processAfter in past', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    expect(isProcessable(msg)).toBe(true);
  });

  it('returns true for FAILED with processAfter in past', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    msg.status = 'FAILED';
    msg.processAfter = new Date(Date.now() - 1000);
    expect(isProcessable(msg)).toBe(true);
  });

  it('returns false for DELIVERED', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    msg.status = 'DELIVERED';
    expect(isProcessable(msg)).toBe(false);
  });

  it('returns false for DEAD_LETTER', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    msg.status = 'DEAD_LETTER';
    expect(isProcessable(msg)).toBe(false);
  });

  it('returns false when processAfter is in the future', () => {
    const msg = createOutboxMessage('topic', 'key', '{}');
    msg.processAfter = new Date(Date.now() + 60000);
    expect(isProcessable(msg)).toBe(false);
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

  it('PROCESSING -> DEAD_LETTER is valid', () => {
    expect(canTransitionTo('PROCESSING', 'DEAD_LETTER')).toBe(true);
  });

  it('FAILED -> PROCESSING is valid', () => {
    expect(canTransitionTo('FAILED', 'PROCESSING')).toBe(true);
  });

  it('DELIVERED -> PENDING is invalid', () => {
    expect(canTransitionTo('DELIVERED', 'PENDING')).toBe(false);
  });

  it('DELIVERED -> FAILED is invalid', () => {
    expect(canTransitionTo('DELIVERED', 'FAILED')).toBe(false);
  });

  it('DEAD_LETTER -> PENDING is invalid', () => {
    expect(canTransitionTo('DEAD_LETTER', 'PENDING')).toBe(false);
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
    const msg = createOutboxMessage('topic', 'key-1', '{"key":"value"}');
    const store = new MockStore();
    store.messages = [msg];
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 10);
    const count = await processor.processBatch();
    expect(count).toBe(1);
    expect(publisher.published).toHaveLength(1);
    expect(publisher.published[0].topic).toBe('topic');

    // Processing -> Delivered (2 updates)
    expect(store.updatedMessages).toHaveLength(2);
    expect(store.updatedMessages[0].status).toBe('PROCESSING');
    expect(store.updatedMessages[1].status).toBe('DELIVERED');
  });

  it('updates to Failed when publish fails', async () => {
    const msg = createOutboxMessage('topic', 'key-1', '{}');
    const store = new MockStore();
    store.messages = [msg];
    const publisher = new MockPublisher();
    publisher.error = new Error('kafka unavailable');

    const processor = new OutboxProcessor(store, publisher, 10);
    const count = await processor.processBatch();
    expect(count).toBe(0);

    // Processing -> Failed (2 updates)
    expect(store.updatedMessages).toHaveLength(2);
    expect(store.updatedMessages[0].status).toBe('PROCESSING');
    expect(store.updatedMessages[1].status).toBe('FAILED');
    expect(store.updatedMessages[1].retryCount).toBe(1);
    expect(store.updatedMessages[1].lastError).toBe('kafka unavailable');
  });

  it('throws when store fetchPending fails', async () => {
    const store = new MockStore();
    store.fetchErr = new Error('db error');
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
      store.messages.push(createOutboxMessage('topic', `key-${i}`, '{}'));
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
      createOutboxMessage('topic', 'key-1', '{"n":1}'),
      createOutboxMessage('topic', 'key-2', '{"n":2}'),
      createOutboxMessage('topic', 'key-3', '{"n":3}'),
    ];
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 10);
    const count = await processor.processBatch();
    expect(count).toBe(3);
    expect(publisher.published).toHaveLength(3);
    // 各メッセージが Processing -> Delivered の 2 回ずつ更新される
    expect(store.updatedMessages).toHaveLength(6);
  });

  it('marks DEAD_LETTER after max retries exceeded', async () => {
    const msg = createOutboxMessage('topic', 'key-1', '{}');
    msg.maxRetries = 1;
    const store = new MockStore();
    store.messages = [msg];
    const publisher = new MockPublisher();
    publisher.error = new Error('always fail');

    const processor = new OutboxProcessor(store, publisher, 10);
    const count = await processor.processBatch();
    expect(count).toBe(0);

    // Processing update + DEAD_LETTER update
    expect(store.updatedMessages).toHaveLength(2);
    expect(store.updatedMessages[1].status).toBe('DEAD_LETTER');
    expect(store.updatedMessages[1].retryCount).toBe(1);
  });

  it('uses default batchSize of 100 when 0 is provided', async () => {
    const store = new MockStore();
    for (let i = 0; i < 5; i++) {
      store.messages.push(createOutboxMessage('topic', `key-${i}`, '{}'));
    }
    const publisher = new MockPublisher();

    const processor = new OutboxProcessor(store, publisher, 0);
    const count = await processor.processBatch();
    expect(count).toBe(5); // 全件処理される（100 > 5）
  });
});

describe('OutboxStore interface', () => {
  it('save stores message', async () => {
    const store = new MockStore();
    const msg = createOutboxMessage('topic', 'key', '{}');
    await store.save(msg);
    expect(store.savedMessages).toHaveLength(1);
  });

  it('deleteDelivered returns count', async () => {
    const store = new MockStore();
    store.deletedCount = 5;
    const count = await store.deleteDelivered(30);
    expect(count).toBe(5);
  });
});

describe('OutboxError', () => {
  it('has correct message with detail', () => {
    const err = new OutboxError('STORE_ERROR', 'connection failed');
    expect(err.message).toContain('STORE_ERROR');
    expect(err.message).toContain('connection failed');
    expect(err.code).toBe('STORE_ERROR');
  });

  it('has correct message without detail', () => {
    const err = new OutboxError('PUBLISH_ERROR');
    expect(err.message).toContain('PUBLISH_ERROR');
    expect(err.name).toBe('OutboxError');
  });

  it('supports all error codes', () => {
    const codes: OutboxErrorCode[] = ['STORE_ERROR', 'PUBLISH_ERROR', 'SERIALIZATION_ERROR', 'NOT_FOUND'];
    for (const code of codes) {
      const err = new OutboxError(code, 'test');
      expect(err.code).toBe(code);
    }
  });
});
