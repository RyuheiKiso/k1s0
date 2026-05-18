// tier3 notifications package の型定義
// 通知の種別と構造を定義する

// NotificationLevel: 通知レベルの enum 型
export type NotificationLevel = 'info' | 'warning' | 'error' | 'success';

// Notification: 1 通知の型定義
export type Notification = {
  // 通知の一意 ID (idempotency key として使用する)
  id: string;
  // 通知レベル
  level: NotificationLevel;
  // 通知メッセージ
  message: string;
  // タイムスタンプ (ISO 8601 形式)
  timestamp: string;
};
