import axios, { type AxiosInstance } from 'axios';

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

  return client;
}

function getCsrfToken(): string | null {
  if (typeof document === 'undefined') return null;
  const meta = document.querySelector<HTMLMetaElement>('meta[name="csrf-token"]');
  return meta?.content ?? null;
}
