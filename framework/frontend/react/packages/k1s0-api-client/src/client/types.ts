import type { TokenManager } from '../auth/TokenManager.js';
import type { ApiTelemetry } from '../telemetry/OTelTracer.js';

/**
 * APIクライアントの設定
 */
export interface ApiClientConfig {
  /** ベースURL（末尾スラッシュなし） */
  baseUrl: string;
  /** デフォルトタイムアウト（ms）デフォルト: 30000 */
  timeout?: number;
  /** リトライ回数（デフォルト: 0 = リトライなし） */
  retryCount?: number;
  /** リトライ対象のステータスコード */
  retryStatusCodes?: number[];
  /** トークンマネージャー（認証が必要な場合） */
  tokenManager?: TokenManager;
  /** テレメトリー（計測が必要な場合） */
  telemetry?: ApiTelemetry;
  /** カスタムヘッダー */
  headers?: Record<string, string>;
  /** 認証エラー時のコールバック */
  onAuthError?: () => void;
}

/**
 * リクエストオプション
 */
export interface RequestOptions {
  /** HTTPメソッド */
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  /** リクエストヘッダー */
  headers?: Record<string, string>;
  /** リクエストボディ（JSON） */
  body?: unknown;
  /** タイムアウト（ms）設定で上書き */
  timeout?: number;
  /** 認証をスキップするか */
  skipAuth?: boolean;
  /** このリクエストでリトライするか（デフォルトは設定に従う） */
  retry?: boolean;
  /** AbortSignal（外部からのキャンセル） */
  signal?: AbortSignal;
}

/**
 * APIレスポンス
 */
export interface ApiResponse<T> {
  /** レスポンスデータ */
  data: T;
  /** HTTPステータスコード */
  status: number;
  /** レスポンスヘッダー */
  headers: Headers;
  /** トレースID（デバッグ用） */
  traceId?: string;
}

/**
 * リトライポリシー
 */
export interface RetryPolicy {
  /** リトライ回数 */
  count: number;
  /** リトライ対象のステータスコード */
  statusCodes: number[];
  /** リトライ間隔（ms） */
  delay: number;
  /** バックオフ倍率 */
  backoffMultiplier: number;
  /** 最大リトライ間隔（ms） */
  maxDelay: number;
}

/**
 * デフォルトのリトライポリシー
 * 原則 retry 0（リトライなし）
 */
export const DEFAULT_RETRY_POLICY: RetryPolicy = {
  count: 0,
  statusCodes: [502, 503, 504],
  delay: 1000,
  backoffMultiplier: 2,
  maxDelay: 10000,
};

/**
 * デフォルトタイムアウト（30秒）
 */
export const DEFAULT_TIMEOUT = 30000;
