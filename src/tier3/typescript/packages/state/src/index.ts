// k1s0 tier3 state パッケージ public API
// 4 layer state reducer の全公開シンボルをまとめて re-export する

// 4 layer 型と lineage 型を公開する
export type {
  LayerId,
  Lineage,
  ServerTruthEntry,
  OptimisticLocalEntry,
  PendingQueueEntry,
  DraftEntry,
  PurgeReason,
  PurgeEvent,
} from "./layers.js";

// 5 event 型と BusinessConflict subtype 型を公開する
export type {
  ConflictEventId,
  ConflictEvent,
  ServerTruthAdvanceEvent,
  OptimisticAcknowledgedEvent,
  OptimisticRejectedEvent,
  PendingQueueResumeEvent,
  BusinessConflictReceivedEvent,
  BusinessConflictSubtype,
  FieldDiff,
} from "./events.js";

// subtype actions の解決関数を公開する（UI 分岐 override 禁止のため関数シグネチャを固定）
export { resolveSubtypeActions } from "./subtypes.js";

// 4 layer reducer と helper を公開する
// AutoResendWithChainedKeyAction を追加で export する（T3-4 typed action 公開）
export type {
  ClientState,
  ReducerResult,
  ReducerAction,
  AutoResendWithChainedKeyAction,
} from "./reducer.js";
export { createInitialState, reducePurge, reduce } from "./reducer.js";

// Idempotency-Key 形式定数・型・ヘルパー関数を公開する
// 04_状態管理.md §Idempotency-Key format: ULID + tenant_id + RPC method short hash
export type { IdempotencyKeyComponents, OverTtlPolicy } from "./idempotency.js";
export {
  IDEMPOTENCY_KEY_SEPARATOR,
  IDEMPOTENCY_KEY_METHOD_HASH_LENGTH,
  IDEMPOTENCY_KEY_TTL_MS,
  DEFAULT_OVER_TTL_POLICY,
  buildIdempotencyKey,
  chainIdempotencyKey,
  isIdempotencyKeyExpired,
} from "./idempotency.js";

// R3-5: client_purge_event emit 実装を公開する
// layers.yaml purge_triggers の audit_emit: client_purge_event に対する物理実装
export type { PurgeTrigger } from "./emitPurgeEvent.js";
export { emitClientPurgeEvent } from "./emitPurgeEvent.js";
