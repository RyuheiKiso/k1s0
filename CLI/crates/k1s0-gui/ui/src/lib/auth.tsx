import { createContext, useContext, useState, type ReactNode } from 'react';
import type { AuthTokens } from './tauri-commands';

const STORAGE_KEY = 'k1s0.authSession';

export interface AuthSession {
  issuer: string;
  authenticatedAt: string;
  tokens: AuthTokens;
}

interface AuthContextValue {
  session: AuthSession | null;
  isAuthenticated: boolean;
  saveSession: (issuer: string, tokens: AuthTokens) => void;
  clearSession: () => void;
}

const defaultContextValue: AuthContextValue = {
  session: null,
  isAuthenticated: false,
  saveSession: () => undefined,
  clearSession: () => undefined,
};

const AuthContext = createContext<AuthContextValue>(defaultContextValue);

function loadStoredSession(): AuthSession | null {
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw) as AuthSession;
  } catch {
    window.localStorage.removeItem(STORAGE_KEY);
    return null;
  }
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<AuthSession | null>(() => loadStoredSession());

  function saveSession(issuer: string, tokens: AuthTokens) {
    const nextSession: AuthSession = {
      issuer,
      authenticatedAt: new Date().toISOString(),
      tokens,
    };
    setSession(nextSession);
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(nextSession));
  }

  function clearSession() {
    setSession(null);
    window.localStorage.removeItem(STORAGE_KEY);
  }

  return (
    <AuthContext.Provider
      value={{
        session,
        isAuthenticated: session !== null,
        saveSession,
        clearSession,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
