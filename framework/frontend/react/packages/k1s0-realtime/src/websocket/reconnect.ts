/**
 * WebSocket 再接続ハンドラ
 */

import type { ReconnectConfig } from '../types.js';
import { calculateBackoff, addJitter } from '../utils/backoff.js';

/** デフォルト再接続設定 */
export const DEFAULT_RECONNECT_CONFIG: ReconnectConfig = {
  enabled: true,
  maxAttempts: 10,
  backoff: 'exponential',
  initialDelay: 1000,
  maxDelay: 30000,
  jitter: true,
};

/**
 * 再接続を管理するクラス
 */
export class ReconnectHandler {
  private config: ReconnectConfig;
  private attempt = 0;
  private timerId: ReturnType<typeof setTimeout> | null = null;
  private stopped = false;

  constructor(config: Partial<ReconnectConfig> = {}) {
    this.config = { ...DEFAULT_RECONNECT_CONFIG, ...config };
  }

  /** 現在の試行回数 */
  getAttempt(): number {
    return this.attempt;
  }

  /**
   * 再接続をスケジュールする
   * @param onReconnect - 再接続実行時のコールバック
   * @param onAttempt - 試行開始時のコールバック
   * @returns スケジュールされた場合 true
   */
  schedule(onReconnect: () => void, onAttempt?: (attempt: number) => void): boolean {
    if (!this.config.enabled || this.stopped) return false;
    if (this.config.maxAttempts > 0 && this.attempt >= this.config.maxAttempts) return false;

    this.attempt++;
    let delay = calculateBackoff(
      this.attempt - 1,
      this.config.initialDelay,
      this.config.maxDelay,
      this.config.backoff,
    );

    if (this.config.jitter) {
      delay = addJitter(delay);
    }

    onAttempt?.(this.attempt);

    this.timerId = setTimeout(() => {
      this.timerId = null;
      if (!this.stopped) {
        onReconnect();
      }
    }, delay);

    return true;
  }

  /** 試行回数をリセットする */
  reset(): void {
    this.attempt = 0;
    this.cancel();
  }

  /** スケジュール済みの再接続をキャンセルする */
  cancel(): void {
    if (this.timerId !== null) {
      clearTimeout(this.timerId);
      this.timerId = null;
    }
  }

  /** 再接続を完全に停止する */
  stop(): void {
    this.stopped = true;
    this.cancel();
  }

  /** 停止状態を解除する */
  restart(): void {
    this.stopped = false;
    this.attempt = 0;
  }
}
