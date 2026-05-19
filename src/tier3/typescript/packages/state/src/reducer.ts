// k1s0 tier3 4 layer state reducer
// 11_クライアント状態適合仕様.md の 5 event × 4 subtype × 4 layer を実装する
// reducer は決定論的（同 event に対し常に同じ actions を返す）

import type { ConflictEvent } from "./events.js";
import type { PendingQueueEntry, ServerTruthEntry, OptimisticLocalEntry, PurgeEvent, PurgeReason } from "./layers.js";
import { resolveSubtypeActions } from "./subtypes.js";
import type { AnySubtypeAction } from "./subtypes.js";
// PII strip を outbox パッケージから import する（PQ enqueue 前に PII を除去するため）
// @k1s0/tier3-outbox は pnpm workspace:* で state パッケージと連携する
import { stripPiiFields } from "@k1s0/tier3-outbox";

// 4 layer state の集合体
export interface ClientState<T, TPayload = unknown> {
  // server_truth layer: tier2 確定値（in-memory のみ）
  readonly serverTruth: ServerTruthEntry<T> | null;
  // optimistic_local layer: in-flight overlay（in-memory のみ）
  readonly optimisticLocal: OptimisticLocalEntry<T> | null;
  // pending_queue layer: offline 永続化 mutation 経路
  readonly pendingQueue: readonly PendingQueueEntry<TPayload>[];
  // 現在 queue hold 中か否か
  readonly queueHeld: boolean;
}

// reducer の結果（actions と次の state）
export interface ReducerResult<T, TPayload = unknown> {
  // 次の client state
  readonly nextState: ClientState<T, TPayload>;
  // 実行すべき副作用 actions（Adapter 層が実行する）
  readonly actions: readonly ReducerAction[];
}

// auto_resend_with_chained_key の型付きアクション（T3-4: string ではなく明示的型で表現する）
export interface AutoResendWithChainedKeyAction {
  // アクション種別（文字列リテラル型で固定する）
  readonly subtypeAction: "auto_resend_with_chained_key";
  // chain 元の idempotency_key（rebase 前の key）
  readonly chainedFrom: string;
  // chain 後の新しい idempotency_key
  readonly newKey: string;
}

// reducer から返される副作用 actions 型
export type ReducerAction =
  // server_truth を更新する
  | { readonly type: "UPDATE_SERVER_TRUTH"; readonly version: number }
  // optimistic local を rollback する
  | { readonly type: "ROLLBACK_OPTIMISTIC" }
  // pending queue を再評価する
  | { readonly type: "RE_EVALUATE_PENDING_QUEUE" }
  // optimistic を server_truth に promote する
  | { readonly type: "PROMOTE_OPTIMISTIC"; readonly idempotencyKey: string }
  // pending queue entry を削除する
  | { readonly type: "DELETE_PQ_ENTRY"; readonly idempotencyKey: string }
  // queue を in-order で送信する
  | { readonly type: "SEND_QUEUE_IN_ORDER" }
  // business error を表示する
  | { readonly type: "PRESENT_BUSINESS_ERROR"; readonly errorCode: string }
  // BusinessConflict subtype action を実行する（subtypeAction は typed union で必ず "auto_resend_with_chained_key" を含む）
  | { readonly type: "DISPATCH_CONFLICT_SUBTYPE"; readonly subtypeAction: string; readonly detail: unknown }
  // auto_resend_with_chained_key の型付きアクション（T3-4: 明示的型付きバリアント）
  | { readonly type: "AUTO_RESEND_WITH_CHAINED_KEY"; readonly action: AutoResendWithChainedKeyAction }
  // 3way merge UI を表示する
  | { readonly type: "PRESENT_3WAY_MERGE_UI" }
  // silent toast を表示する
  | { readonly type: "NOTIFY_SILENT_TOAST"; readonly message: string }
  // queue を hold する
  | { readonly type: "HOLD_QUEUE" }
  // presence indicator を更新する
  | { readonly type: "UPDATE_PRESENCE"; readonly actorId: string }
  // 全 layer を purge する（5 trigger）
  | { readonly type: "PURGE_ALL_LAYERS"; readonly reason: PurgeReason };

// 空の初期 state を生成する
export function createInitialState<T, TPayload = unknown>(): ClientState<T, TPayload> {
  // 全 layer を null / 空で初期化する
  return {
    serverTruth: null,
    optimisticLocal: null,
    pendingQueue: [],
    queueHeld: false,
  };
}

// PQ enqueue のユーティリティ: PII strip を適用してから PendingQueueEntry を追加する
// T3-3: enqueue 前に必ず stripPiiFields を呼び出すことを強制する
export function enqueuePendingQueue<TPayload extends Record<string, unknown>>(
  queue: readonly PendingQueueEntry<TPayload>[],
  entry: PendingQueueEntry<TPayload>,
  piiFieldNames: readonly string[],
): readonly PendingQueueEntry<TPayload>[] {
  // PII strip を実行してから payload を差し替える
  const strippedPayload = stripPiiFields(
    // entry の payload を PII strip の対象とする
    entry.payload as Record<string, unknown>,
    // PII フィールド名リスト
    piiFieldNames,
  ) as TPayload;
  // PII strip 済み payload で entry を差し替えた新しいエントリを生成する
  const strippedEntry: PendingQueueEntry<TPayload> = {
    // layer 識別子はそのまま引き継ぐ
    layer: entry.layer,
    // PII strip 済み payload を設定する
    payload: strippedPayload,
    // lineage はそのまま引き継ぐ
    lineage: entry.lineage,
  };
  // PII strip 済みエントリを queue の末尾に追加して返す
  return [...queue, strippedEntry];
}

// 全 layer purge（5 trigger: logout / refresh_token_expiry / tenant_switch / actor_switch / device_bound_key_rotate）
export function reducePurge<T, TPayload>(
  _state: ClientState<T, TPayload>,
  event: PurgeEvent,
): ReducerResult<T, TPayload> {
  // 全 layer を purge して初期 state に戻す
  return {
    nextState: createInitialState<T, TPayload>(),
    actions: [{ type: "PURGE_ALL_LAYERS", reason: event.reason }],
  };
}

// 5 event を処理する決定論的 reducer
export function reduce<T, TPayload>(
  state: ClientState<T, TPayload>,
  event: ConflictEvent,
): ReducerResult<T, TPayload> {
  // event 種別に応じて reducer を分岐する
  switch (event.eventId) {
    case "server_truth_advance":
      // server_truth_advance: ST 更新 + OL rollback + PQ 再評価
      return reduceServerTruthAdvance(state, event.newVersion);

    case "optimistic_acknowledged":
      // optimistic_acknowledged: OL → ST promote + PQ entry 削除
      return reduceOptimisticAcknowledged(state, event.idempotencyKey, event.confirmedVersion);

    case "optimistic_rejected":
      // optimistic_rejected: OL rollback + business error 表示
      return reduceOptimisticRejected(state, event.idempotencyKey, event.errorCode, event.conflictSubtype);

    case "pending_queue_resume":
      // pending_queue_resume: PQ を in-order で送信する
      return reducePendingQueueResume(state);

    case "business_conflict_received":
      // business_conflict_received: subtype に応じて決定論的に actions を dispatch する
      return reduceBusinessConflict(state, event.subtype, event.aggregateId, event.fieldDiff);

    default: {
      // 網羅性チェック（新 event 追加時はコンパイルエラー）
      const _exhaustive: never = event;
      throw new Error(`Unknown event: ${String(_exhaustive)}`);
    }
  }
}

// server_truth_advance の reducer
function reduceServerTruthAdvance<T, TPayload>(
  state: ClientState<T, TPayload>,
  newVersion: number,
): ReducerResult<T, TPayload> {
  // OL が存在する場合は rollback する
  const nextState: ClientState<T, TPayload> = {
    ...state,
    optimisticLocal: null,
  };
  const actions: ReducerAction[] = [
    { type: "UPDATE_SERVER_TRUTH", version: newVersion },
  ];
  // OL が存在した場合のみ rollback action を追加する
  if (state.optimisticLocal !== null) {
    actions.push({ type: "ROLLBACK_OPTIMISTIC" });
  }
  // PQ 再評価
  actions.push({ type: "RE_EVALUATE_PENDING_QUEUE" });
  return { nextState, actions };
}

// optimistic_acknowledged の reducer
// Rust の 2 action: promote_optimistic_to_server_truth + delete_pending_queue_entry_by_idempotency_key のみを発行する
function reduceOptimisticAcknowledged<T, TPayload>(
  state: ClientState<T, TPayload>,
  idempotencyKey: string,
  _confirmedVersion: number,
): ReducerResult<T, TPayload> {
  // OL を promote して ST に格納し、対応する PQ entry を削除する
  const nextState: ClientState<T, TPayload> = {
    ...state,
    optimisticLocal: null,
    pendingQueue: state.pendingQueue.filter(
      (pq) => pq.lineage.idempotencyKey !== idempotencyKey,
    ),
  };
  // Rust 等価: promote_optimistic_to_server_truth + delete_pending_queue_entry_by_idempotency_key の 2 action のみ
  return {
    nextState,
    actions: [
      { type: "PROMOTE_OPTIMISTIC", idempotencyKey },
      { type: "DELETE_PQ_ENTRY", idempotencyKey },
    ],
  };
}

// optimistic_rejected の reducer
function reduceOptimisticRejected<T, TPayload>(
  state: ClientState<T, TPayload>,
  idempotencyKey: string,
  errorCode: string,
  conflictSubtype: import("./events.js").BusinessConflictSubtype | undefined,
): ReducerResult<T, TPayload> {
  // OL を rollback する
  const nextState: ClientState<T, TPayload> = {
    ...state,
    optimisticLocal: null,
  };
  const actions: ReducerAction[] = [
    { type: "ROLLBACK_OPTIMISTIC" },
    { type: "PRESENT_BUSINESS_ERROR", errorCode },
  ];
  // BusinessConflict の場合は subtype dispatch を追加する
  if (conflictSubtype !== undefined) {
    const subtypeActions = resolveSubtypeActions(conflictSubtype, idempotencyKey, undefined, undefined);
    for (const sa of subtypeActions) {
      // T3-4: auto_resend_with_chained_key は明示的型付き AUTO_RESEND_WITH_CHAINED_KEY action として dispatch する
      if (sa.actionType === "auto_resend_with_chained_key") {
        // 型付き AutoResendWithChainedKeyAction として action を生成する
        const typedAction: AutoResendWithChainedKeyAction = {
          // subtypeAction は必ず "auto_resend_with_chained_key" リテラル
          subtypeAction: "auto_resend_with_chained_key",
          // chain 元の idempotency_key を設定する
          chainedFrom: sa.chainedFrom,
          // chain 後の新しい idempotency_key を設定する
          newKey: sa.newKey,
        };
        // 型付きアクションを AUTO_RESEND_WITH_CHAINED_KEY として push する
        actions.push({ type: "AUTO_RESEND_WITH_CHAINED_KEY", action: typedAction });
      } else {
        // auto_resend_with_chained_key 以外は従来通り DISPATCH_CONFLICT_SUBTYPE として dispatch する
        actions.push({ type: "DISPATCH_CONFLICT_SUBTYPE", subtypeAction: sa.actionType, detail: sa });
      }
    }
  }
  return { nextState, actions };
}

// pending_queue_resume の reducer
function reducePendingQueueResume<T, TPayload>(
  state: ClientState<T, TPayload>,
): ReducerResult<T, TPayload> {
  // PQ を in-order で送信する（hold 中は送信しない）
  if (state.queueHeld || state.pendingQueue.length === 0) {
    return { nextState: state, actions: [] };
  }
  return {
    nextState: state,
    actions: [{ type: "SEND_QUEUE_IN_ORDER" }],
  };
}

// business_conflict_received の reducer
function reduceBusinessConflict<T, TPayload>(
  state: ClientState<T, TPayload>,
  subtype: import("./events.js").BusinessConflictSubtype,
  _aggregateId: string,
  fieldDiff: import("./events.js").FieldDiff | undefined,
): ReducerResult<T, TPayload> {
  // PQ の先頭 entry から idempotency_key を取得する
  const firstEntry = state.pendingQueue[0];
  const idempotencyKey = firstEntry?.lineage.idempotencyKey ?? "unknown";
  // subtype に応じた actions を決定論的に取得する
  const subtypeActions = resolveSubtypeActions(subtype, idempotencyKey, fieldDiff, undefined);

  const actions: ReducerAction[] = [];
  let nextState = state;

  // subtype actions を ReducerAction に変換する（union 型で型安全にアクセスする）
  for (const sa of subtypeActions as readonly AnySubtypeAction[]) {
    switch (sa.actionType) {
      case "auto_resend_with_chained_key":
        // T3-4: auto_resend_with_chained_key は AUTO_RESEND_WITH_CHAINED_KEY 型付きアクションとして dispatch する
        // subtypeAction フィールドは必ず "auto_resend_with_chained_key" リテラルになる（string ではなく型安全）
        actions.push({
          type: "AUTO_RESEND_WITH_CHAINED_KEY",
          // 型付き AutoResendWithChainedKeyAction を設定する
          action: {
            // subtypeAction は "auto_resend_with_chained_key" リテラル型で固定する
            subtypeAction: "auto_resend_with_chained_key",
            // chain 元の idempotency_key
            chainedFrom: sa.chainedFrom,
            // chain 後の新しい idempotency_key
            newKey: sa.newKey,
          },
        });
        break;
      case "present_3way_merge_ui_hold_queue":
        // 3way merge UI + queue hold
        nextState = { ...nextState, queueHeld: true };
        actions.push({ type: "PRESENT_3WAY_MERGE_UI" });
        actions.push({ type: "HOLD_QUEUE" });
        break;
      case "refetch_server_truth":
        // server_truth を再取得する（version=0 で再初期化する）
        actions.push({ type: "UPDATE_SERVER_TRUTH", version: 0 });
        break;
      case "delete_queue_entry":
        // queue entry 削除
        nextState = {
          ...nextState,
          pendingQueue: nextState.pendingQueue.filter(
            (pq) => pq.lineage.idempotencyKey !== idempotencyKey,
          ),
        };
        actions.push({ type: "DELETE_PQ_ENTRY", idempotencyKey: sa.idempotencyKey });
        break;
      case "notify_user_silent_toast":
        // silent toast 通知
        actions.push({ type: "NOTIFY_SILENT_TOAST", message: sa.message });
        break;
      case "update_presence_indicator":
        // presence indicator 更新
        actions.push({ type: "UPDATE_PRESENCE", actorId: sa.actorId });
        break;
      case "allow_user_to_continue_or_abort":
        // ユーザーに続行/中止を選択させる（従来通り DISPATCH_CONFLICT_SUBTYPE として dispatch する）
        actions.push({ type: "DISPATCH_CONFLICT_SUBTYPE", subtypeAction: sa.actionType, detail: sa });
        break;
    }
  }

  return { nextState, actions };
}
