// system-client の createApiClient を使用して BFF の CSRF 契約に準拠する
// X-XSRF-TOKEN (Cookie) ではなく X-CSRF-Token (/auth/session JSON) を使用する
import { createApiClient, setCsrfToken } from '@k1s0/system-client';

// BFF /auth/session から取得した CSRF トークンを保持する
// setCsrfToken() で外部から注入する（AuthProvider が /auth/session 後に呼び出す）
export { setCsrfToken };

// BFF API クライアント: CSRF トークン管理と 401 リダイレクトを自動処理する
export const apiClient = createApiClient({
  baseURL: '/bff/api/v1',
  onUnauthorized: () => {
    window.location.href = '/login';
  },
});
