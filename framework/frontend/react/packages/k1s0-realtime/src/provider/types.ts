/**
 * Realtime Provider の型定義
 */

import type { ConnectionStatus, ReconnectConfig, HeartbeatConfig } from '../types.js';

/** 接続情報 */
export interface ConnectionInfo {
  /** 接続 ID */
  id: string;
  /** 接続状態 */
  status: ConnectionStatus;
  /** 再接続試行回数 */
  reconnectAttempt: number;
  /** 接続日時 */
  connectedAt?: Date;
  /** 切断日時 */
  disconnectedAt?: Date;
}

/** オフラインキュー設定 */
export interface OfflineQueueConfig {
  /** キューを有効にする */
  enabled: boolean;
  /** 最大キューサイズ */
  maxSize: number;
  /** localStorage に永続化する */
  persistToStorage: boolean;
  /** ストレージキー */
  storageKey?: string;
}

/** Realtime 設定 */
export interface RealtimeConfig {
  /** ネットワーク監視設定 */
  networkMonitor?: {
    enabled: boolean;
    onOnline?: () => void;
    onOffline?: () => void;
  };

  /** オフラインキュー設定 */
  offlineQueue?: Partial<OfflineQueueConfig>;

  /** デフォルト再接続設定 */
  defaultReconnect?: Partial<ReconnectConfig>;

  /** デフォルトハートビート設定 */
  defaultHeartbeat?: Partial<HeartbeatConfig>;
}

/** Realtime Context の値 */
export interface RealtimeContextValue {
  /** ネットワークのオンライン状態 */
  isOnline: boolean;

  /** 登録済み接続の一覧 */
  connections: Map<string, ConnectionInfo>;
  /** 接続を登録する */
  registerConnection: (id: string, info: ConnectionInfo) => void;
  /** 接続を解除する */
  unregisterConnection: (id: string) => void;

  /** オフラインキューにアイテムを追加する */
  queue: <T>(connectionId: string, item: T) => void;
  /** キューをフラッシュする */
  flush: (connectionId: string, send: (item: unknown) => void) => void;
  /** キュー内のアイテムを取得する */
  getQueuedItems: (connectionId: string) => unknown[];
  /** キューをクリアする */
  clearQueue: (connectionId: string) => void;
}
