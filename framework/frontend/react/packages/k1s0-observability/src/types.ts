import { z } from 'zod';

/**
 * ログレベル
 */
export type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';

/**
 * ログレベルの数値優先度
 */
export const LOG_LEVEL_PRIORITY: Record<LogLevel, number> = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
};

/**
 * 観測性設定のスキーマ
 * バックエンドの k1s0-observability crate の ObservabilityConfig に対応
 */
export const ObservabilityConfigSchema = z.object({
  /** サービス名 */
  serviceName: z.string().min(1),
  /** 環境名（dev/stg/prod） */
  env: z.enum(['dev', 'stg', 'prod']),
  /** サービスバージョン */
  version: z.string().optional(),
  /** OTLP エンドポイント */
  otlpEndpoint: z.string().url().optional(),
  /** サンプリングレート（0.0 - 1.0） */
  samplingRate: z.number().min(0).max(1).default(1.0),
  /** 最小ログレベル */
  logLevel: z.enum(['DEBUG', 'INFO', 'WARN', 'ERROR']).default('INFO'),
  /** コンソール出力を有効にするか */
  enableConsole: z.boolean().default(true),
  /** バッチ送信を有効にするか */
  enableBatching: z.boolean().default(true),
  /** バッチサイズ */
  batchSize: z.number().int().positive().default(100),
  /** バッチ送信間隔（ms） */
  batchIntervalMs: z.number().int().positive().default(5000),
});

export type ObservabilityConfig = z.infer<typeof ObservabilityConfigSchema>;

/**
 * 必須ログフィールド
 * observability.md の規約に準拠
 */
export interface RequiredLogFields {
  /** ISO 8601 形式のタイムスタンプ */
  timestamp: string;
  /** ログレベル */
  level: LogLevel;
  /** サービス名 */
  service_name: string;
  /** 環境名 */
  env: string;
  /** トレースID */
  trace_id: string;
  /** スパンID */
  span_id: string;
}

/**
 * ログエントリ
 */
export interface LogEntry extends RequiredLogFields {
  /** ログメッセージ */
  message: string;
  /** リクエストID */
  request_id?: string;
  /** 追加フィールド */
  [key: string]: unknown;
}

/**
 * スパン情報
 */
export interface SpanInfo {
  /** トレースID */
  traceId: string;
  /** スパンID */
  spanId: string;
  /** 親スパンID */
  parentSpanId?: string;
  /** スパン名 */
  name: string;
  /** 開始時刻（Unix timestamp ms） */
  startTime: number;
  /** 終了時刻（Unix timestamp ms） */
  endTime?: number;
  /** 属性 */
  attributes: Record<string, string | number | boolean>;
  /** ステータス */
  status?: SpanStatus;
}

/**
 * スパンステータス
 */
export interface SpanStatus {
  code: 'OK' | 'ERROR' | 'UNSET';
  message?: string;
}

/**
 * エラー情報
 */
export interface ErrorInfo {
  /** エラー名 */
  name: string;
  /** エラーメッセージ */
  message: string;
  /** スタックトレース */
  stack?: string;
  /** エラーコード */
  code?: string;
  /** 元のエラー */
  cause?: ErrorInfo;
}

/**
 * パフォーマンス計測情報
 */
export interface PerformanceMetric {
  /** メトリクス名 */
  name: string;
  /** 値 */
  value: number;
  /** 単位 */
  unit: 'ms' | 'bytes' | 'count' | 'percent';
  /** タイムスタンプ */
  timestamp: number;
  /** タグ */
  tags?: Record<string, string>;
}

/**
 * Web Vitals メトリクス
 */
export interface WebVitals {
  /** Largest Contentful Paint */
  LCP?: number;
  /** First Input Delay */
  FID?: number;
  /** Cumulative Layout Shift */
  CLS?: number;
  /** First Contentful Paint */
  FCP?: number;
  /** Time to First Byte */
  TTFB?: number;
  /** Interaction to Next Paint */
  INP?: number;
}

/**
 * ログシンク（ログの出力先）
 */
export interface LogSink {
  /** ログを出力 */
  write(entry: LogEntry): void;
  /** バッファをフラッシュ */
  flush?(): Promise<void>;
  /** リソースを解放 */
  dispose?(): void;
}

/**
 * トレースエクスポーター
 */
export interface TraceExporter {
  /** スパンをエクスポート */
  export(spans: SpanInfo[]): Promise<void>;
  /** シャットダウン */
  shutdown(): Promise<void>;
}

/**
 * メトリクスエクスポーター
 */
export interface MetricsExporter {
  /** メトリクスをエクスポート */
  export(metrics: PerformanceMetric[]): Promise<void>;
  /** シャットダウン */
  shutdown(): Promise<void>;
}

/**
 * 観測性コンテキスト
 */
export interface ObservabilityContext {
  /** トレースID */
  traceId: string;
  /** スパンID */
  spanId: string;
  /** リクエストID */
  requestId?: string;
  /** 追加コンテキスト */
  baggage?: Record<string, string>;
}
