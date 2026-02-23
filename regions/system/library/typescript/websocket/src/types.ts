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

export function defaultConfig(): WsConfig {
  return {
    url: 'ws://localhost',
    reconnect: true,
    maxReconnectAttempts: 5,
    reconnectDelayMs: 1000,
  };
}
