// k1s0 tier3 5 conflict event 型定義
// 11_クライアント状態適合仕様.md の v1 conflict event カタログを TypeScript 型として実装する
// event の追加は purely additive（新 event の追加で破壊的変更にならない）

// 5 conflict event の ID 型
export type ConflictEventId =
  // server_truth バージョン増加（domain_event 受信 or API 応答）
  | "server_truth_advance"
  // mutation in-flight の ack（api_response_2xx）
  | "optimistic_acknowledged"
  // mutation in-flight の reject（api_response_4xx）
  | "optimistic_rejected"
  // ネットワーク復帰 / アプリ再開（PQ 非空）
  | "pending_queue_resume"
  // api_response_409 + subtype（BusinessConflict）
  | "business_conflict_received";

// server_truth_advance event payload
export interface ServerTruthAdvanceEvent {
  // event ID
  readonly eventId: "server_truth_advance";
  // 新しい aggregate バージョン
  readonly newVersion: number;
  // HLC タイムスタンプ
  readonly hlcTimestamp: string;
}

// optimistic_acknowledged event payload
export interface OptimisticAcknowledgedEvent {
  // event ID
  readonly eventId: "optimistic_acknowledged";
  // 確定した idempotency_key
  readonly idempotencyKey: string;
  // 確定後の aggregate バージョン
  readonly confirmedVersion: number;
}

// optimistic_rejected event payload
export interface OptimisticRejectedEvent {
  // event ID
  readonly eventId: "optimistic_rejected";
  // reject された idempotency_key
  readonly idempotencyKey: string;
  // business error の種別（BusinessConflict の場合は subtype も含む）
  readonly errorCode: string;
  // BusinessConflict の場合の subtype
  readonly conflictSubtype?: BusinessConflictSubtype;
}

// pending_queue_resume event payload
export interface PendingQueueResumeEvent {
  // event ID
  readonly eventId: "pending_queue_resume";
  // 復帰理由
  readonly resumeReason: "network_recovery" | "app_resume";
}

// business_conflict_received event payload（subtype 分岐は reducer で処理）
export interface BusinessConflictReceivedEvent {
  // event ID
  readonly eventId: "business_conflict_received";
  // subtype（4 種）
  readonly subtype: BusinessConflictSubtype;
  // conflict が発生した aggregate ID
  readonly aggregateId: string;
  // server side の最新バージョン
  readonly serverVersion: number;
  // field-level diff（stale_write / lost_update 判定に使用）
  readonly fieldDiff?: FieldDiff;
}

// BusinessConflict subtype（4 種固定、追加は purely additive）
export type BusinessConflictSubtype =
  // field disjoint: field-level rebase で auto resend 可能
  | "stale_write"
  // field intersect: 3way merge UI 必要
  | "lost_update"
  // 同 actor の後続 op で同 aggregate が既に更新済み
  | "supersede"
  // 他 actor が presence で編集中
  | "concurrent_edit";

// FieldDiff: field-level diff（stale_write / lost_update 判定用）
export interface FieldDiff {
  // client が変更したフィールド集合
  readonly clientFields: readonly string[];
  // server が変更したフィールド集合
  readonly serverFields: readonly string[];
}

// 全 conflict event の union 型
export type ConflictEvent =
  | ServerTruthAdvanceEvent
  | OptimisticAcknowledgedEvent
  | OptimisticRejectedEvent
  | PendingQueueResumeEvent
  | BusinessConflictReceivedEvent;
