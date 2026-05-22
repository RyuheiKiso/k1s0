// k1s0 tier3 BusinessConflict subtype actions 型定義
// 11_クライアント状態適合仕様.md の subtype branches を TypeScript 型として実装する
// subtype と UI 分岐の対応は override 禁止（CI 整合 5 の物理根拠）

import type { BusinessConflictSubtype, FieldDiff } from "./events.js";

// stale_write actions（rebase_clean: auto resend / rebase_dirty: 3way merge UI）
export type StaleWriteAction =
  // field-level rebase で clean → auto resend（新 idempotency_key chain）
  | { readonly actionType: "auto_resend_with_chained_key"; readonly chainedFrom: string; readonly newKey: string }
  // field-level rebase で dirty → 3way merge UI 表示 + queue hold
  | { readonly actionType: "present_3way_merge_ui_hold_queue" };

// lost_update actions（3way merge UI 表示 + queue hold）
export type LostUpdateAction =
  | { readonly actionType: "refetch_server_truth" }
  | { readonly actionType: "present_3way_merge_ui_hold_queue" };

// supersede actions（queue entry 削除 + silent toast）
export type SupersedeAction =
  | { readonly actionType: "delete_queue_entry"; readonly idempotencyKey: string }
  | { readonly actionType: "notify_user_silent_toast"; readonly message: string };

// concurrent_edit actions（presence indicator 更新 + user choice）
export type ConcurrentEditAction =
  | { readonly actionType: "update_presence_indicator"; readonly actorId: string }
  | { readonly actionType: "allow_user_to_continue_or_abort" };

// 全 subtype action の union 型（reducer が型安全にアクセスできるよう公開する）
export type AnySubtypeAction =
  | StaleWriteAction
  | LostUpdateAction
  | SupersedeAction
  | ConcurrentEditAction;

// subtype → actions の決定論的マッピング（override 禁止）
export function resolveSubtypeActions(
  subtype: BusinessConflictSubtype,
  idempotencyKey: string,
  fieldDiff?: FieldDiff,
  actorId?: string,
): readonly AnySubtypeAction[] {
  // subtype に応じて actions を決定論的に返す（分岐を外部から override 禁止）
  switch (subtype) {
    case "stale_write": {
      // field-level diff が disjoint かどうかで rebase_clean / dirty を判定する
      const isClean = isRebaseClean(fieldDiff);
      if (isClean) {
        // rebase_clean: auto resend（chain 元を記録した新 key を生成）
        // wall-clock TTL 禁止規約に従い Date.now() を使用せず crypto.randomUUID() で一意性を確保する
        const newKey = `${idempotencyKey}_chained_${crypto.randomUUID()}`;
        return [
          { actionType: "auto_resend_with_chained_key", chainedFrom: idempotencyKey, newKey } satisfies StaleWriteAction,
        ];
      } else {
        // rebase_dirty: 3way merge UI + queue hold
        return [
          { actionType: "present_3way_merge_ui_hold_queue" } satisfies StaleWriteAction,
        ];
      }
    }
    case "lost_update": {
      // refetch → 3way merge UI + queue hold
      return [
        { actionType: "refetch_server_truth" } satisfies LostUpdateAction,
        { actionType: "present_3way_merge_ui_hold_queue" } satisfies LostUpdateAction,
      ];
    }
    case "supersede": {
      // queue entry 削除 + silent toast
      return [
        { actionType: "delete_queue_entry", idempotencyKey } satisfies SupersedeAction,
        { actionType: "notify_user_silent_toast", message: "後続の操作で既に上書きされました" } satisfies SupersedeAction,
      ];
    }
    case "concurrent_edit": {
      // presence indicator 更新 + user choice
      return [
        { actionType: "update_presence_indicator", actorId: actorId ?? "unknown" } satisfies ConcurrentEditAction,
        { actionType: "allow_user_to_continue_or_abort" } satisfies ConcurrentEditAction,
      ];
    }
    default: {
      // 網羅性チェック（新 subtype 追加時はコンパイルエラー）
      const _exhaustive: never = subtype;
      throw new Error(`Unknown subtype: ${String(_exhaustive)}`);
    }
  }
}

// rebase が clean かどうかを判定する（field-level diff 利用）
function isRebaseClean(fieldDiff?: FieldDiff): boolean {
  // fieldDiff が無い場合は dirty とみなす（安全側に倒す）
  if (!fieldDiff) return false;
  // client / server それぞれが変更したフィールドが disjoint かどうかを確認する
  const clientSet = new Set(fieldDiff.clientFields);
  return !fieldDiff.serverFields.some((f) => clientSet.has(f));
}

// ---- subtype 別 判定 helper ----

/// isStaleWrite は BusinessConflictSubtype が "stale_write" かどうかを判定する
/// stale_write: クライアントが古い server_truth の version で write した場合に発生する
export function isStaleWrite(subtype: BusinessConflictSubtype): subtype is "stale_write" {
  // subtype が "stale_write" であれば true を返す
  return subtype === "stale_write";
}

/// isLostUpdate は BusinessConflictSubtype が "lost_update" かどうかを判定する
/// lost_update: 別のクライアントが先に同一エンティティを書き込んだ場合に発生する
export function isLostUpdate(subtype: BusinessConflictSubtype): subtype is "lost_update" {
  // subtype が "lost_update" であれば true を返す
  return subtype === "lost_update";
}

/// isSupersede は BusinessConflictSubtype が "supersede" かどうかを判定する
/// supersede: 同一クライアントの後続 pending_queue エントリが先行エントリを上書きする場合に発生する
export function isSupersede(subtype: BusinessConflictSubtype): subtype is "supersede" {
  // subtype が "supersede" であれば true を返す
  return subtype === "supersede";
}

/// isConcurrentEdit は BusinessConflictSubtype が "concurrent_edit" かどうかを判定する
/// concurrent_edit: 複数クライアントが同一エンティティを同時編集している場合に発生する
export function isConcurrentEdit(subtype: BusinessConflictSubtype): subtype is "concurrent_edit" {
  // subtype が "concurrent_edit" であれば true を返す
  return subtype === "concurrent_edit";
}

// ---- field-level diff rebase ロジック ----

/// RebaseResult は field-level diff rebase の結果を表す型
/// clean の場合は merged_values に rebase 後の値が格納される
export type RebaseResult =
  // clean rebase: client と server の変更フィールドが disjoint で自動マージ可能
  | { readonly outcome: "clean"; readonly mergedValues: Readonly<Record<string, unknown>> }
  // dirty rebase: client と server の変更フィールドが競合して 3way merge UI が必要
  | { readonly outcome: "dirty"; readonly conflictingFields: readonly string[] };

/// rebaseFieldDiff は stale_write 時の field-level diff rebase を実行する
/// clientValues: クライアント側の最新値（pending_queue のエントリ）
/// serverValues: サーバー側の最新値（server_truth の現在値）
/// fieldDiff: client / server それぞれが変更したフィールドの差分情報
/// 戻り値: clean の場合は merged_values、dirty の場合は conflictingFields
export function rebaseFieldDiff(
  // クライアント側の変更後フィールド値
  clientValues: Readonly<Record<string, unknown>>,
  // サーバー側の最新フィールド値
  serverValues: Readonly<Record<string, unknown>>,
  // field-level diff 情報（clientFields / serverFields それぞれが変更したフィールド名）
  fieldDiff: FieldDiff,
): RebaseResult {
  // client と server それぞれが変更したフィールドの Set を構築する
  const clientFieldSet = new Set(fieldDiff.clientFields);
  // server が変更したフィールドの Set を構築する
  const serverFieldSet = new Set(fieldDiff.serverFields);

  // 競合フィールド（client と server が同時に変更したフィールド）を特定する
  const conflictingFields = fieldDiff.clientFields.filter((f) => serverFieldSet.has(f));

  // 競合フィールドが存在する場合は dirty rebase を返す
  if (conflictingFields.length > 0) {
    // 競合フィールドが存在するため dirty として返す（3way merge UI が必要）
    return { outcome: "dirty", conflictingFields: conflictingFields as readonly string[] };
  }

  // 競合がない場合は clean rebase を実行する
  // base: server_truth の値をベースとして使用する
  const merged: Record<string, unknown> = { ...serverValues };

  // client が変更したフィールドを server_truth の上に適用する
  for (const field of fieldDiff.clientFields) {
    // client の変更を merged に反映する（server は変更していないため安全に適用できる）
    if (field in clientValues) {
      // client の値を merged に設定する
      merged[field] = clientValues[field];
    }
  }

  // clean rebase 結果を返す（mergedValues に自動マージ結果を設定する）
  return { outcome: "clean", mergedValues: merged };
}

/// applyServerTruthToLostUpdate は lost_update 時のサーバー値適用を実行する
/// serverValues を server_truth として返し、pending_queue を hold 状態にする準備をする
/// 戻り値: サーバー側の最新値（client は 3way merge UI でユーザーに選択させる）
export function applyServerTruthToLostUpdate(
  // サーバー側の最新フィールド値（refetch 後の server_truth）
  serverValues: Readonly<Record<string, unknown>>,
): Readonly<Record<string, unknown>> {
  // lost_update の場合はサーバー値をそのまま返す（client は pending_queue に保持する）
  // UI はこの値をベースにして 3way merge 画面を表示する
  return serverValues;
}

/// buildSupersededKeySet は supersede 時に削除すべき idempotency_key の Set を構築する
/// latestKey は保持し、それより古い同一エンティティへの write を supersede 対象とする
/// olderKeys: 削除すべき古い idempotency_key の一覧
export function buildSupersededKeySet(
  // 削除すべき古い idempotency_key の一覧
  olderKeys: readonly string[],
): ReadonlySet<string> {
  // 古い key を Set にして返す（O(1) lookup のため Set を使用する）
  return new Set(olderKeys);
}

/// mergeConcurrentEditPresence は concurrent_edit 時の presence indicator 状態を更新する
/// currentActors: 現在の編集中ユーザー Map（actorId → 最終 HLC タイムスタンプ）
/// newActorId: 新しく検出された編集者の actorId
/// newHlcTimestamp: 新しく検出された HLC タイムスタンプ（wall-clock TTL 禁止に従い HLC を使用する）
export function mergeConcurrentEditPresence(
  // 現在の presence 状態（actorId → HLC タイムスタンプ の Map）
  currentActors: ReadonlyMap<string, string>,
  // 新しく検出された actorId
  newActorId: string,
  // 新しい HLC タイムスタンプ（wall-clock TTL 禁止規約に従い HLC を使用する）
  newHlcTimestamp: string,
): ReadonlyMap<string, string> {
  // 現在の presence Map を mutable な Map にコピーする
  const updated = new Map(currentActors);
  // 新しい actor の HLC タイムスタンプを更新する（存在しない場合は追加する）
  updated.set(newActorId, newHlcTimestamp);
  // 更新後の presence Map を返す（ReadonlyMap として返す）
  return updated;
}
