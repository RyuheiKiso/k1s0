import axios from 'axios';

// BFF経由でAPIにアクセスするaxiosインスタンス
// HttpOnly Cookieを使用するため withCredentials: true を設定
export const apiClient = axios.create({
  baseURL: '/bff/api/v1',
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
  },
});

// CookieからXSRF-TOKENを取得するヘルパー関数
// サーバーが Set-Cookie で XSRF-TOKEN を設定している前提
function getCsrfToken(): string | null {
  const match = document.cookie.match(/(?:^|;\s*)XSRF-TOKEN=([^;]*)/);
  return match ? decodeURIComponent(match[1]) : null;
}

// CSRF対策: リクエスト送信前にCookieからCSRFトークンを読み取りヘッダに付与する
// これによりサーバー側でDouble Submit Cookie パターンによるCSRF検証が可能になる
apiClient.interceptors.request.use((config) => {
  const token = getCsrfToken();
  if (token) {
    config.headers['X-XSRF-TOKEN'] = token;
  }
  return config;
});

// 認証エラーハンドリング: 401レスポンス時にログインページへリダイレクトする
// セッション切れやトークン失効時にユーザーを適切に誘導する
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);
