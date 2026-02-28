import pino from 'pino';
import { trace } from '@opentelemetry/api';
import type { TelemetryConfig } from './telemetry';

/**
 * createLogger は pino ベースの構造化ロガーを生成する。
 * サービス名・バージョン・Tier・環境を標準フィールドとして付与する。
 * アクティブな OpenTelemetry スパンがあれば trace_id / span_id を自動注入する。
 * logFormat が "text" の場合は pino-pretty で人間可読フォーマットを使用する。
 */
export function createLogger(cfg: TelemetryConfig): pino.Logger {
  const options: pino.LoggerOptions = {
    level: cfg.logLevel,
    base: {
      service: cfg.serviceName,
      version: cfg.version,
      tier: cfg.tier,
      environment: cfg.environment,
    },
    mixin() {
      const span = trace.getActiveSpan();
      if (span) {
        const spanContext = span.spanContext();
        return {
          trace_id: spanContext.traceId,
          span_id: spanContext.spanId,
        };
      }
      return {};
    },
  };

  if (cfg.logFormat === 'text') {
    options.transport = {
      target: 'pino-pretty',
      options: { colorize: true, translateTime: 'SYS:standard' },
    };
  }

  return pino(options);
}
