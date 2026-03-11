import { createContext, useContext } from 'react';
import type { AuthSessionSummary } from './tauri-commands';

export type AuthSession = AuthSessionSummary;

export interface AuthContextValue {
  session: AuthSession | null;
  isAuthenticated: boolean;
  loading: boolean;
  refreshSession: () => Promise<AuthSession | null>;
  setSession: (session: AuthSession | null) => void;
  clearSession: () => Promise<void>;
}

const defaultContextValue: AuthContextValue = {
  session: null,
  isAuthenticated: false,
  loading: false,
  refreshSession: async () => null,
  setSession: () => undefined,
  clearSession: async () => undefined,
};

export const AuthContext = createContext<AuthContextValue>(defaultContextValue);

export function useAuth() {
  return useContext(AuthContext);
}
