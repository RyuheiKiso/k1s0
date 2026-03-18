import pino from 'pino';
import type { TelemetryConfig } from './telemetry';
/**
 * createLogger は pino ベースの構造化ロガーを生成する。
 * サービス名・バージョン・Tier・環境を標準フィールドとして付与する。
 * アクティブな OpenTelemetry スパンがあれば trace_id / span_id を自動注入する。
 * logFormat が "text" の場合は pino-pretty で人間可読フォーマットを使用する。
 */
export declare function createLogger(cfg: TelemetryConfig): pino.Logger;
