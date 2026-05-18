// src/tier3/typescript/packages/notifications/src/index.ts
// tier3 通知パッケージ: Toast / Snackbar / Dialog の通知管理
// WCAG 2.1 AA: aria-live="polite" / aria-live="assertive" で支援技術に通知する

// 通知種別を定義する (info / success / warning / error)
export type NotificationKind = "info" | "success" | "warning" | "error";

// 通知エントリの型を定義する
export type NotificationEntry = {
  // 通知の一意識別子 (UUID v4 推奨)
  readonly id: string;
  // 通知種別
  readonly kind: NotificationKind;
  // 通知メッセージ (i18n 済みの文字列)
  readonly message: string;
  // 通知の表示時間 (ms)、null の場合は手動 dismiss まで表示する
  readonly durationMs: number | null;
  // ARIA live region の緊急度 (assertive はエラーのみ使用する)
  readonly ariaLive: "polite" | "assertive";
};

// 通知の最大同時表示数 (UX 設計原則: 画面占有を防ぐ)
export const MAX_NOTIFICATIONS = 5;

// 通知エントリを生成するファクトリ関数
export function createNotification(
  kind: NotificationKind,
  message: string,
  durationMs: number | null = 4000,
): NotificationEntry {
  // UUID v4 形式の ID を生成する (crypto.randomUUID が利用可能な場合)
  const id =
    typeof crypto !== "undefined" && crypto.randomUUID
      ? // Web Crypto API が利用可能な場合は randomUUID を使用する
        crypto.randomUUID()
      : // フォールバック: Math.random ベースの疑似 UUID を生成する
        `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  // aria-live の緊急度を kind から決定する (error のみ assertive)
  const ariaLive: "polite" | "assertive" =
    kind === "error" ? "assertive" : "polite";
  // NotificationEntry を返す
  return { id, kind, message, durationMs, ariaLive };
}

// 通知ストアの型を定義する (React の useState と統合する)
export type NotificationStore = {
  // 現在表示中の通知リスト
  readonly notifications: readonly NotificationEntry[];
  // 通知を追加する関数
  readonly add: (entry: NotificationEntry) => void;
  // 通知を ID で削除する関数
  readonly dismiss: (id: string) => void;
  // 全通知を削除する関数
  readonly clear: () => void;
};

// 通知の idempotent な追加ロジック
// 同一 ID の通知が重複して追加されることを防ぐ
export function addNotificationIdempotent(
  // 既存の通知リスト
  notifications: readonly NotificationEntry[],
  // 追加する通知
  notification: NotificationEntry,
): NotificationEntry[] {
  // 同一 ID の通知が既に存在する場合は追加しない (idempotency)
  if (notifications.some(n => n.id === notification.id)) {
    // 既存のリストをそのまま返す
    return [...notifications];
  }
  // 最大件数を超える場合は最古の通知を削除する
  const next = [...notifications, notification];
  // MAX_NOTIFICATIONS 件数以内に収める
  return next.slice(-MAX_NOTIFICATIONS);
}
