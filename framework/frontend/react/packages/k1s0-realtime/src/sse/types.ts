/**
 * SSE（Server-Sent Events）の型定義
 */

import type { ConnectionStatus, ReconnectConfig } from '../types.js';

/** SSE イベント */
export interface SSEEvent<T = unknown> {
  /** イベントタイプ */
  type: string;
  /** イベントデータ */
  data: T;
}

/** SSE フックオプション */
export interface UseSSEOptions<T = unknown> {
  /** 接続先 URL */
  url: string;

  /** クレデンシャル送信 */
  withCredentials?: boolean;

  /** イベントタイプ別ハンドラ */
  eventHandlers?: Record<string, (data: T) => void>;

  /** デフォルトメッセージハンドラ */
  onMessage?: (data: T) => void;

  /** エラーハンドラ */
  onError?: (event: Event) => void;

  /** 再接続設定（EventSource 標準機能を拡張） */
  reconnect?: {
    enabled: boolean;
    maxAttempts?: number;
    interval?: number;
  };

  /** デシリアライズ関数 */
  deserialize?: (data: string) => T;

  /** 自動接続（デフォルト: true） */
  autoConnect?: boolean;
}

/** SSE フック戻り値 */
export interface UseSSEReturn<T = unknown> {
  /** 接続状態 */
  status: ConnectionStatus;
  /** 最後に受信したイベント */
  lastEvent: SSEEvent<T> | null;
  /** エラー */
  error: Error | null;

  /** 接続 */
  connect: () => void;
  /** 切断 */
  disconnect: () => void;
}
