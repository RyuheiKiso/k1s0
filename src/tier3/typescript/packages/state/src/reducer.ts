// k1s0 tier3 4 layer state reducer
// 11_クライアント状態適合仕様.md の 5 event × 4 subtype × 4 layer を実装する
// reducer は決定論的（同 event に対し常に同じ actions を返す）

import type { ConflictEvent } from "./events.js";
import type { PendingQueueEntry, ServerTruthEntry, OptimisticLocalEntry, PurgeEvent, PurgeReason } from "./layers.js";
import { resolveSubtypeActions } from "./subtypes.js";
import type { AnySubtypeAction } from "./subtypes.js";

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
  // BusinessConflict subtype action を実行する
  | { readonly type: "DISPATCH_CONFLICT_SUBTYPE"; readonly subtypeAction: string; readonly detail: unknown }
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
function reduceOptimisticAcknowledged<T, TPayload>(
  state: ClientState<T, TPayload>,
  idempotencyKey: string,
  confirmedVersion: number,
): ReducerResult<T, TPayload> {
  // OL を promote して ST に格納し、対応する PQ entry を削除する
  const nextState: ClientState<T, TPayload> = {
    ...state,
    optimisticLocal: null,
    pendingQueue: state.pendingQueue.filter(
      (pq) => pq.lineage.idempotencyKey !== idempotencyKey,
    ),
  };
  return {
    nextState,
    actions: [
      { type: "PROMOTE_OPTIMISTIC", idempotencyKey },
      { type: "DELETE_PQ_ENTRY", idempotencyKey },
      { type: "UPDATE_SERVER_TRUTH", version: confirmedVersion },
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
      actions.push({ type: "DISPATCH_CONFLICT_SUBTYPE", subtypeAction: sa.actionType, detail: sa });
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
        // PQ の idempotency_key を更新する（chain 後の新 key に変更）
        actions.push({ type: "DISPATCH_CONFLICT_SUBTYPE", subtypeAction: sa.actionType, detail: sa });
        break;
      case "present_3way_merge_ui_hold_queue":
        // 3way merge UI + queue hold
        nextState = { ...nextState, queueHeld: true };
        actions.push({ type: "PRESENT_3WAY_MERGE_UI" });
        actions.push({ type: "HOLD_QUEUE" });
        break;
      case "refetch_server_truth":
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
        actions.push({ type: "NOTIFY_SILENT_TOAST", message: sa.message });
        break;
      case "update_presence_indicator":
        actions.push({ type: "UPDATE_PRESENCE", actorId: sa.actorId });
        break;
      case "allow_user_to_continue_or_abort":
        actions.push({ type: "DISPATCH_CONFLICT_SUBTYPE", subtypeAction: sa.actionType, detail: sa });
        break;
    }
  }

  return { nextState, actions };
}
