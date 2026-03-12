import { createContext } from 'react';

export interface User {
  id: string;
  username: string;
  roles?: string[];
}

export interface AuthContextValue {
  user: User | null;
  isAuthenticated: boolean;
  login: (credentials: { username: string; password: string }) => Promise<void>;
  logout: () => Promise<void>;
}

export const AuthContext = createContext<AuthContextValue | null>(null);
