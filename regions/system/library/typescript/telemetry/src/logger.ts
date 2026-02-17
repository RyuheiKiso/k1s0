import pino from 'pino';
import type { TelemetryConfig } from './telemetry';

/**
 * createLogger は pino ベースの構造化ロガーを生成する。
 * サービス名・バージョン・Tier・環境を標準フィールドとして付与する。
 */
export function createLogger(cfg: TelemetryConfig): pino.Logger {
  return pino({
    level: cfg.logLevel,
    base: {
      service: cfg.serviceName,
      version: cfg.version,
      tier: cfg.tier,
      environment: cfg.environment,
    },
  });
}
