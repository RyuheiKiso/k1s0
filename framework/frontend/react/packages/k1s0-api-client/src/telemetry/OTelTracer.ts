import type { Tracer, Span, Context } from '@opentelemetry/api';
import type { RequestTelemetry, TelemetryListener, TelemetryEvent } from './types.js';

/**
 * トレースID生成（16バイト/32文字の16進数）
 */
function generateTraceId(): string {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return Array.from(array)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * スパンID生成（8バイト/16文字の16進数）
 */
function generateSpanId(): string {
  const array = new Uint8Array(8);
  crypto.getRandomValues(array);
  return Array.from(array)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

// OTel SpanKind.CLIENT = 2
const SPAN_KIND_CLIENT = 2;

// OTel SpanStatusCode
const SPAN_STATUS_OK = 1;
const SPAN_STATUS_ERROR = 2;

/**
 * OTel API オブジェクトのインターフェース
 * peerDependencyとして@opentelemetry/apiが必要
 */
interface OTelApi {
  trace: {
    setSpan(context: Context, span: Span): Context;
  };
  context: {
    active(): Context;
  };
}

/**
 * APIリクエストのテレメトリー計測クラス
 * - OTel APIが利用可能な場合はそれを使用
 * - 利用不可の場合はフォールバック実装を使用
 */
export class ApiTelemetry {
  private tracer: Tracer | null = null;
  private otelApi: OTelApi | null = null;
  private listeners: TelemetryListener[] = [];
  private serviceName: string;

  constructor(serviceName: string = 'frontend') {
    this.serviceName = serviceName;
  }

  /**
   * OpenTelemetry Tracerを設定（アプリ初期化時に呼ぶ）
   * @param tracer OTelトレーサー
   * @param otelApi @opentelemetry/api モジュール（trace, context をエクスポートしたもの）
   */
  setTracer(tracer: Tracer, otelApi?: OTelApi): void {
    this.tracer = tracer;
    this.otelApi = otelApi ?? null;
  }

  /**
   * テレメトリーリスナーを追加
   */
  addListener(listener: TelemetryListener): () => void {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  /**
   * イベントを通知
   */
  private emit(event: TelemetryEvent): void {
    for (const listener of this.listeners) {
      try {
        listener(event);
      } catch {
        // リスナーエラーは無視
      }
    }
  }

  /**
   * リクエスト開始時にテレメトリーを開始
   */
  startRequest(method: string, url: string): RequestTelemetry {
    const traceId = generateTraceId();
    const spanId = generateSpanId();
    const startTime = performance.now();

    const telemetry: RequestTelemetry = {
      traceId,
      spanId,
      startTime,
      method,
      url,
    };

    this.emit({ type: 'request_start', telemetry });

    return telemetry;
  }

  /**
   * リクエスト正常終了時にテレメトリーを完了
   */
  endRequest(
    telemetry: RequestTelemetry,
    statusCode: number,
    responseTraceId?: string
  ): RequestTelemetry {
    const updated: RequestTelemetry = {
      ...telemetry,
      // レスポンスからtrace_idが返ってきた場合はそちらを使用
      traceId: responseTraceId ?? telemetry.traceId,
      endTime: performance.now(),
      statusCode,
    };

    this.emit({ type: 'request_end', telemetry: updated });

    return updated;
  }

  /**
   * リクエストエラー時にテレメトリーを完了
   */
  errorRequest(
    telemetry: RequestTelemetry,
    error: Error,
    statusCode?: number,
    errorCode?: string
  ): RequestTelemetry {
    const updated: RequestTelemetry = {
      ...telemetry,
      endTime: performance.now(),
      statusCode,
      errorCode,
    };

    this.emit({ type: 'request_error', telemetry: updated, error });

    return updated;
  }

  /**
   * W3C Trace Context形式のtraceparentヘッダー値を生成
   */
  getTraceparent(telemetry: RequestTelemetry): string {
    // 形式: version-trace_id-parent_id-flags
    // version: 00 (固定)
    // flags: 01 = sampled
    return `00-${telemetry.traceId}-${telemetry.spanId}-01`;
  }

  /**
   * OTelスパンを開始（OTel APIが設定されている場合のみ）
   */
  startSpan(
    name: string,
    telemetry: RequestTelemetry
  ): { span: Span | null; context: Context | null } {
    if (!this.tracer || !this.otelApi) {
      return { span: null, context: null };
    }

    try {
      const span = this.tracer.startSpan(name, {
        kind: SPAN_KIND_CLIENT,
        attributes: {
          'http.method': telemetry.method,
          'http.url': telemetry.url,
          'service.name': this.serviceName,
        },
      });

      const ctx = this.otelApi.trace.setSpan(
        this.otelApi.context.active(),
        span
      );
      return { span, context: ctx };
    } catch {
      return { span: null, context: null };
    }
  }

  /**
   * OTelスパンを終了
   */
  endSpan(span: Span | null, statusCode?: number, errorCode?: string): void {
    if (!span) return;

    try {
      if (statusCode) {
        span.setAttribute('http.status_code', statusCode);
      }
      if (errorCode) {
        span.setAttribute('error.code', errorCode);
        span.setStatus({ code: SPAN_STATUS_ERROR, message: errorCode });
      } else if (statusCode && statusCode >= 400) {
        span.setStatus({ code: SPAN_STATUS_ERROR });
      } else {
        span.setStatus({ code: SPAN_STATUS_OK });
      }

      span.end();
    } catch {
      // OTelエラーは無視
    }
  }
}

/**
 * デフォルトのテレメトリーインスタンス
 */
export const defaultTelemetry = new ApiTelemetry();
