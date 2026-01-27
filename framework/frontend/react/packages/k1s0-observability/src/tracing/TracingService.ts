import type {
  ObservabilityConfig,
  SpanInfo,
  SpanStatus,
  ObservabilityContext,
} from '../types.js';
import {
  generateTraceId,
  generateSpanId,
  generateTraceparent,
  parseTraceparent,
} from '../utils/ids.js';

/**
 * OpenTelemetry API の Tracer インターフェース（optional dependency）
 */
interface OTelTracer {
  startSpan(name: string, options?: unknown): OTelSpan;
}

interface OTelSpan {
  setAttribute(key: string, value: string | number | boolean): void;
  setStatus(status: { code: number; message?: string }): void;
  recordException(exception: Error): void;
  end(): void;
}

interface OTelApi {
  trace: {
    getTracer(name: string, version?: string): OTelTracer;
    setSpan(context: unknown, span: OTelSpan): unknown;
  };
  context: {
    active(): unknown;
    with<T>(ctx: unknown, fn: () => T): T;
  };
}

/**
 * スパンビルダー
 */
export class SpanBuilder {
  private span: SpanInfo;
  private tracingService: TracingService;

  constructor(tracingService: TracingService, name: string, parentSpanId?: string) {
    const traceId = tracingService.getCurrentTraceId() ?? generateTraceId();

    this.tracingService = tracingService;
    this.span = {
      traceId,
      spanId: generateSpanId(),
      parentSpanId,
      name,
      startTime: performance.now(),
      attributes: {},
    };
  }

  /**
   * 属性を設定
   */
  setAttribute(key: string, value: string | number | boolean): this {
    this.span.attributes[key] = value;
    return this;
  }

  /**
   * 複数の属性を設定
   */
  setAttributes(attributes: Record<string, string | number | boolean>): this {
    Object.assign(this.span.attributes, attributes);
    return this;
  }

  /**
   * スパンを終了
   */
  end(status?: SpanStatus): SpanInfo {
    this.span.endTime = performance.now();
    this.span.status = status ?? { code: 'OK' };

    this.tracingService.recordSpan(this.span);

    return this.span;
  }

  /**
   * エラーでスパンを終了
   */
  endWithError(error: Error, errorCode?: string): SpanInfo {
    this.span.attributes['error.name'] = error.name;
    this.span.attributes['error.message'] = error.message;
    if (errorCode) {
      this.span.attributes['error.code'] = errorCode;
    }

    return this.end({ code: 'ERROR', message: error.message });
  }

  /**
   * 現在のスパン情報を取得
   */
  getSpanInfo(): SpanInfo {
    return this.span;
  }
}

/**
 * トレーシングサービス
 *
 * - OpenTelemetry 統合（設定時）
 * - フォールバック実装（OTel なしでも動作）
 * - trace_id/span_id の自動付与
 * - W3C Trace Context サポート
 */
export class TracingService {
  private config: ObservabilityConfig;
  private otelApi: OTelApi | null = null;
  private otelTracer: OTelTracer | null = null;

  // 現在のコンテキスト
  private currentContext: ObservabilityContext | null = null;

  // スパンスタック（ネストしたスパン用）
  private spanStack: SpanInfo[] = [];

  // 完了したスパンのバッファ
  private spanBuffer: SpanInfo[] = [];

  // リスナー
  private spanListeners: Set<(span: SpanInfo) => void> = new Set();

  constructor(config: ObservabilityConfig) {
    this.config = config;
  }

  /**
   * OpenTelemetry を設定
   */
  setOpenTelemetry(otelApi: OTelApi): void {
    this.otelApi = otelApi;
    this.otelTracer = otelApi.trace.getTracer(
      this.config.serviceName,
      this.config.version
    );
  }

  /**
   * 新しいスパンを開始
   */
  startSpan(name: string): SpanBuilder {
    const parentSpanId = this.getCurrentSpanId();
    const builder = new SpanBuilder(this, name, parentSpanId);

    // スタックに追加
    this.spanStack.push(builder.getSpanInfo());

    return builder;
  }

  /**
   * 関数を実行してスパンを記録
   */
  async withSpan<T>(
    name: string,
    fn: (span: SpanBuilder) => T | Promise<T>
  ): Promise<T> {
    const span = this.startSpan(name);

    try {
      const result = await fn(span);
      span.end({ code: 'OK' });
      return result;
    } catch (error) {
      span.endWithError(error instanceof Error ? error : new Error(String(error)));
      throw error;
    }
  }

  /**
   * 現在のトレースIDを取得
   */
  getCurrentTraceId(): string | undefined {
    return this.currentContext?.traceId ?? this.spanStack[0]?.traceId;
  }

  /**
   * 現在のスパンIDを取得
   */
  getCurrentSpanId(): string | undefined {
    if (this.spanStack.length > 0) {
      return this.spanStack[this.spanStack.length - 1].spanId;
    }
    return this.currentContext?.spanId;
  }

  /**
   * 現在のコンテキストを取得
   */
  getCurrentContext(): ObservabilityContext | null {
    if (this.currentContext) {
      return this.currentContext;
    }

    const traceId = this.getCurrentTraceId();
    const spanId = this.getCurrentSpanId();

    if (traceId && spanId) {
      return { traceId, spanId };
    }

    return null;
  }

  /**
   * 新しいコンテキストを開始
   */
  startContext(requestId?: string): ObservabilityContext {
    this.currentContext = {
      traceId: generateTraceId(),
      spanId: generateSpanId(),
      requestId,
    };
    return this.currentContext;
  }

  /**
   * コンテキストを設定（受信した traceparent から）
   */
  setContext(traceparent: string, requestId?: string): ObservabilityContext | null {
    const parsed = parseTraceparent(traceparent);
    if (!parsed) return null;

    this.currentContext = {
      traceId: parsed.traceId,
      spanId: parsed.spanId,
      requestId,
    };

    return this.currentContext;
  }

  /**
   * コンテキストをクリア
   */
  clearContext(): void {
    this.currentContext = null;
    this.spanStack = [];
  }

  /**
   * traceparent ヘッダー値を生成
   */
  getTraceparent(): string | null {
    const ctx = this.getCurrentContext();
    if (!ctx) return null;

    const sampled = Math.random() < this.config.samplingRate;
    return generateTraceparent(ctx.traceId, ctx.spanId, sampled);
  }

  /**
   * スパンを記録
   */
  recordSpan(span: SpanInfo): void {
    // スタックから削除
    const index = this.spanStack.findIndex((s) => s.spanId === span.spanId);
    if (index !== -1) {
      this.spanStack.splice(index, 1);
    }

    // サンプリング
    if (Math.random() >= this.config.samplingRate) {
      return;
    }

    // バッファに追加
    this.spanBuffer.push(span);

    // リスナーに通知
    for (const listener of this.spanListeners) {
      try {
        listener(span);
      } catch {
        // リスナーエラーは無視
      }
    }

    // バッチサイズに達したらフラッシュ
    if (this.config.enableBatching && this.spanBuffer.length >= this.config.batchSize) {
      this.flushSpans();
    }
  }

  /**
   * スパンリスナーを追加
   */
  onSpan(listener: (span: SpanInfo) => void): () => void {
    this.spanListeners.add(listener);
    return () => {
      this.spanListeners.delete(listener);
    };
  }

  /**
   * バッファ内のスパンをフラッシュ
   */
  async flushSpans(): Promise<SpanInfo[]> {
    const spans = [...this.spanBuffer];
    this.spanBuffer = [];
    return spans;
  }

  /**
   * リソースを解放
   */
  dispose(): void {
    this.spanListeners.clear();
    this.spanBuffer = [];
    this.spanStack = [];
    this.currentContext = null;
  }
}
