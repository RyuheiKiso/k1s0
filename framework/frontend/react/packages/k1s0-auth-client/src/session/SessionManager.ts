import type { SessionInfo, DeviceInfo } from '../types.js';

const SESSION_STORAGE_KEY = 'k1s0_session';

/**
 * UUID v4 を生成
 */
function generateSessionId(): string {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  // フォールバック
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * デバイス情報を検出
 */
function detectDeviceInfo(): DeviceInfo {
  if (typeof navigator === 'undefined') {
    return { deviceType: 'unknown' };
  }

  const ua = navigator.userAgent;
  let browser: string | undefined;
  let os: string | undefined;
  let deviceType: DeviceInfo['deviceType'] = 'desktop';

  // ブラウザ検出
  if (ua.includes('Firefox')) {
    browser = 'Firefox';
  } else if (ua.includes('Edge') || ua.includes('Edg')) {
    browser = 'Edge';
  } else if (ua.includes('Chrome')) {
    browser = 'Chrome';
  } else if (ua.includes('Safari')) {
    browser = 'Safari';
  }

  // OS検出
  if (ua.includes('Windows')) {
    os = 'Windows';
  } else if (ua.includes('Mac OS')) {
    os = 'macOS';
  } else if (ua.includes('Linux')) {
    os = 'Linux';
  } else if (ua.includes('Android')) {
    os = 'Android';
  } else if (ua.includes('iOS') || ua.includes('iPhone') || ua.includes('iPad')) {
    os = 'iOS';
  }

  // デバイスタイプ検出
  if (/Mobi|Android/i.test(ua)) {
    deviceType = 'mobile';
  } else if (/Tablet|iPad/i.test(ua)) {
    deviceType = 'tablet';
  }

  return { browser, os, deviceType };
}

/**
 * セッション管理設定
 */
export interface SessionManagerOptions {
  /** セッションの有効期間（ms）デフォルト: 24時間 */
  sessionDurationMs?: number;
  /** アイドルタイムアウト（ms）デフォルト: 30分 */
  idleTimeoutMs?: number;
  /** アクティビティ検知間隔（ms）デフォルト: 1分 */
  activityCheckIntervalMs?: number;
  /** セッションタイムアウト時のコールバック */
  onSessionTimeout?: () => void;
  /** アイドルタイムアウト時のコールバック */
  onIdleTimeout?: () => void;
}

/**
 * セッション管理クラス
 *
 * - セッションの開始/終了
 * - アクティビティトラッキング
 * - アイドルタイムアウト検知
 * - セッション情報の永続化
 */
export class SessionManager {
  private sessionDurationMs: number;
  private idleTimeoutMs: number;
  private activityCheckIntervalMs: number;
  private onSessionTimeout: (() => void) | undefined;
  private onIdleTimeout: (() => void) | undefined;

  private activityCheckTimer: ReturnType<typeof setInterval> | null = null;
  private session: SessionInfo | null = null;
  private isActive: boolean = true;

  constructor(options?: SessionManagerOptions) {
    this.sessionDurationMs = options?.sessionDurationMs ?? 24 * 60 * 60 * 1000; // 24時間
    this.idleTimeoutMs = options?.idleTimeoutMs ?? 30 * 60 * 1000; // 30分
    this.activityCheckIntervalMs = options?.activityCheckIntervalMs ?? 60 * 1000; // 1分
    this.onSessionTimeout = options?.onSessionTimeout;
    this.onIdleTimeout = options?.onIdleTimeout;
  }

  /**
   * セッションを開始
   */
  startSession(userId: string): SessionInfo {
    const now = Date.now();

    this.session = {
      id: generateSessionId(),
      startedAt: now,
      lastActiveAt: now,
      expiresAt: now + this.sessionDurationMs,
      userId,
      deviceInfo: detectDeviceInfo(),
    };

    this.saveSession();
    this.startActivityTracking();

    return this.session;
  }

  /**
   * セッションを終了
   */
  endSession(): void {
    this.stopActivityTracking();
    this.session = null;
    this.clearSavedSession();
  }

  /**
   * 保存されたセッションを復元
   */
  restoreSession(): SessionInfo | null {
    const saved = this.loadSavedSession();
    if (!saved) return null;

    const now = Date.now();

    // セッション有効期限チェック
    if (saved.expiresAt && now > saved.expiresAt) {
      this.clearSavedSession();
      this.onSessionTimeout?.();
      return null;
    }

    // アイドルタイムアウトチェック
    if (now - saved.lastActiveAt > this.idleTimeoutMs) {
      this.clearSavedSession();
      this.onIdleTimeout?.();
      return null;
    }

    this.session = saved;
    this.startActivityTracking();

    return this.session;
  }

  /**
   * 現在のセッションを取得
   */
  getSession(): SessionInfo | null {
    return this.session;
  }

  /**
   * セッションが有効かどうかを確認
   */
  isSessionValid(): boolean {
    if (!this.session) return false;

    const now = Date.now();

    // 有効期限チェック
    if (this.session.expiresAt && now > this.session.expiresAt) {
      return false;
    }

    // アイドルタイムアウトチェック
    if (now - this.session.lastActiveAt > this.idleTimeoutMs) {
      return false;
    }

    return true;
  }

  /**
   * アクティビティを記録
   */
  recordActivity(): void {
    if (!this.session) return;

    this.session.lastActiveAt = Date.now();
    this.isActive = true;
    this.saveSession();
  }

  /**
   * セッションを延長
   */
  extendSession(additionalMs?: number): void {
    if (!this.session) return;

    const extension = additionalMs ?? this.sessionDurationMs;
    this.session.expiresAt = Date.now() + extension;
    this.saveSession();
  }

  /**
   * リソースの解放
   */
  dispose(): void {
    this.stopActivityTracking();
    this.session = null;
  }

  /**
   * アクティビティトラッキングを開始
   */
  private startActivityTracking(): void {
    this.stopActivityTracking();

    if (typeof window === 'undefined') return;

    // ユーザーアクティビティイベントをリッスン
    const activityEvents = ['mousedown', 'keydown', 'touchstart', 'scroll'];
    const handleActivity = () => {
      this.recordActivity();
    };

    activityEvents.forEach((event) => {
      window.addEventListener(event, handleActivity, { passive: true });
    });

    // 定期的なチェック
    this.activityCheckTimer = setInterval(() => {
      this.checkSession();
    }, this.activityCheckIntervalMs);

    // クリーンアップ関数を保存
    (this as unknown as { _cleanupActivity: () => void })._cleanupActivity = () => {
      activityEvents.forEach((event) => {
        window.removeEventListener(event, handleActivity);
      });
    };
  }

  /**
   * アクティビティトラッキングを停止
   */
  private stopActivityTracking(): void {
    if (this.activityCheckTimer) {
      clearInterval(this.activityCheckTimer);
      this.activityCheckTimer = null;
    }

    const cleanup = (this as unknown as { _cleanupActivity?: () => void })
      ._cleanupActivity;
    if (cleanup) {
      cleanup();
    }
  }

  /**
   * セッション状態をチェック
   */
  private checkSession(): void {
    if (!this.session) return;

    const now = Date.now();

    // セッション有効期限チェック
    if (this.session.expiresAt && now > this.session.expiresAt) {
      this.endSession();
      this.onSessionTimeout?.();
      return;
    }

    // アイドルタイムアウトチェック
    if (now - this.session.lastActiveAt > this.idleTimeoutMs) {
      this.endSession();
      this.onIdleTimeout?.();
      return;
    }
  }

  /**
   * セッションを保存
   */
  private saveSession(): void {
    if (typeof sessionStorage === 'undefined' || !this.session) return;

    try {
      sessionStorage.setItem(SESSION_STORAGE_KEY, JSON.stringify(this.session));
    } catch {
      // ストレージ書き込み失敗時は無視
    }
  }

  /**
   * 保存されたセッションを読み込み
   */
  private loadSavedSession(): SessionInfo | null {
    if (typeof sessionStorage === 'undefined') return null;

    try {
      const stored = sessionStorage.getItem(SESSION_STORAGE_KEY);
      if (!stored) return null;
      return JSON.parse(stored) as SessionInfo;
    } catch {
      return null;
    }
  }

  /**
   * 保存されたセッションを削除
   */
  private clearSavedSession(): void {
    if (typeof sessionStorage === 'undefined') return;

    try {
      sessionStorage.removeItem(SESSION_STORAGE_KEY);
    } catch {
      // ストレージ削除失敗時は無視
    }
  }
}
