import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { SessionManager, type SessionManagerOptions } from './SessionManager.js';
import type { SessionInfo } from '../types.js';
import { useAuth } from '../provider/AuthContext.js';

/**
 * セッション管理フックの戻り値
 */
export interface UseSessionResult {
  /** 現在のセッション情報 */
  session: SessionInfo | null;
  /** セッションが有効かどうか */
  isSessionValid: boolean;
  /** セッションを開始 */
  startSession: () => SessionInfo | null;
  /** セッションを終了 */
  endSession: () => void;
  /** セッションを延長 */
  extendSession: (additionalMs?: number) => void;
  /** アクティビティを記録 */
  recordActivity: () => void;
}

/**
 * セッション管理フック
 *
 * 認証状態と連動してセッションを管理する
 */
export function useSession(options?: SessionManagerOptions): UseSessionResult {
  const { state, logout } = useAuth();
  const [session, setSession] = useState<SessionInfo | null>(null);

  // SessionManager は一度だけ作成
  const sessionManagerRef = useRef<SessionManager | null>(null);
  if (!sessionManagerRef.current) {
    sessionManagerRef.current = new SessionManager({
      ...options,
      onSessionTimeout: () => {
        options?.onSessionTimeout?.();
        logout();
      },
      onIdleTimeout: () => {
        options?.onIdleTimeout?.();
        logout();
      },
    });
  }
  const sessionManager = sessionManagerRef.current;

  // 認証状態に応じてセッションを管理
  useEffect(() => {
    if (state.isAuthenticated && state.user) {
      // 既存セッションの復元を試みる
      const restored = sessionManager.restoreSession();
      if (restored) {
        setSession(restored);
      } else {
        // 新しいセッションを開始
        const newSession = sessionManager.startSession(state.user.id);
        setSession(newSession);
      }
    } else {
      // ログアウト時はセッションを終了
      sessionManager.endSession();
      setSession(null);
    }
  }, [state.isAuthenticated, state.user, sessionManager]);

  // クリーンアップ
  useEffect(() => {
    return () => {
      sessionManager.dispose();
    };
  }, [sessionManager]);

  const startSession = useCallback((): SessionInfo | null => {
    if (!state.user) return null;
    const newSession = sessionManager.startSession(state.user.id);
    setSession(newSession);
    return newSession;
  }, [state.user, sessionManager]);

  const endSession = useCallback(() => {
    sessionManager.endSession();
    setSession(null);
  }, [sessionManager]);

  const extendSession = useCallback(
    (additionalMs?: number) => {
      sessionManager.extendSession(additionalMs);
      setSession(sessionManager.getSession());
    },
    [sessionManager]
  );

  const recordActivity = useCallback(() => {
    sessionManager.recordActivity();
    setSession(sessionManager.getSession());
  }, [sessionManager]);

  const isSessionValid = useMemo(() => {
    return sessionManager.isSessionValid();
  }, [session, sessionManager]);

  return {
    session,
    isSessionValid,
    startSession,
    endSession,
    extendSession,
    recordActivity,
  };
}
