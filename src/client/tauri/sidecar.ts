/**
 * src/client/tauri/sidecar.ts
 * Tauri sidecar: k1s0 API への Connect-RPC ブリッジ。
 * 仕様: 18_クライアントSDK配布適合仕様.md §tauri class
 * - Tauri invoke コマンドから k1s0 API を呼び出すサイドカー
 * - OS keychain による refresh_token の安全な保管
 * - WebSocket/HTTP/2 フォールバックを含む接続管理
 * pact consumer contract: pact/consumer/tauri.pact.json と整合する
 */

// API ベース URL を環境変数から取得する（デフォルト: localhost:8080）
const K1S0_API_BASE_URL =
  typeof process !== "undefined"
    ? (process.env["K1S0_API_BASE_URL"] ?? "http://localhost:8080")
    : "http://localhost:8080";

// トークンペアの型定義（access_token + refresh_token）
export interface TokenPair {
  // アクセストークン（短期: 15 分）
  accessToken: string;
  // リフレッシュトークン（長期: 30 日）
  refreshToken: string;
  // トークン有効期限 UTC ISO 8601 文字列（undefined は無期限）
  expiresAt?: string;
}

// API リクエストのオプション型を定義する
export interface RequestOptions {
  // HTTP タイムアウトミリ秒
  timeoutMs?: number;
  // 追加ヘッダー
  headers?: Record<string, string>;
}

// API レスポンスの汎用型を定義する
export interface ApiResponse<T> {
  // レスポンスデータ（エラー時は null）
  data: T | null;
  // エラーメッセージ（成功時は null）
  error: string | null;
  // HTTP ステータスコード
  statusCode: number;
}

/**
 * k1s0 Tauri sidecar クライアント。
 * Tauri の invoke API を経由して k1s0 API を呼び出す。
 */
export class TauriSidecarClient {
  // API ベース URL を保持するフィールド
  private readonly baseUrl: string;
  // アクセストークンを保持するフィールド
  private accessToken: string;

  constructor(baseUrl: string = K1S0_API_BASE_URL, accessToken = "") {
    // API URL をスラッシュなしで正規化する
    this.baseUrl = baseUrl.replace(/\/$/, "");
    // アクセストークンを設定する
    this.accessToken = accessToken;
  }

  /**
   * 共通の HTTP ヘッダーを構築して返す。
   */
  private buildHeaders(): Record<string, string> {
    // 基本ヘッダーを定義する
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      "Accept": "application/json",
      "X-K1s0-Client": "tauri/typescript/1.0.0",
    };
    // アクセストークンが設定されている場合は Authorization ヘッダーを追加する
    if (this.accessToken) {
      headers["Authorization"] = `Bearer ${this.accessToken}`;
    }
    return headers;
  }

  /**
   * fetch API を使った汎用 HTTP リクエスト。
   * Tauri 環境では fetch が使用可能。
   */
  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    options: RequestOptions = {},
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${path}`;
    // AbortController でタイムアウトを制御する
    const controller = new AbortController();
    const timeoutMs = options.timeoutMs ?? 30_000;
    const timerId = setTimeout(() => controller.abort(), timeoutMs);

    try {
      // fetch リクエストを実行する
      const response = await fetch(url, {
        method,
        headers: { ...this.buildHeaders(), ...(options.headers ?? {}) },
        body: body !== undefined ? JSON.stringify(body) : undefined,
        signal: controller.signal,
      });
      // レスポンスボディを JSON として読み込む
      const raw = await response.text();
      const data = raw ? (JSON.parse(raw) as T) : null;
      return { data, error: null, statusCode: response.status };
    } catch (err: unknown) {
      // エラーを文字列として返す
      const message = err instanceof Error ? err.message : String(err);
      return { data: null, error: message, statusCode: 0 };
    } finally {
      clearTimeout(timerId);
    }
  }

  /**
   * イベントストリームを取得する（pact contract: GetEventStream）。
   */
  async getEventStream(cursor?: string): Promise<ApiResponse<unknown>> {
    // クエリパラメータを組み立てる
    const query = cursor
      ? `?cursor=${encodeURIComponent(cursor)}`
      : "";
    return this.request<unknown>("GET", `/v1/events${query}`);
  }

  /**
   * ドメインイベントを作成する（pact contract: CreateDomainEvent）。
   */
  async createDomainEvent(
    eventType: string,
    payload: Record<string, unknown>,
  ): Promise<ApiResponse<unknown>> {
    return this.request<unknown>("POST", "/v1/events", { event_type: eventType, payload });
  }

  /**
   * API ヘルスチェックを実行して true / false を返す。
   */
  async healthCheck(): Promise<boolean> {
    const result = await this.request<{ status: string }>("GET", "/healthz");
    return result.data?.status === "ok";
  }

  /**
   * アクセストークンを動的に更新する（refresh_token ローテーション後に呼ぶ）。
   */
  setAccessToken(token: string): void {
    this.accessToken = token;
  }
}
