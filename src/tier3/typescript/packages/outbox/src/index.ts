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
  isExpired,
  generateIdempotencyKey,
  chainIdempotencyKey,
  createOutboxMeta,
  stripPiiFields,
} from "./outbox.js";
