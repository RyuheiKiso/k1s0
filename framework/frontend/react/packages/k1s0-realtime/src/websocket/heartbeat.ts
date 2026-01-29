/**
 * WebSocket ハートビートハンドラ
 */

import type { HeartbeatConfig } from '../types.js';

/** デフォルトハートビート設定 */
export const DEFAULT_HEARTBEAT_CONFIG: HeartbeatConfig = {
  enabled: true,
  interval: 30000,
  timeout: 5000,
  message: '{"type":"ping"}',
  expectedResponse: undefined,
};

/**
 * ハートビート（ping/pong）を管理するクラス
 */
export class HeartbeatHandler {
  private config: HeartbeatConfig;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private pongTimer: ReturnType<typeof setTimeout> | null = null;
  private onTimeout: (() => void) | null = null;
  private sendFn: ((data: string) => void) | null = null;

  constructor(config: Partial<HeartbeatConfig> = {}) {
    this.config = { ...DEFAULT_HEARTBEAT_CONFIG, ...config };
  }

  /**
   * ハートビートを開始する
   * @param send - メッセージ送信関数
   * @param onTimeout - タイムアウト時のコールバック（接続切断トリガー）
   */
  start(send: (data: string) => void, onTimeout: () => void): void {
    if (!this.config.enabled) return;

    this.sendFn = send;
    this.onTimeout = onTimeout;
    this.stop();

    this.pingTimer = setInterval(() => {
      this.sendPing();
    }, this.config.interval);
  }

  /** ハートビートを停止する */
  stop(): void {
    if (this.pingTimer !== null) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
    this.clearPongTimer();
  }

  /**
   * 受信メッセージがハートビート応答かを判定する
   * @param data - 受信データ
   * @returns ハートビート応答の場合 true
   */
  handleMessage(data: unknown): boolean {
    if (!this.config.expectedResponse) {
      // expectedResponse 未設定の場合、全メッセージを pong として扱う
      this.clearPongTimer();
      return false;
    }

    if (typeof this.config.expectedResponse === 'string') {
      if (data === this.config.expectedResponse) {
        this.clearPongTimer();
        return true;
      }
    } else if (this.config.expectedResponse(data)) {
      this.clearPongTimer();
      return true;
    }

    return false;
  }

  private sendPing(): void {
    if (!this.sendFn) return;

    const message = typeof this.config.message === 'function'
      ? this.config.message()
      : this.config.message;

    try {
      this.sendFn(message);
    } catch {
      // 送信失敗は無視（接続切断で検知される）
      return;
    }

    this.pongTimer = setTimeout(() => {
      this.pongTimer = null;
      this.onTimeout?.();
    }, this.config.timeout);
  }

  private clearPongTimer(): void {
    if (this.pongTimer !== null) {
      clearTimeout(this.pongTimer);
      this.pongTimer = null;
    }
  }
}
