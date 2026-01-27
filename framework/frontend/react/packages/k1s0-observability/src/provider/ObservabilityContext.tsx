import React, {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  type ReactNode,
} from 'react';
import type { ObservabilityConfig, SpanInfo, PerformanceMetric } from '../types.js';
import { ObservabilityConfigSchema } from '../types.js';
import { TracingService, SpanBuilder } from '../tracing/TracingService.js';
import { Logger } from '../logging/Logger.js';
import { MetricsCollector } from '../metrics/MetricsCollector.js';
import { ErrorTracker, type ErrorEvent } from '../errors/ErrorTracker.js';

/**
 * 観測性コンテキストの値
 */
export interface ObservabilityContextValue {
  /** 設定 */
  config: ObservabilityConfig;
  /** トレーシングサービス */
  tracing: TracingService;
  /** ロガー */
  logger: Logger;
  /** メトリクスコレクター */
  metrics: MetricsCollector;
  /** エラートラッカー */
  errors: ErrorTracker;
}

const ObservabilityContext = createContext<ObservabilityContextValue | null>(
  null
);

interface ObservabilityProviderProps {
  children: ReactNode;
  /** 観測性設定 */
  config: Partial<ObservabilityConfig> & Pick<ObservabilityConfig, 'serviceName' | 'env'>;
  /** グローバルエラーハンドリングを有効にするか（デフォルト: true） */
  enableGlobalErrorHandling?: boolean;
  /** Web Vitals 計測を有効にするか（デフォルト: true） */
  enableWebVitals?: boolean;
  /** スパン完了時のコールバック */
  onSpan?: (span: SpanInfo) => void;
  /** エラー発生時のコールバック */
  onError?: (event: ErrorEvent) => void;
  /** メトリクス記録時のコールバック */
  onMetric?: (metric: PerformanceMetric) => void;
}

/**
 * 観測性コンテキストプロバイダ
 *
 * - トレーシング、ロギング、メトリクス、エラートラッキングを統合
 * - グローバルエラーハンドリング
 * - Web Vitals 自動計測
 */
export function ObservabilityProvider({
  children,
  config: configProp,
  enableGlobalErrorHandling = true,
  enableWebVitals = true,
  onSpan,
  onError,
  onMetric,
}: ObservabilityProviderProps) {
  // 設定のバリデーションとデフォルト値の適用
  const config = useMemo(() => {
    const parsed = ObservabilityConfigSchema.safeParse(configProp);
    if (parsed.success) {
      return parsed.data;
    }
    // バリデーション失敗時はデフォルト値でフォールバック
    console.warn('Invalid observability config:', parsed.error);
    return ObservabilityConfigSchema.parse({
      ...configProp,
      logLevel: configProp.logLevel ?? 'INFO',
      samplingRate: configProp.samplingRate ?? 1.0,
      enableConsole: configProp.enableConsole ?? true,
      enableBatching: configProp.enableBatching ?? true,
      batchSize: configProp.batchSize ?? 100,
      batchIntervalMs: configProp.batchIntervalMs ?? 5000,
    });
  }, [configProp]);

  // サービスインスタンスを一度だけ作成
  const servicesRef = useRef<{
    tracing: TracingService;
    logger: Logger;
    metrics: MetricsCollector;
    errors: ErrorTracker;
  } | null>(null);

  if (!servicesRef.current) {
    const tracing = new TracingService(config);
    const logger = new Logger(config, tracing);
    const metrics = new MetricsCollector(config);
    const errors = new ErrorTracker(config, logger, tracing);

    servicesRef.current = { tracing, logger, metrics, errors };
  }

  const { tracing, logger, metrics, errors } = servicesRef.current;

  // コールバックの設定
  useEffect(() => {
    const cleanups: (() => void)[] = [];

    if (onSpan) {
      cleanups.push(tracing.onSpan(onSpan));
    }

    if (onError) {
      cleanups.push(errors.onError(onError));
    }

    if (onMetric) {
      cleanups.push(metrics.onMetric(onMetric));
    }

    return () => {
      cleanups.forEach((cleanup) => cleanup());
    };
  }, [tracing, errors, metrics, onSpan, onError, onMetric]);

  // グローバルエラーハンドリング
  useEffect(() => {
    if (!enableGlobalErrorHandling) return;

    const cleanup = errors.enableGlobalHandling();
    return cleanup;
  }, [errors, enableGlobalErrorHandling]);

  // Web Vitals 計測
  useEffect(() => {
    if (!enableWebVitals) return;

    const cleanup = metrics.observeWebVitals();
    return cleanup;
  }, [metrics, enableWebVitals]);

  // クリーンアップ
  useEffect(() => {
    return () => {
      tracing.dispose();
      logger.dispose();
      metrics.dispose();
      errors.dispose();
    };
  }, [tracing, logger, metrics, errors]);

  const value = useMemo<ObservabilityContextValue>(
    () => ({
      config,
      tracing,
      logger,
      metrics,
      errors,
    }),
    [config, tracing, logger, metrics, errors]
  );

  return (
    <ObservabilityContext.Provider value={value}>
      {children}
    </ObservabilityContext.Provider>
  );
}

/**
 * 観測性コンテキストを取得するフック
 */
export function useObservability(): ObservabilityContextValue {
  const context = useContext(ObservabilityContext);
  if (!context) {
    throw new Error(
      'useObservability must be used within an ObservabilityProvider'
    );
  }
  return context;
}

/**
 * トレーシングサービスを取得するフック
 */
export function useTracing(): TracingService {
  return useObservability().tracing;
}

/**
 * ロガーを取得するフック
 */
export function useLogger(): Logger {
  return useObservability().logger;
}

/**
 * メトリクスコレクターを取得するフック
 */
export function useMetrics(): MetricsCollector {
  return useObservability().metrics;
}

/**
 * エラートラッカーを取得するフック
 */
export function useErrorTracker(): ErrorTracker {
  return useObservability().errors;
}

/**
 * スパンを作成するフック
 */
export function useSpan(name: string): SpanBuilder {
  const tracing = useTracing();
  return tracing.startSpan(name);
}

/**
 * 現在のトレースコンテキストを取得するフック
 */
export function useTraceContext(): {
  traceId: string | undefined;
  spanId: string | undefined;
  traceparent: string | null;
} {
  const tracing = useTracing();
  return {
    traceId: tracing.getCurrentTraceId(),
    spanId: tracing.getCurrentSpanId(),
    traceparent: tracing.getTraceparent(),
  };
}
