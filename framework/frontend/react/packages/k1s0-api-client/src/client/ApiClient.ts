import type {
  ApiClientConfig,
  RequestOptions,
  ApiResponse,
  RetryPolicy,
} from './types.js';
import { DEFAULT_RETRY_POLICY, DEFAULT_TIMEOUT } from './types.js';
import { ApiError } from '../error/ApiError.js';
import type { TokenManager } from '../auth/TokenManager.js';
import { ApiTelemetry, defaultTelemetry } from '../telemetry/OTelTracer.js';
import type { RequestTelemetry } from '../telemetry/types.js';

/**
 * 共通APIクライアント
 * - 認証トークンの自動付与（設定時）
 * - タイムアウト制御
 * - リトライ（opt-in、デフォルトは0回）
 * - OTel計測
 * - エラーの統一変換
 */
export class ApiClient {
  private baseUrl: string;
  private timeout: number;
  private retryPolicy: RetryPolicy;
  private tokenManager: TokenManager | undefined;
  private telemetry: ApiTelemetry;
  private defaultHeaders: Record<string, string>;
  private onAuthError: (() => void) | undefined;

  constructor(config: ApiClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // 末尾スラッシュ削除
    this.timeout = config.timeout ?? DEFAULT_TIMEOUT;
    this.retryPolicy = {
      ...DEFAULT_RETRY_POLICY,
      count: config.retryCount ?? 0,
      statusCodes: config.retryStatusCodes ?? DEFAULT_RETRY_POLICY.statusCodes,
    };
    this.tokenManager = config.tokenManager;
    this.telemetry = config.telemetry ?? defaultTelemetry;
    this.defaultHeaders = {
      'Content-Type': 'application/json',
      Accept: 'application/json',
      ...config.headers,
    };
    this.onAuthError = config.onAuthError;
  }

  /**
   * GETリクエスト
   */
  async get<T>(path: string, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'GET' });
  }

  /**
   * POSTリクエスト
   */
  async post<T>(
    path: string,
    body?: unknown,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'POST', body });
  }

  /**
   * PUTリクエスト
   */
  async put<T>(
    path: string,
    body?: unknown,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'PUT', body });
  }

  /**
   * PATCHリクエスト
   */
  async patch<T>(
    path: string,
    body?: unknown,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'PATCH', body });
  }

  /**
   * DELETEリクエスト
   */
  async delete<T>(path: string, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'DELETE' });
  }

  /**
   * 汎用リクエスト
   */
  async request<T>(path: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${path.startsWith('/') ? path : `/${path}`}`;
    const method = options.method ?? 'GET';
    const timeout = options.timeout ?? this.timeout;
    const shouldRetry = options.retry ?? this.retryPolicy.count > 0;

    // テレメトリー開始
    const telemetry = this.telemetry.startRequest(method, url);

    try {
      // 認証トークン取得
      const authHeader = await this.getAuthHeader(options.skipAuth);

      // ヘッダー組み立て
      const headers: Record<string, string> = {
        ...this.defaultHeaders,
        ...options.headers,
        // W3C Trace Context
        traceparent: this.telemetry.getTraceparent(telemetry),
      };

      if (authHeader) {
        headers['Authorization'] = authHeader;
      }

      // リクエスト実行（リトライ付き）
      const response = await this.executeWithRetry(
        url,
        {
          method,
          headers,
          body: options.body ? JSON.stringify(options.body) : undefined,
          signal: this.createTimeoutSignal(timeout, options.signal),
        },
        shouldRetry ? this.retryPolicy : { ...this.retryPolicy, count: 0 },
        telemetry
      );

      // レスポンスからtrace_idを取得
      const responseTraceId = response.headers.get('x-trace-id') ?? undefined;

      // エラーレスポンスの処理
      if (!response.ok) {
        const error = await ApiError.fromResponse(response, telemetry.traceId);
        this.telemetry.errorRequest(
          telemetry,
          error,
          response.status,
          error.errorCode
        );

        // 認証エラーの場合はコールバックを呼ぶ
        if (error.requiresAuthentication && this.onAuthError) {
          this.onAuthError();
        }

        throw error;
      }

      // 成功レスポンスの処理
      this.telemetry.endRequest(telemetry, response.status, responseTraceId);

      // 204 No Content の場合は空オブジェクト
      if (response.status === 204) {
        return {
          data: {} as T,
          status: response.status,
          headers: response.headers,
          traceId: responseTraceId,
        };
      }

      const data = await response.json();
      return {
        data: data as T,
        status: response.status,
        headers: response.headers,
        traceId: responseTraceId,
      };
    } catch (error) {
      // 既にApiErrorの場合はそのまま投げる
      if (error instanceof ApiError) {
        throw error;
      }

      // ネットワークエラー等をApiErrorに変換
      const apiError = ApiError.fromNetworkError(error, telemetry.traceId);
      this.telemetry.errorRequest(telemetry, apiError);
      throw apiError;
    }
  }

  /**
   * 認証ヘッダーを取得
   */
  private async getAuthHeader(skipAuth?: boolean): Promise<string | null> {
    if (skipAuth || !this.tokenManager) {
      return null;
    }

    const result = await this.tokenManager.getValidToken();

    switch (result.type) {
      case 'valid':
      case 'refreshed':
        return `Bearer ${result.token}`;
      case 'expired':
        // 認証期限切れ
        if (this.onAuthError) {
          this.onAuthError();
        }
        return null;
      case 'none':
        return null;
    }
  }

  /**
   * タイムアウト付きAbortSignalを作成
   */
  private createTimeoutSignal(
    timeout: number,
    externalSignal?: AbortSignal
  ): AbortSignal {
    const controller = new AbortController();

    // タイムアウト設定
    const timeoutId = setTimeout(() => {
      controller.abort(new DOMException('Timeout', 'AbortError'));
    }, timeout);

    // 外部シグナルがある場合はリンク
    if (externalSignal) {
      if (externalSignal.aborted) {
        controller.abort(externalSignal.reason);
      } else {
        externalSignal.addEventListener('abort', () => {
          controller.abort(externalSignal.reason);
          clearTimeout(timeoutId);
        });
      }
    }

    // クリーンアップ
    controller.signal.addEventListener('abort', () => {
      clearTimeout(timeoutId);
    });

    return controller.signal;
  }

  /**
   * リトライ付きでリクエストを実行
   */
  private async executeWithRetry(
    url: string,
    init: RequestInit,
    retryPolicy: RetryPolicy,
    telemetry: RequestTelemetry
  ): Promise<Response> {
    let lastError: Error | null = null;
    let delay = retryPolicy.delay;

    for (let attempt = 0; attempt <= retryPolicy.count; attempt++) {
      try {
        const response = await fetch(url, init);

        // リトライ対象のステータスコードかチェック
        if (
          attempt < retryPolicy.count &&
          retryPolicy.statusCodes.includes(response.status)
        ) {
          // リトライする
          await this.sleep(delay);
          delay = Math.min(
            delay * retryPolicy.backoffMultiplier,
            retryPolicy.maxDelay
          );
          continue;
        }

        return response;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        // タイムアウト/ネットワークエラーの場合、リトライ対象
        if (attempt < retryPolicy.count) {
          await this.sleep(delay);
          delay = Math.min(
            delay * retryPolicy.backoffMultiplier,
            retryPolicy.maxDelay
          );
          continue;
        }

        throw lastError;
      }
    }

    throw lastError ?? new Error('Request failed');
  }

  /**
   * 指定時間待機
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

/**
 * APIクライアントのファクトリ関数
 */
export function createApiClient(config: ApiClientConfig): ApiClient {
  return new ApiClient(config);
}
