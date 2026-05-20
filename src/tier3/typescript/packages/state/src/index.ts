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
