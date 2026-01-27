import type { ErrorInfo, ObservabilityConfig } from '../types.js';
import { Logger } from '../logging/Logger.js';
import { TracingService } from '../tracing/TracingService.js';

/**
 * エラーイベント
 */
export interface ErrorEvent {
  /** エラー情報 */
  error: ErrorInfo;
  /** 発生時刻 */
  timestamp: number;
  /** トレースID */
  traceId?: string;
  /** スパンID */
  spanId?: string;
  /** 発生したURL */
  url?: string;
  /** ユーザーエージェント */
  userAgent?: string;
  /** 追加コンテキスト */
  context?: Record<string, unknown>;
}

/**
 * エラーリスナー
 */
export type ErrorListener = (event: ErrorEvent) => void;

/**
 * エラートラッキングクラス
 *
 * - グローバルエラーハンドリング
 * - Promise rejection のキャッチ
 * - React Error Boundary 統合
 * - エラーの構造化
 */
export class ErrorTracker {
  private config: ObservabilityConfig;
  private logger: Logger | null;
  private tracingService: TracingService | null;
  private listeners: Set<ErrorListener> = new Set();
  private errorBuffer: ErrorEvent[] = [];

  // グローバルハンドラーの参照（クリーンアップ用）
  private errorHandler: ((event: ErrorEvent) => void) | null = null;
  private rejectionHandler: ((event: PromiseRejectionEvent) => void) | null = null;

  constructor(
    config: ObservabilityConfig,
    logger?: Logger,
    tracingService?: TracingService
  ) {
    this.config = config;
    this.logger = logger ?? null;
    this.tracingService = tracingService ?? null;
  }

  /**
   * グローバルエラーハンドリングを有効化
   */
  enableGlobalHandling(): () => void {
    if (typeof window === 'undefined') {
      return () => {};
    }

    // window.onerror
    const errorHandler = (event: ErrorEvent) => {
      const error =
        event.error instanceof Error
          ? event.error
          : new Error(event.error ? String(event.error) : 'Unknown error');
      this.captureError(error);
    };

    // unhandledrejection
    const rejectionHandler = (event: PromiseRejectionEvent) => {
      const error =
        event.reason instanceof Error
          ? event.reason
          : new Error(String(event.reason));
      this.captureError(error, { unhandledRejection: true });
    };

    window.addEventListener('error', errorHandler as unknown as EventListener);
    window.addEventListener('unhandledrejection', rejectionHandler);

    this.errorHandler = errorHandler as unknown as (event: ErrorEvent) => void;
    this.rejectionHandler = rejectionHandler;

    // クリーンアップ関数
    return () => {
      window.removeEventListener(
        'error',
        errorHandler as unknown as EventListener
      );
      window.removeEventListener('unhandledrejection', rejectionHandler);
      this.errorHandler = null;
      this.rejectionHandler = null;
    };
  }

  /**
   * エラーをキャプチャ
   */
  captureError(error: Error, context?: Record<string, unknown>): ErrorEvent {
    const tracingContext = this.tracingService?.getCurrentContext();

    const event: ErrorEvent = {
      error: this.serializeError(error),
      timestamp: Date.now(),
      traceId: tracingContext?.traceId,
      spanId: tracingContext?.spanId,
      url: typeof window !== 'undefined' ? window.location.href : undefined,
      userAgent:
        typeof navigator !== 'undefined' ? navigator.userAgent : undefined,
      context,
    };

    // ログに出力
    this.logger?.error(error.message, error, {
      error_name: error.name,
      error_code: 'code' in error ? (error.code as string) : undefined,
      ...context,
    });

    // バッファに追加
    this.errorBuffer.push(event);

    // リスナーに通知
    for (const listener of this.listeners) {
      try {
        listener(event);
      } catch {
        // リスナーエラーは無視
      }
    }

    // バッファサイズチェック
    if (
      this.config.enableBatching &&
      this.errorBuffer.length >= this.config.batchSize
    ) {
      this.flush();
    }

    return event;
  }

  /**
   * 例外をキャプチャ（try-catch 用）
   */
  captureException(error: unknown, context?: Record<string, unknown>): ErrorEvent {
    if (error instanceof Error) {
      return this.captureError(error, context);
    }

    // Error 以外の場合はラップ
    const wrappedError = new Error(String(error));
    wrappedError.name = 'CapturedValue';
    return this.captureError(wrappedError, {
      ...context,
      originalValue: error,
    });
  }

  /**
   * メッセージをエラーとしてキャプチャ
   */
  captureMessage(
    message: string,
    level: 'error' | 'warning' = 'error',
    context?: Record<string, unknown>
  ): ErrorEvent {
    const error = new Error(message);
    error.name = level === 'warning' ? 'Warning' : 'Error';
    return this.captureError(error, { ...context, level });
  }

  /**
   * React Error Boundary 用のハンドラー
   */
  createErrorBoundaryHandler(
    componentName?: string
  ): (error: Error, errorInfo: { componentStack?: string }) => void {
    return (error: Error, errorInfo: { componentStack?: string }) => {
      this.captureError(error, {
        componentName,
        componentStack: errorInfo.componentStack,
        type: 'react_error_boundary',
      });
    };
  }

  /**
   * エラーリスナーを追加
   */
  onError(listener: ErrorListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * バッファをフラッシュ
   */
  flush(): ErrorEvent[] {
    const errors = [...this.errorBuffer];
    this.errorBuffer = [];
    return errors;
  }

  /**
   * リソースを解放
   */
  dispose(): void {
    // グローバルハンドラーをクリーンアップ
    if (
      typeof window !== 'undefined' &&
      this.errorHandler &&
      this.rejectionHandler
    ) {
      window.removeEventListener(
        'error',
        this.errorHandler as unknown as EventListener
      );
      window.removeEventListener('unhandledrejection', this.rejectionHandler);
    }

    this.listeners.clear();
    this.errorBuffer = [];
    this.errorHandler = null;
    this.rejectionHandler = null;
  }

  /**
   * エラーをシリアライズ
   */
  private serializeError(error: Error): ErrorInfo {
    const info: ErrorInfo = {
      name: error.name,
      message: error.message,
      stack: error.stack,
    };

    // カスタムエラーコード
    if ('code' in error && typeof error.code === 'string') {
      info.code = error.code;
    }

    // cause チェーン
    if (error.cause instanceof Error) {
      info.cause = this.serializeError(error.cause);
    }

    return info;
  }
}
