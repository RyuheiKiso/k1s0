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
