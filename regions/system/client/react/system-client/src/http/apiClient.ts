import axios, { type AxiosInstance, type AxiosError } from 'axios';

interface ApiClientOptions {
  baseURL: string;
  timeout?: number;
  // 401 未認証エラー時に呼び出されるコールバック（リダイレクト等の処理を呼び出し側で制御可能にする）
  onUnauthorized?: () => void;
}

// モジュールレベルの CSRF トークンストア（/auth/session レスポンスから取得した値を保持する）
let _csrfToken: string | null = null;

// CSRF トークンをプログラム的に設定する（BFF セッションレスポンスから取得した値を保持）
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

// CSRF トークンを取得する（プログラム的に設定されたトークンを優先し、なければ meta タグから取得）
function getCsrfToken(): string | null {
  if (_csrfToken) return _csrfToken;
  if (typeof document === 'undefined') return null;
  const meta = document.querySelector<HTMLMetaElement>('meta[name="csrf-token"]');
  return meta?.content ?? null;
}
