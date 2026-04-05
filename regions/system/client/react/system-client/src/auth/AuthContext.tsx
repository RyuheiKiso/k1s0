import { createContext } from 'react';

export interface User {
  id: string;
  username: string;
  roles?: string[];
}

export interface AuthContextValue {
  user: User | null;
  isAuthenticated: boolean;
  // M-009 監査対応: セッション確認中のローディング状態を公開する
  // ProtectedRoute が fallback を一瞬フラッシュしないようにするために使用する
  loading: boolean;
  // BFF の OAuth2/OIDC 認可コードフローへリダイレクトする（引数なし）
  login: () => void;
  logout: () => Promise<void>;
}

export const AuthContext = createContext<AuthContextValue | null>(null);
