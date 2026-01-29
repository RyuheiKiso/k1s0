/**
 * リアルタイム通信の共通型定義
 */

/** 接続状態 */
export type ConnectionStatus = 'connecting' | 'connected' | 'disconnecting' | 'disconnected';

/** 再接続設定 */
export interface ReconnectConfig {
  /** 再接続を有効にする */
  enabled: boolean;
  /** 最大再接続回数（0 = 無限） */
  maxAttempts: number;
  /** バックオフ戦略 */
  backoff: 'linear' | 'exponential';
  /** 初回遅延（ms） */
  initialDelay: number;
  /** 最大遅延（ms） */
  maxDelay: number;
  /** ランダム揺らぎの追加 */
  jitter: boolean;
}

/** ハートビート設定 */
export interface HeartbeatConfig {
  /** ハートビートを有効にする */
  enabled: boolean;
  /** 送信間隔（ms） */
  interval: number;
  /** 応答タイムアウト（ms） */
  timeout: number;
  /** 送信メッセージ */
  message: string | (() => string);
  /** 期待するレスポンスの判定 */
  expectedResponse?: string | ((msg: unknown) => boolean);
}

/** WebSocket フックオプション */
export interface UseWebSocketOptions<T = unknown> {
  /** 接続先 URL */
  url: string;
  /** WebSocket サブプロトコル */
  protocols?: string | string[];

  /** 再接続設定 */
  reconnect?: Partial<ReconnectConfig>;

  /** ハートビート設定 */
  heartbeat?: Partial<HeartbeatConfig>;

  /** 認証トークン取得関数 */
  getAuthToken?: () => string | Promise<string>;

  /** 送信データのシリアライズ */
  serialize?: (data: unknown) => string;
  /** 受信データのデシリアライズ */
  deserialize?: (data: string) => T;

  /** 接続開始時のコールバック */
  onOpen?: (event: Event) => void;
  /** 接続切断時のコールバック */
  onClose?: (event: CloseEvent) => void;
  /** エラー発生時のコールバック */
  onError?: (event: Event) => void;
  /** メッセージ受信時のコールバック */
  onMessage?: (data: T) => void;
  /** 再接続試行時のコールバック */
  onReconnecting?: (attempt: number) => void;
  /** 再接続成功時のコールバック */
  onReconnected?: () => void;

  /** 自動接続（デフォルト: true） */
  autoConnect?: boolean;
}

/** WebSocket フック戻り値 */
export interface UseWebSocketReturn<T = unknown> {
  /** 接続状態 */
  status: ConnectionStatus;
  /** 最後に受信したメッセージ */
  lastMessage: T | null;
  /** エラー */
  error: Error | null;
  /** 再接続試行回数 */
  reconnectAttempt: number;

  /** 接続 */
  connect: () => void;
  /** 切断 */
  disconnect: (code?: number, reason?: string) => void;
  /** メッセージ送信 */
  sendMessage: (data: unknown) => void;
  /** JSON メッセージ送信 */
  sendJson: (data: object) => void;

  /** WebSocket インスタンス取得 */
  getSocket: () => WebSocket | null;
}
