// k1s0 tier3 outbox パッケージ public API
// WebCrypto + IndexedDB encrypted outbox の全公開シンボルをまとめて re-export する

// WebCrypto AES-GCM ラッパーを公開する
export {
  generateDeviceBoundKey,
  encryptData,
  decryptData,
  encryptJson,
  decryptJson,
} from "./crypto.js";

// IndexedDB encrypted outbox を公開する
export type {
  PiiStripped,
  OutboxEntryMeta,
  OverTtlPolicy,
} from "./outbox.js";
export {
  IDEMPOTENCY_KEY_TTL_MS,
  // HLC タイムスタンプ生成関数（wall-clock TTL 禁止規約に従い HLC を提供する）
  hlcNow,
  isExpired,
  generateIdempotencyKey,
  chainIdempotencyKey,
  createOutboxMeta,
  stripPiiFields,
} from "./outbox.js";

// IndexedDB encrypted store（低水準 API）を公開する
export type { IndexedDbStore, PersistLayer } from "./indexeddb_store.js";
export { createStore } from "./indexeddb_store.js";

// IndexedDB persistence 高水準 API を公開する
export {
  persist,
  restore,
  persistPendingQueueEntry,
  persistDraftEntry,
  restorePendingQueue,
  restoreDraft,
  clearLayer,
  purgeAllLayers,
  _resetStoreForTest,
} from "./persistence.js";
