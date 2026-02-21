import { describe, it, expect } from 'vitest';
import {
  createEventMetadata,
  EventEnvelope,
  NoOpEventProducer,
  MessagingError,
} from '../src/index.js';

const UUID_V4_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;

describe('createEventMetadata', () => {
  it('generates a UUID v4 format eventId', () => {
    const meta = createEventMetadata('user.created.v1', 'auth-service');
    expect(meta.eventId).toMatch(UUID_V4_REGEX);
  });

  it('generates an ISO timestamp', () => {
    const meta = createEventMetadata('user.created.v1', 'auth-service');
    expect(new Date(meta.timestamp).toISOString()).toBe(meta.timestamp);
  });

  it('uses provided correlationId and traceId', () => {
    const meta = createEventMetadata('user.created.v1', 'auth-service', 'corr-123', 'trace-456');
    expect(meta.correlationId).toBe('corr-123');
    expect(meta.traceId).toBe('trace-456');
  });

  it('auto-generates correlationId and traceId when not provided', () => {
    const meta = createEventMetadata('user.created.v1', 'auth-service');
    expect(meta.correlationId).toMatch(UUID_V4_REGEX);
    expect(meta.traceId).toMatch(UUID_V4_REGEX);
  });

  it('sets eventType and source correctly', () => {
    const meta = createEventMetadata('user.created.v1', 'auth-service');
    expect(meta.eventType).toBe('user.created.v1');
    expect(meta.source).toBe('auth-service');
  });

  it('generates unique eventIds', () => {
    const meta1 = createEventMetadata('event.v1', 'svc');
    const meta2 = createEventMetadata('event.v1', 'svc');
    expect(meta1.eventId).not.toBe(meta2.eventId);
  });
});

describe('EventEnvelope', () => {
  it('can be constructed correctly', () => {
    const envelope: EventEnvelope = {
      topic: 'k1s0.system.user.created.v1',
      payload: { key: 'value' },
      metadata: createEventMetadata('user.created.v1', 'auth-service', 'corr-1'),
    };
    expect(envelope.topic).toBe('k1s0.system.user.created.v1');
    expect(envelope.payload).toEqual({ key: 'value' });
    expect(envelope.metadata.eventType).toBe('user.created.v1');
  });
});

describe('NoOpEventProducer', () => {
  it('adds events to published on publish', async () => {
    const producer = new NoOpEventProducer();
    const event: EventEnvelope = {
      topic: 'k1s0.system.test.event.v1',
      payload: 'test-payload',
      metadata: createEventMetadata('test.v1', 'svc', 'corr-1'),
    };
    await producer.publish(event);
    expect(producer.published).toHaveLength(1);
    expect(producer.published[0].topic).toBe('k1s0.system.test.event.v1');
  });

  it('close resolves without error', async () => {
    const producer = new NoOpEventProducer();
    await expect(producer.close()).resolves.toBeUndefined();
  });
});

describe('MessagingError', () => {
  it('has correct message', () => {
    const cause = new Error('connection refused');
    const err = new MessagingError('Publish', cause);
    expect(err.message).toBe('Publish: connection refused');
    expect(err.op).toBe('Publish');
  });

  it('retains the original cause', () => {
    const cause = new Error('connection refused');
    const err = new MessagingError('Publish', cause);
    expect(err.cause).toBe(cause);
  });

  it('works without cause', () => {
    const err = new MessagingError('Publish');
    expect(err.message).toBe('Publish');
    expect(err.cause).toBeUndefined();
  });
});
