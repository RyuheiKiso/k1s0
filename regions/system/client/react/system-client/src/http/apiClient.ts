import axios, { type AxiosInstance, type AxiosError } from 'axios';

interface ApiClientOptions {
  baseURL: string;
  timeout?: number;
}

export function createApiClient({ baseURL, timeout = 30000 }: ApiClientOptions): AxiosInstance {
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

      if (status === 401) {
        if (typeof window !== 'undefined') {
          window.location.href = '/auth/login';
        }
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

function getCsrfToken(): string | null {
  if (typeof document === 'undefined') return null;
  const meta = document.querySelector<HTMLMetaElement>('meta[name="csrf-token"]');
  return meta?.content ?? null;
}
