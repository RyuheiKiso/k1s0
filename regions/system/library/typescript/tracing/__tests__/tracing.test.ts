import { describe, it, expect } from 'vitest';
import {
  toTraceparent,
  fromTraceparent,
  Baggage,
  injectContext,
  extractContext,
} from '../src/index.js';
import type { TraceContext } from '../src/index.js';

describe('TraceparentRoundtrip', () => {
  it('traceparent をエンコード・デコードできる', () => {
    const ctx: TraceContext = {
      traceId: '0af7651916cd43dd8448eb211c80319c',
      parentId: 'b7ad6b7169203331',
      flags: 0x01,
    };

    const header = toTraceparent(ctx);
    expect(header).toBe('00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01');

    const parsed = fromTraceparent(header);
    expect(parsed).not.toBeNull();
    expect(parsed!.traceId).toBe(ctx.traceId);
    expect(parsed!.parentId).toBe(ctx.parentId);
    expect(parsed!.flags).toBe(ctx.flags);
  });

  it('不正な traceparent は null を返す', () => {
    expect(fromTraceparent('invalid')).toBeNull();
    expect(fromTraceparent('01-abc-def-00')).toBeNull();
  });
});

describe('BaggageRoundtrip', () => {
  it('baggage をエンコード・デコードできる', () => {
    const bag = new Baggage();
    bag.set('userId', 'alice');
    bag.set('tenantId', 't-1');

    const header = bag.toHeader();
    expect(header).toContain('userId=alice');
    expect(header).toContain('tenantId=t-1');

    const parsed = Baggage.fromHeader(header);
    expect(parsed.get('userId')).toBe('alice');
    expect(parsed.get('tenantId')).toBe('t-1');
  });

  it('空ヘッダーから空の Baggage が返る', () => {
    const bag = Baggage.fromHeader('');
    expect(bag.get('any')).toBeUndefined();
  });
});

describe('InjectExtract', () => {
  it('ヘッダーに注入・抽出できる', () => {
    const ctx: TraceContext = {
      traceId: '0af7651916cd43dd8448eb211c80319c',
      parentId: 'b7ad6b7169203331',
      flags: 0x01,
    };
    const bag = new Baggage();
    bag.set('requestId', 'req-123');

    const headers: Record<string, string> = {};
    injectContext(headers, ctx, bag);

    expect(headers['traceparent']).toBeDefined();
    expect(headers['baggage']).toBeDefined();

    const extracted = extractContext(headers);
    expect(extracted.context).not.toBeNull();
    expect(extracted.context!.traceId).toBe(ctx.traceId);
    expect(extracted.context!.parentId).toBe(ctx.parentId);
    expect(extracted.context!.flags).toBe(ctx.flags);
    expect(extracted.baggage.get('requestId')).toBe('req-123');
  });

  it('空ヘッダーからは null context と空 baggage が返る', () => {
    const extracted = extractContext({});
    expect(extracted.context).toBeNull();
    expect(extracted.baggage.get('any')).toBeUndefined();
  });
});
