import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import {
  clearAuthSession as clearStoredAuthSession,
  getAuthSession,
  type AuthSessionSummary,
} from './tauri-commands';

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

export function AuthProvider({ children }: { children: ReactNode }) {
  const [session, setSessionState] = useState<AuthSession | null>(null);
  const [loading, setLoading] = useState(true);

  async function refreshSession() {
    setLoading(true);
    try {
      const nextSession = await getAuthSession();
      setSessionState(nextSession);
      return nextSession;
    } catch {
      setSessionState(null);
      return null;
    } finally {
      setLoading(false);
    }
  }

  function setSession(nextSession: AuthSession | null) {
    setSessionState(nextSession);
  }

  async function clearSession() {
    setLoading(true);
    try {
      await clearStoredAuthSession();
      setSessionState(null);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    let active = true;

    void getAuthSession()
      .then((nextSession) => {
        if (active) {
          setSessionState(nextSession);
        }
      })
      .catch(() => {
        if (active) {
          setSessionState(null);
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!session) {
      return;
    }

    const delayMs = Math.min(
      Math.max(session.expires_at_epoch_secs * 1000 - Date.now() + 1000, 1000),
      2_147_483_647,
    );
    const timerId = window.setTimeout(() => {
      void refreshSession();
    }, delayMs);

    return () => {
      window.clearTimeout(timerId);
    };
  }, [session]);

  return (
    <AuthContext.Provider
      value={{
        session,
        isAuthenticated: session !== null,
        loading,
        refreshSession,
        setSession,
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
