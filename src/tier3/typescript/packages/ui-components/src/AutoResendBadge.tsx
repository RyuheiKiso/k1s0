// AutoResendBadge.tsx — pending queue auto-resend 状態バッジコンポーネント
// pending queue に溜まった mutation の自動再送信状態を視覚的に表示する
// 11_クライアント状態適合仕様.md §pending_queue_resume → send_queue_in_order の物理 UI
// wall-clock TTL 禁止: HLC カウンタベースの状態管理を使用する

// React をインポートする
import React from 'react';

// auto-resend の現在状態を表す型定義
export type AutoResendStatus =
  // pending queue が空（表示不要）
  | 'idle'
  // pending queue に未送信の mutation がある（送信待ち）
  | 'pending'
  // 送信中（WebSocket 経由で送信処理中）
  | 'sending'
  // 送信完了（最後の mutation が acknowledge された直後）
  | 'completed'
  // 送信エラー（再試行待ち）
  | 'error';

// AutoResendBadge の props 型定義
export interface AutoResendBadgeProps {
  // 現在の auto-resend 状態
  status: AutoResendStatus;
  // pending queue の未送信件数
  pendingCount: number;
  // 送信エラーの場合のエラーメッセージ（省略可能）
  errorMessage?: string;
  // バッジをクリックした際のコールバック（手動再送信トリガー用）
  onManualRetry?: () => void;
}

// auto-resend 状態に応じたバッジのラベルを返すヘルパー関数
function getStatusLabel(status: AutoResendStatus, pendingCount: number): string {
  // 状態に応じたラベル文字列を返す
  switch (status) {
    case 'idle':
      // idle 状態は表示しない（空文字を返す）
      return '';
    case 'pending':
      // pending 件数を含むラベルを返す
      return `${pendingCount} 件の変更を送信待ち`;
    case 'sending':
      // 送信中ラベルを返す
      return `${pendingCount} 件を送信中...`;
    case 'completed':
      // 完了ラベルを返す
      return '変更を送信しました';
    case 'error':
      // エラーラベルを返す
      return '送信エラー（タップして再試行）';
    default: {
      // 未知の状態は TypeScript の exhaustive check のため never に型アサートする
      const _exhaustive: never = status;
      void _exhaustive;
      return '';
    }
  }
}

// AutoResendBadge コンポーネント
export const AutoResendBadge: React.FC<AutoResendBadgeProps> = ({
  // auto-resend の現在状態を受け取る
  status,
  // pending queue の未送信件数を受け取る
  pendingCount,
  // エラーメッセージを受け取る（省略可能）
  errorMessage,
  // 手動再送信コールバックを受け取る（省略可能）
  onManualRetry,
}) => {
  // idle 状態はバッジを表示しない
  if (status === 'idle') {
    return null;
  }

  // バッジのラベルを取得する
  const label = getStatusLabel(status, pendingCount);

  // エラー状態でかつ手動再試行コールバックがある場合はボタンとして表示する
  if (status === 'error' && onManualRetry !== undefined) {
    return (
      // role="button" のバッジ（クリックで手動再送信をトリガーする）
      <button
        type="button"
        onClick={onManualRetry}
        aria-label={errorMessage !== undefined ? `${label}: ${errorMessage}` : label}
        // aria-live="assertive" でスクリーンリーダーにエラーを即時通知する
        aria-live="assertive"
      >
        {/* エラーアイコンとラベルを表示する */}
        <span aria-hidden="true">⚠️</span>
        {' '}
        {label}
      </button>
    );
  }

  return (
    // status インジケータとして表示する（ARIA role="status"）
    <div
      role="status"
      aria-label={label}
      // aria-live="polite" で非緊急の状態変化をスクリーンリーダーに通知する
      aria-live="polite"
      aria-atomic="true"
    >
      {/* 状態アイコンを条件分岐で表示する */}
      {status === 'pending' && (
        // 送信待ちアイコン（時計マーク）
        <span aria-hidden="true">⏳</span>
      )}
      {status === 'sending' && (
        // 送信中アイコン（回転中マーク）
        <span aria-hidden="true" role="img" aria-label="送信中">↻</span>
      )}
      {status === 'completed' && (
        // 完了アイコン（チェックマーク）
        <span aria-hidden="true">✓</span>
      )}
      {' '}
      {/* ラベルテキストを表示する */}
      {label}
    </div>
  );
};
