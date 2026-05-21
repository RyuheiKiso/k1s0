// k1s0 tier3 4 layer クライアント状態型定義
// 11_クライアント状態適合仕様.md の v1 layer セットを TypeScript 型として実装する
// 新しい layer の追加は破壊的変更（major version up 必須）

// layer を識別する型
export type LayerId =
  // tier2 atomic 三表書込確定値のキャッシュ
  | "server_truth"
  // mutation in-flight overlay（ack で promote / reject で rollback）
  | "optimistic_local"
  // offline 永続化 mutation 経路（IndexedDB 暗号化）
  | "pending_queue"
  // 編集中フォームの dirty state（IndexedDB 暗号化）
  | "draft";

// lineage タプル: 値がどの layer 由来かを追跡する
export interface Lineage {
  // 所属 layer
  readonly layer: LayerId;
  // aggregate バージョン
  readonly aggregateVersion: number;
  // HLC タイムスタンプ（wall-clock 禁止、HLC のみ使用）
  readonly hlcTimestamp: string;
}

// ServerTruth layer: tier2 確定値キャッシュ（in-memory のみ）
export interface ServerTruthEntry<T> {
  // layer 識別子
  readonly layer: "server_truth";
  // aggregate の現在値
  readonly value: T;
  // lineage タプル（trace_id / observed_at を含む）
  readonly lineage: Lineage & {
    readonly traceId: string;
    readonly observedAt: string;
  };
}

// OptimisticLocal layer: in-flight mutation overlay
export interface OptimisticLocalEntry<T> {
  // layer 識別子
  readonly layer: "optimistic_local";
  // 楽観的更新後の予測値
  readonly value: T;
  // lineage タプル（idempotency_key / started_at を含む）
  readonly lineage: Lineage & {
    readonly predictedNextVersion: number;
    readonly idempotencyKey: string;
    readonly startedAt: string;
  };
}

// PendingQueue layer エントリ（IndexedDB 永続化 / PII strip 済み）
export interface PendingQueueEntry<TPayload> {
  // layer 識別子
  readonly layer: "pending_queue";
  // PII strip 済みの payload（PII フィールドを in-memory でのみ保持する）
  readonly payload: TPayload;
  // lineage タプル（idempotency_key / enqueued_at を含む）
  readonly lineage: Lineage & {
    readonly idempotencyKey: string;
    readonly enqueuedAt: string;
    // 24h TTL（UNIX ミリ秒）
    readonly expiresAtMs: number;
    // chain 元 idempotency_key（rebase 再送時に設定される）
    readonly chainedFrom?: string;
  };
}

// Draft layer エントリ（IndexedDB 永続化 / 編集中フォーム）
export interface DraftEntry<T> {
  // layer 識別子
  readonly layer: "draft";
  // 編集中の値（confirm 前）
  readonly value: T;
  // lineage タプル（form_id / edited_at を含む）
  readonly lineage: Lineage & {
    readonly formId: string;
    readonly editedAt: string;
  };
}

// PurgeReason: 5 purge trigger のいずれかを識別する型
export type PurgeReason =
  // ログアウト
  | "logout"
  // refresh token 失効
  | "refresh_token_expiry"
  // テナント切り替え
  | "tenant_switch"
  // actor 切り替え
  | "actor_switch"
  // device_bound_key ローテーション
  | "device_bound_key_rotate";

// ALL_LAYERS: 全 layer の識別子を網羅した定数配列
// rotation_wiring.ts / logout_handler.ts が LAYERS_TO_PURGE の代わりにこの定数を import して使用する
// 新しい layer 追加時はこの定数も更新する（major version up が必要）
export const ALL_LAYERS = [
  // tier2 atomic 三表書込確定値のキャッシュ（server_truth の略称: ST）
  "ST",
  // mutation in-flight overlay（optimistic_local の略称: OL）
  "OL",
  // offline 永続化 mutation 経路（pending_queue の略称: PQ）
  "PQ",
  // 編集中フォームの dirty state（draft の略称: DR）
  "DR",
] as const;

// AllLayer: ALL_LAYERS の各要素の型（略称形式）
export type AllLayer = (typeof ALL_LAYERS)[number];

// PurgeEvent: 全 layer purge 時に audit emit するイベント
export interface PurgeEvent {
  // purge 理由
  readonly reason: PurgeReason;
  // purge 対象 tenant_id（BFF cookie から注入。直接引数禁止）
  readonly tenantId?: never;
  // HLC タイムスタンプ
  readonly hlcTimestamp: string;
}
