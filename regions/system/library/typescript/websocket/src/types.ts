export type MessageType = 'text' | 'binary' | 'ping' | 'pong' | 'close';

export interface WsMessage {
  type: MessageType;
  payload: string | Uint8Array;
}

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting' | 'closing';

export interface WsConfig {
  url: string;
  reconnect: boolean;
  maxReconnectAttempts: number;
  reconnectDelayMs: number;
  pingIntervalMs?: number;
}

/// WebSocket 設定を生成する。URL は必須。
export function createConfig(url: string, overrides?: Partial<Omit<WsConfig, 'url'>>): WsConfig {
  return {
    url,
    reconnect: overrides?.reconnect ?? true,
    maxReconnectAttempts: overrides?.maxReconnectAttempts ?? 5,
    reconnectDelayMs: overrides?.reconnectDelayMs ?? 1000,
    pingIntervalMs: overrides?.pingIntervalMs,
  };
}
