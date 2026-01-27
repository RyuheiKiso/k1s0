/**
 * TracingService のテスト
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { TracingService, SpanBuilder } from '../../src/tracing/TracingService';
import type { ObservabilityConfig, SpanInfo } from '../../src/types';

const createTestConfig = (): ObservabilityConfig => ({
  serviceName: 'test-service',
  env: 'dev',
  version: '1.0.0',
  samplingRate: 1.0,
  logLevel: 'DEBUG',
  enableConsole: false,
  enableBatching: false,
  batchSize: 10,
  batchIntervalMs: 5000,
});

describe('TracingService', () => {
  let tracingService: TracingService;

  beforeEach(() => {
    tracingService = new TracingService(createTestConfig());
  });

  afterEach(() => {
    tracingService.dispose();
  });

  describe('startContext', () => {
    it('新しいコンテキストを開始できること', () => {
      const context = tracingService.startContext();

      expect(context.traceId).toBeDefined();
      expect(context.spanId).toBeDefined();
      expect(context.traceId).toMatch(/^[0-9a-f]{32}$/);
      expect(context.spanId).toMatch(/^[0-9a-f]{16}$/);
    });

    it('リクエストIDを設定できること', () => {
      const context = tracingService.startContext('request-123');

      expect(context.requestId).toBe('request-123');
    });
  });

  describe('setContext', () => {
    it('traceparent からコンテキストを設定できること', () => {
      const traceId = 'a'.repeat(32);
      const spanId = 'b'.repeat(16);
      const traceparent = `00-${traceId}-${spanId}-01`;

      const context = tracingService.setContext(traceparent);

      expect(context).not.toBeNull();
      expect(context?.traceId).toBe(traceId);
      expect(context?.spanId).toBe(spanId);
    });

    it('無効な traceparent の場合 null を返すこと', () => {
      const context = tracingService.setContext('invalid');

      expect(context).toBeNull();
    });
  });

  describe('clearContext', () => {
    it('コンテキストをクリアできること', () => {
      tracingService.startContext();
      expect(tracingService.getCurrentContext()).not.toBeNull();

      tracingService.clearContext();
      expect(tracingService.getCurrentContext()).toBeNull();
    });
  });

  describe('startSpan', () => {
    it('新しいスパンを開始できること', () => {
      tracingService.startContext();
      const span = tracingService.startSpan('test-span');

      expect(span).toBeInstanceOf(SpanBuilder);
      expect(span.getSpanInfo().name).toBe('test-span');
    });

    it('親スパンIDが設定されること', () => {
      tracingService.startContext();
      const parentSpan = tracingService.startSpan('parent');
      const childSpan = tracingService.startSpan('child');

      expect(childSpan.getSpanInfo().parentSpanId).toBe(parentSpan.getSpanInfo().spanId);
    });

    it('同じトレースIDを共有すること', () => {
      const context = tracingService.startContext();
      const span = tracingService.startSpan('test');

      expect(span.getSpanInfo().traceId).toBe(context.traceId);
    });
  });

  describe('SpanBuilder', () => {
    it('属性を設定できること', () => {
      tracingService.startContext();
      const span = tracingService.startSpan('test');

      span.setAttribute('key1', 'value1');
      span.setAttributes({ key2: 123, key3: true });

      const info = span.getSpanInfo();
      expect(info.attributes['key1']).toBe('value1');
      expect(info.attributes['key2']).toBe(123);
      expect(info.attributes['key3']).toBe(true);
    });

    it('スパンを正常終了できること', () => {
      tracingService.startContext();
      const span = tracingService.startSpan('test');

      const info = span.end();

      expect(info.endTime).toBeDefined();
      expect(info.status?.code).toBe('OK');
    });

    it('スパンをエラーで終了できること', () => {
      tracingService.startContext();
      const span = tracingService.startSpan('test');

      const error = new Error('Test error');
      const info = span.endWithError(error, 'TEST_ERROR');

      expect(info.status?.code).toBe('ERROR');
      expect(info.attributes['error.name']).toBe('Error');
      expect(info.attributes['error.message']).toBe('Test error');
      expect(info.attributes['error.code']).toBe('TEST_ERROR');
    });
  });

  describe('withSpan', () => {
    it('関数を実行してスパンを記録できること', async () => {
      tracingService.startContext();

      const result = await tracingService.withSpan('test-operation', async (span) => {
        span.setAttribute('input', 'test');
        return 'success';
      });

      expect(result).toBe('success');
    });

    it('エラー時にスパンをエラーで終了すること', async () => {
      tracingService.startContext();

      const listener = vi.fn();
      tracingService.onSpan(listener);

      await expect(
        tracingService.withSpan('failing-operation', async () => {
          throw new Error('Operation failed');
        })
      ).rejects.toThrow('Operation failed');

      expect(listener).toHaveBeenCalled();
      const recordedSpan = listener.mock.calls[0][0] as SpanInfo;
      expect(recordedSpan.status?.code).toBe('ERROR');
    });
  });

  describe('getCurrentTraceId', () => {
    it('現在のトレースIDを取得できること', () => {
      const context = tracingService.startContext();

      expect(tracingService.getCurrentTraceId()).toBe(context.traceId);
    });

    it('コンテキストがない場合 undefined を返すこと', () => {
      expect(tracingService.getCurrentTraceId()).toBeUndefined();
    });
  });

  describe('getCurrentSpanId', () => {
    it('現在のスパンIDを取得できること', () => {
      tracingService.startContext();
      const span = tracingService.startSpan('test');

      expect(tracingService.getCurrentSpanId()).toBe(span.getSpanInfo().spanId);
    });
  });

  describe('getTraceparent', () => {
    it('traceparent ヘッダー値を生成できること', () => {
      tracingService.startContext();

      const traceparent = tracingService.getTraceparent();

      expect(traceparent).toMatch(/^00-[0-9a-f]{32}-[0-9a-f]{16}-0[01]$/);
    });

    it('コンテキストがない場合 null を返すこと', () => {
      expect(tracingService.getTraceparent()).toBeNull();
    });
  });

  describe('recordSpan', () => {
    it('スパンをバッファに追加すること', () => {
      tracingService.startContext();
      const span = tracingService.startSpan('test');
      span.end();

      // flush でバッファからスパンを取得
      const spans = tracingService.flushSpans();
      expect(spans.length).toBeGreaterThan(0);
    });

    it('リスナーに通知すること', () => {
      tracingService.startContext();
      const listener = vi.fn();
      tracingService.onSpan(listener);

      const span = tracingService.startSpan('test');
      span.end();

      expect(listener).toHaveBeenCalled();
    });
  });

  describe('onSpan', () => {
    it('リスナーを登録できること', () => {
      const listener = vi.fn();
      tracingService.onSpan(listener);

      tracingService.startContext();
      const span = tracingService.startSpan('test');
      span.end();

      expect(listener).toHaveBeenCalled();
    });

    it('リスナーを解除できること', () => {
      const listener = vi.fn();
      const unsubscribe = tracingService.onSpan(listener);
      unsubscribe();

      tracingService.startContext();
      const span = tracingService.startSpan('test');
      span.end();

      expect(listener).not.toHaveBeenCalled();
    });
  });

  describe('flushSpans', () => {
    it('バッファ内のスパンを返すこと', async () => {
      tracingService.startContext();
      const span1 = tracingService.startSpan('span1');
      span1.end();
      const span2 = tracingService.startSpan('span2');
      span2.end();

      const spans = await tracingService.flushSpans();

      expect(spans.length).toBe(2);
    });

    it('フラッシュ後はバッファが空になること', async () => {
      tracingService.startContext();
      const span = tracingService.startSpan('test');
      span.end();

      await tracingService.flushSpans();
      const spans = await tracingService.flushSpans();

      expect(spans.length).toBe(0);
    });
  });

  describe('サンプリング', () => {
    it('サンプリングレートが 0 の場合、スパンが記録されないこと', () => {
      const configWithNoSampling = {
        ...createTestConfig(),
        samplingRate: 0,
      };
      const service = new TracingService(configWithNoSampling);
      const listener = vi.fn();
      service.onSpan(listener);

      service.startContext();
      const span = service.startSpan('test');
      span.end();

      expect(listener).not.toHaveBeenCalled();

      service.dispose();
    });
  });

  describe('dispose', () => {
    it('リソースを解放すること', () => {
      tracingService.startContext();
      const span = tracingService.startSpan('test');
      span.end();

      tracingService.dispose();

      // dispose 後はコンテキストがクリアされている
      expect(tracingService.getCurrentContext()).toBeNull();
    });
  });
});
