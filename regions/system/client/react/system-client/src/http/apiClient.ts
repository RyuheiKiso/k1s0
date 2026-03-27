import axios, { type AxiosInstance, type AxiosError } from 'axios';

interface ApiClientOptions {
  baseURL: string;
  timeout?: number;
  // 401 未認証エラー時に呼び出されるコールバック（リダイレクト等の処理を呼び出し側で制御可能にする）
  onUnauthorized?: () => void;
}

/**
 * CSRFトークンのモジュールレベルキャッシュ（H-11 監査対応）
 * SPA（シングルページアプリケーション）ではモジュールインスタンスが1つのみ存在するため、
 * モジュールスコープ変数は安全に使用できる。
 * SSRを導入する場合はリクエストスコープへの移行が必要。
 */
let _csrfToken: string | null = null;

/**
 * CSRFトークンをモジュールキャッシュに設定する。
 * BFF の /auth/session レスポンスから取得したトークンを保持し、
 * 以降のすべてのリクエストインターセプターで利用される。
 * ログアウト時は null を渡してクリアすること。
 */
export function setCsrfToken(token: string | null): void {
  _csrfToken = token;
}

export function createApiClient({ baseURL, timeout = 30000, onUnauthorized }: ApiClientOptions): AxiosInstance {
  const client = axios.create({
    baseURL,
    timeout,
    withCredentials: true,
    headers: {
      'Content-Type': 'application/json',
    },
  });

  // CSRF トークンインターセプター
  client.interceptors.request.use((config) => {
    const csrfToken = getCsrfToken();
    if (csrfToken) {
      config.headers['X-CSRF-Token'] = csrfToken;
    }
    return config;
  });

  // レスポンスエラーインターセプター
  client.interceptors.response.use(
    (response) => response,
    (error: AxiosError) => {
      const status = error.response?.status;

      // 401 未認証時はコールバックを呼び出す（呼び出し側でリダイレクト等を制御）
      if (status === 401) {
        onUnauthorized?.();
      } else if (status === 403) {
        console.error('[API] 403 Forbidden:', error.config?.url);
      } else if (status != null && status >= 500) {
        console.error('[API] Server Error:', status, error.config?.url);
      }

      return Promise.reject(error);
    },
  );

  return client;
}

/**
 * CSRFトークンを取得する。
 * モジュールキャッシュに値があればそれを優先し、なければ
 * HTML の <meta name="csrf-token"> タグにフォールバックする。
 * SSR 環境（document が未定義）では null を返す。
 */
function getCsrfToken(): string | null {
  if (_csrfToken) return _csrfToken;
  if (typeof document === 'undefined') return null;
  const meta = document.querySelector<HTMLMetaElement>('meta[name="csrf-token"]');
  return meta?.content ?? null;
}
