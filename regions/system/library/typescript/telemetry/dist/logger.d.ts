import pino from 'pino';
import type { TelemetryConfig } from './telemetry';
/**
 * createLogger は pino ベースの構造化ロガーを生成する。
 * サービス名・バージョン・Tier・環境を標準フィールドとして付与する。
 */
export declare function createLogger(cfg: TelemetryConfig): pino.Logger;
