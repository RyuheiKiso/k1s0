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
