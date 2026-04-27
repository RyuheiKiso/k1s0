// @k1s0/api-client エントリポイント。
//
// tier3 web の app から BFF（portal-bff / admin-bff）を呼ぶ薄い fetch wrapper。
// リリース時点 では REST + GraphQL の最小呼出のみ。リリース時点 で TanStack Query / Apollo に拡張。

import type { AppConfig } from '@k1s0/config';

// ApiClient の構築に必要な依存。
export interface ApiClientOptions {
  // 設定（BFF URL / tenantId 等）。
  config: AppConfig;
  // テスト容易性のため fetch 関数を差し替え可能（既定は global fetch）。
  fetchFn?: typeof fetch;
  // 認証トークン取得関数（未指定時は Authorization ヘッダ未付与）。
  getToken?: () => string | null;
}

// 失敗時の標準エラー型（BFF が返す JSON エラーボディに揃える）。
export class ApiError extends Error {
  // HTTP status。
  status: number;
  // BFF コード（E-T3-BFF-* / E-T1-* 等、生のまま伝搬する）。
  code: string;
  // カテゴリ（VALIDATION / UPSTREAM / 等）。
  category: string;

  constructor(status: number, code: string, message: string, category: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
    this.category = category;
  }
}

// State.Get の戻り値（BFF の REST レスポンスと整合）。
export interface StateValue {
  data: string;
  etag: string;
  found: boolean;
}

// ApiClient は BFF への呼出を集約する。
export class ApiClient {
  private readonly config: AppConfig;
  private readonly fetchFn: typeof fetch;
  private readonly getToken: () => string | null;

  constructor(options: ApiClientOptions) {
    this.config = options.config;
    this.fetchFn = options.fetchFn ?? fetch.bind(globalThis);
    this.getToken = options.getToken ?? (() => null);
  }

  // POST /api/state/get（BFF REST 経由）。
  async stateGet(store: string, key: string): Promise<StateValue> {
    return await this.postJson<StateValue>('/api/state/get', { store, key });
  }

  // POST /graphql でクエリを送る汎用ヘルパ。
  async graphql<T>(query: string, variables?: Record<string, unknown>): Promise<T> {
    const body = await this.postJson<{ data?: T; errors?: { message: string }[] }>(
      '/graphql',
      { query, variables: variables ?? {} },
    );
    if (body.errors && body.errors.length > 0) {
      throw new ApiError(200, 'E-T3-BFF-GQL', body.errors[0]?.message ?? 'graphql error', 'UPSTREAM');
    }
    if (!body.data) {
      throw new ApiError(200, 'E-T3-BFF-GQL-EMPTY', 'graphql returned no data', 'UPSTREAM');
    }
    return body.data;
  }

  // 共通 POST JSON ヘルパ。
  private async postJson<T>(path: string, body: unknown): Promise<T> {
    // 完全 URL を組み立てる。
    const url = `${this.config.bffUrl.replace(/\/$/, '')}${path}`;
    // Authorization / X-Tenant-Id ヘッダ。
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'X-Tenant-Id': this.config.tenantId,
    };
    const token = this.getToken();
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
    // fetch 実行。
    const res = await this.fetchFn(url, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });
    if (!res.ok) {
      // BFF の JSON エラーボディを試行的に取り出す。
      let errBody: { error?: { code?: string; message?: string; category?: string } } = {};
      try {
        errBody = (await res.json()) as typeof errBody;
      } catch (_e) {
        // 非 JSON 応答は無視して汎用エラーへ。
      }
      throw new ApiError(
        res.status,
        errBody.error?.code ?? `HTTP_${res.status}`,
        errBody.error?.message ?? `request failed: ${res.status}`,
        errBody.error?.category ?? 'UPSTREAM',
      );
    }
    return (await res.json()) as T;
  }
}

// 利便性のための factory（DI フレームワーク不要にする）。
export function createApiClient(options: ApiClientOptions): ApiClient {
  return new ApiClient(options);
}
