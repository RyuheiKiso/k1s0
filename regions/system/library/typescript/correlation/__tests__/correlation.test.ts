import { describe, it, expect } from 'vitest';
import {
  CorrelationId,
  TraceId,
  newCorrelationContext,
  toHeaders,
  fromHeaders,
  HEADER_CORRELATION_ID,
  HEADER_TRACE_ID,
} from '../src/index.js';

describe('CorrelationId', () => {
  it('generates a non-empty id', () => {
    const id = CorrelationId.generate();
    expect(id.toString()).not.toBe('');
    expect(id.isEmpty()).toBe(false);
  });

  it('generates unique ids', () => {
    const id1 = CorrelationId.generate();
    const id2 = CorrelationId.generate();
    expect(id1.toString()).not.toBe(id2.toString());
  });

  it('parses any string without validation', () => {
    const id = CorrelationId.parse('custom-id-123');
    expect(id.toString()).toBe('custom-id-123');
  });

  it('isEmpty returns true for empty string', () => {
    const id = CorrelationId.parse('');
    expect(id.isEmpty()).toBe(true);
  });

  it('isEmpty returns false for non-empty string', () => {
    const id = CorrelationId.parse('some-id');
    expect(id.isEmpty()).toBe(false);
  });
});

describe('TraceId', () => {
  it('generates a 32-character id', () => {
    const id = TraceId.generate();
    expect(id.toString()).toHaveLength(32);
  });

  it('generates lowercase hex only', () => {
    const id = TraceId.generate();
    expect(id.toString()).toMatch(/^[0-9a-f]{32}$/);
  });

  it('generates unique ids', () => {
    const id1 = TraceId.generate();
    const id2 = TraceId.generate();
    expect(id1.toString()).not.toBe(id2.toString());
  });

  it('parses valid 32-char lowercase hex', () => {
    const valid = '4bf92f3577b34da6a3ce929d0e0e4736';
    const id = TraceId.parse(valid);
    expect(id.toString()).toBe(valid);
  });

  it('throws on wrong length', () => {
    expect(() => TraceId.parse('short')).toThrow();
  });

  it('throws on uppercase hex', () => {
    expect(() => TraceId.parse('4BF92F3577B34DA6A3CE929D0E0E4736')).toThrow();
  });
});

describe('newCorrelationContext', () => {
  it('creates context with non-empty ids', () => {
    const ctx = newCorrelationContext();
    expect(ctx.correlationId.isEmpty()).toBe(false);
    expect(ctx.traceId.isEmpty()).toBe(false);
  });
});

describe('toHeaders', () => {
  it('converts context to header map', () => {
    const ctx = {
      correlationId: CorrelationId.parse('test-id'),
      traceId: TraceId.parse('4bf92f3577b34da6a3ce929d0e0e4736'),
    };
    const headers = toHeaders(ctx);
    expect(headers[HEADER_CORRELATION_ID]).toBe('test-id');
    expect(headers[HEADER_TRACE_ID]).toBe('4bf92f3577b34da6a3ce929d0e0e4736');
  });

  it('omits empty ids from headers', () => {
    const ctx = {
      correlationId: CorrelationId.parse(''),
      traceId: TraceId.generate(),
    };
    const headers = toHeaders(ctx);
    expect(headers[HEADER_CORRELATION_ID]).toBeUndefined();
  });
});

describe('fromHeaders', () => {
  it('extracts existing headers', () => {
    const headers = {
      [HEADER_CORRELATION_ID]: 'existing-id',
      [HEADER_TRACE_ID]: '4bf92f3577b34da6a3ce929d0e0e4736',
    };
    const ctx = fromHeaders(headers);
    expect(ctx.correlationId.toString()).toBe('existing-id');
    expect(ctx.traceId.toString()).toBe('4bf92f3577b34da6a3ce929d0e0e4736');
  });

  it('auto-generates missing correlation id', () => {
    const ctx = fromHeaders({});
    expect(ctx.correlationId.isEmpty()).toBe(false);
  });

  it('auto-generates missing trace id', () => {
    const ctx = fromHeaders({});
    expect(ctx.traceId.isEmpty()).toBe(false);
    expect(ctx.traceId.toString()).toHaveLength(32);
  });

  it('auto-generates trace id when invalid', () => {
    const headers = { [HEADER_TRACE_ID]: 'invalid' };
    const ctx = fromHeaders(headers);
    expect(ctx.traceId.toString()).toHaveLength(32);
    expect(ctx.traceId.toString()).toMatch(/^[0-9a-f]{32}$/);
  });
});
