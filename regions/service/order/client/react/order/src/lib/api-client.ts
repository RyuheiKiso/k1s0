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
