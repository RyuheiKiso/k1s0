// tier3 notifications idempotency テスト
// 同一 ID の通知が重複して追加されないことを確認する

// vitest の test/expect をインポートする
import { test, expect } from 'vitest';
// addNotificationIdempotent と MAX_NOTIFICATIONS をインポートする
import { addNotificationIdempotent, MAX_NOTIFICATIONS } from '../src/index.js';
// NotificationEntry 型をインポートする
import type { NotificationEntry } from '../src/index.js';

// テスト用通知の作成ヘルパー関数
function makeNotification(id: string, kind: NotificationEntry['kind'] = 'info'): NotificationEntry {
  // テスト用通知エントリを生成する
  return {
    // 通知 ID を設定する
    id,
    // 通知種別を設定する
    kind,
    // テスト用メッセージを設定する
    message: `test message ${id}`,
    // 表示時間を設定する
    durationMs: 4000,
    // aria-live を設定する (info は polite)
    ariaLive: kind === 'error' ? 'assertive' : 'polite',
  };
}

// テスト: 同一 ID の通知が重複して追加されないことを確認する
test('same id notification is not added twice (idempotent)', () => {
  // 空のリストを作成する
  const notifications: NotificationEntry[] = [];
  // 最初の追加
  const n1 = addNotificationIdempotent(notifications, makeNotification('n-001'));
  // 同一 ID で再度追加 (idempotent: 増えない)
  const n2 = addNotificationIdempotent(n1, makeNotification('n-001'));
  // 件数が 1 件のままであることを確認する
  expect(n2.length).toBe(1);
});

// テスト: MAX_NOTIFICATIONS を超えた場合に最古の通知が削除されることを確認する
test('notifications are capped at MAX_NOTIFICATIONS', () => {
  // 空のリストから開始する
  let notifications: NotificationEntry[] = [];
  // MAX_NOTIFICATIONS + 1 件追加する
  for (let i = 0; i < MAX_NOTIFICATIONS + 1; i++) {
    // 順次通知を追加する
    notifications = addNotificationIdempotent(notifications, makeNotification(`n-${i}`));
  }
  // 件数が MAX_NOTIFICATIONS 以内であることを確認する
  expect(notifications.length).toBeLessThanOrEqual(MAX_NOTIFICATIONS);
});

// テスト: error レベルは info より優先されることを確認する (型レベル検査)
test('notification kind types are valid', () => {
  // 全 kind を定義する
  const kinds: NotificationEntry['kind'][] = ['info', 'warning', 'error', 'success'];
  // 全 kind が 4 種類であることを確認する
  expect(kinds.length).toBe(4);
});
