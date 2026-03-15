import { createContext } from 'react';

export interface User {
  id: string;
  username: string;
  roles?: string[];
}

export interface AuthContextValue {
  user: User | null;
  isAuthenticated: boolean;
  // BFF の OAuth2/OIDC 認可コードフローへリダイレクトする（引数なし）
  login: () => void;
  logout: () => Promise<void>;
}

export const AuthContext = createContext<AuthContextValue | null>(null);
