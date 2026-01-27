/**
 * ID生成ユーティリティのテスト
 */

import { describe, it, expect } from 'vitest';
import {
  generateTraceId,
  generateSpanId,
  generateRequestId,
  generateTimestamp,
  generateTraceparent,
  parseTraceparent,
} from '../../src/utils/ids';

describe('generateTraceId', () => {
  it('32文字の16進数を生成すること', () => {
    const traceId = generateTraceId();

    expect(traceId).toMatch(/^[0-9a-f]{32}$/);
  });

  it('毎回異なる値を生成すること', () => {
    const id1 = generateTraceId();
    const id2 = generateTraceId();

    expect(id1).not.toBe(id2);
  });
});

describe('generateSpanId', () => {
  it('16文字の16進数を生成すること', () => {
    const spanId = generateSpanId();

    expect(spanId).toMatch(/^[0-9a-f]{16}$/);
  });

  it('毎回異なる値を生成すること', () => {
    const id1 = generateSpanId();
    const id2 = generateSpanId();

    expect(id1).not.toBe(id2);
  });
});

describe('generateRequestId', () => {
  it('UUID v4 形式の文字列を生成すること', () => {
    const requestId = generateRequestId();

    // UUID v4 の形式: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    expect(requestId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/
    );
  });

  it('毎回異なる値を生成すること', () => {
    const id1 = generateRequestId();
    const id2 = generateRequestId();

    expect(id1).not.toBe(id2);
  });
});

describe('generateTimestamp', () => {
  it('ISO 8601 形式のタイムスタンプを生成すること', () => {
    const timestamp = generateTimestamp();

    // ISO 8601 形式: 2024-01-15T12:34:56.789Z
    expect(timestamp).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/);
  });

  it('現在時刻に近い値を生成すること', () => {
    const before = new Date().toISOString();
    const timestamp = generateTimestamp();
    const after = new Date().toISOString();

    expect(timestamp >= before).toBe(true);
    expect(timestamp <= after).toBe(true);
  });
});

describe('generateTraceparent', () => {
  it('W3C Trace Context 形式の traceparent を生成すること', () => {
    const traceId = 'a'.repeat(32);
    const spanId = 'b'.repeat(16);

    const traceparent = generateTraceparent(traceId, spanId, true);

    expect(traceparent).toBe(`00-${traceId}-${spanId}-01`);
  });

  it('sampled が false の場合、flags が 00 になること', () => {
    const traceId = 'a'.repeat(32);
    const spanId = 'b'.repeat(16);

    const traceparent = generateTraceparent(traceId, spanId, false);

    expect(traceparent).toBe(`00-${traceId}-${spanId}-00`);
  });

  it('デフォルトで sampled が true になること', () => {
    const traceId = 'a'.repeat(32);
    const spanId = 'b'.repeat(16);

    const traceparent = generateTraceparent(traceId, spanId);

    expect(traceparent).toMatch(/-01$/);
  });
});

describe('parseTraceparent', () => {
  it('有効な traceparent をパースできること', () => {
    const traceId = 'a'.repeat(32);
    const spanId = 'b'.repeat(16);
    const traceparent = `00-${traceId}-${spanId}-01`;

    const result = parseTraceparent(traceparent);

    expect(result).toEqual({
      traceId,
      spanId,
      sampled: true,
    });
  });

  it('sampled フラグを正しくパースすること', () => {
    const traceId = 'a'.repeat(32);
    const spanId = 'b'.repeat(16);

    const sampledResult = parseTraceparent(`00-${traceId}-${spanId}-01`);
    expect(sampledResult?.sampled).toBe(true);

    const notSampledResult = parseTraceparent(`00-${traceId}-${spanId}-00`);
    expect(notSampledResult?.sampled).toBe(false);
  });

  it('無効な形式の場合 null を返すこと', () => {
    expect(parseTraceparent('invalid')).toBeNull();
    expect(parseTraceparent('00-invalid-format')).toBeNull();
    expect(parseTraceparent('')).toBeNull();
  });

  it('version が 00 でない場合 null を返すこと', () => {
    const traceId = 'a'.repeat(32);
    const spanId = 'b'.repeat(16);

    const result = parseTraceparent(`01-${traceId}-${spanId}-01`);

    expect(result).toBeNull();
  });

  it('trace_id の長さが不正な場合 null を返すこと', () => {
    const shortTraceId = 'a'.repeat(31); // 31文字
    const spanId = 'b'.repeat(16);

    const result = parseTraceparent(`00-${shortTraceId}-${spanId}-01`);

    expect(result).toBeNull();
  });

  it('span_id の長さが不正な場合 null を返すこと', () => {
    const traceId = 'a'.repeat(32);
    const shortSpanId = 'b'.repeat(15); // 15文字

    const result = parseTraceparent(`00-${traceId}-${shortSpanId}-01`);

    expect(result).toBeNull();
  });
});
