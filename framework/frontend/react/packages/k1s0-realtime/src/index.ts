// WebSocket
export {
  useWebSocket,
  WebSocketClient,
  ReconnectHandler,
  DEFAULT_RECONNECT_CONFIG,
  HeartbeatHandler,
  DEFAULT_HEARTBEAT_CONFIG,
} from './websocket/index.js';

// SSE
export {
  useSSE,
  SSEClient,
  type SSEEvent,
  type UseSSEOptions,
  type UseSSEReturn,
} from './sse/index.js';

// Provider
export {
  RealtimeProvider,
  RealtimeContext,
  useConnectionStatus,
  useOfflineQueue,
  type RealtimeConfig,
  type RealtimeContextValue,
  type ConnectionInfo,
  type OfflineQueueConfig,
} from './provider/index.js';

// Types
export type {
  ConnectionStatus,
  ReconnectConfig,
  HeartbeatConfig,
  UseWebSocketOptions,
  UseWebSocketReturn,
} from './types.js';
