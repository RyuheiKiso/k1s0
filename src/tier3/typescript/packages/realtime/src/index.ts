// k1s0 tier3 realtime パッケージ public API
// SSE/WS subscription manager の全公開シンボルをまとめて re-export する

// subscription manager の全型と class を公開する
export type {
  SubscriptionConformanceClass,
  UaAdapterRoute,
  SubscriptionConfig,
  SubscriptionStatus,
} from "./subscription.js";
export { SubscriptionManager } from "./subscription.js";

// Transport インターフェースと関連型を公開する
export type {
  Transport,
  TransportOptions,
  TransportFactory,
  TransportReadyState,
} from "./transport.js";

// WebSocket アダプターを公開する
export { WebSocketAdapter, createWebSocketAdapter } from "./websocket_adapter.js";

// SSE アダプターを公開する
export { SseAdapter, createSseAdapter } from "./sse_adapter.js";

// UA aware route selector を公開する
export type { AdapterName, SelectTransportResult } from "./route_selector.js";
export { selectTransport } from "./route_selector.js";

// PresenceTracker / PresenceEntry を公開する（C-7 realtime 完全化）
// PresenceTracker は PresenceClient の canonical 別名（spec 11 §presence indicator）
export type { PresenceEntry } from "./presence.js";
export { PresenceTracker, PresenceClient } from "./presence.js";

// long-poll アダプターを公開する（C-7 long-poll adapter 実装）
// SSE + long-poll フォールバック / corp_proxy 対応
export { createLongPollAdapter, LongPollAdapter, detectCorpProxy } from "./long_poll_adapter.js";
