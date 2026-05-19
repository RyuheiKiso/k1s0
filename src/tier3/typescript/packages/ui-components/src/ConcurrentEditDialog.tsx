// ConcurrentEditDialog.tsx — 同時編集競合検出ダイアログ
// concurrent_edit subtype の conflict を検出した際に表示するダイアログ
// presence indicator から呼び出され、ユーザーに状況を通知する
// 11_クライアント状態適合仕様.md §concurrent_edit → update_presence_indicator の物理 UI

// React の必要なフックをインポートする
import React, { useCallback } from 'react';

// 同時編集の conflict 情報を表す型定義
export interface ConcurrentEditConflictInfo {
  // 競合を引き起こした他の actor ID
  conflictingActorId: string;
  // 競合が発生した aggregate ID
  aggregateId: string;
  // 競合が発生した aggregate の種別（表示用）
  aggregateType?: string;
}

// ConcurrentEditDialog の props 型定義
export interface ConcurrentEditDialogProps {
  // ダイアログを表示するかどうかのフラグ
  isOpen: boolean;
  // 競合情報
  conflictInfo: ConcurrentEditConflictInfo | null;
  // 編集を継続するコールバック（presence を更新して編集を続ける）
  onContinueEditing: () => void;
  // 編集を中断するコールバック（draft を破棄して server_truth に戻る）
  onAbortEditing: () => void;
}

// ConcurrentEditDialog コンポーネント
export const ConcurrentEditDialog: React.FC<ConcurrentEditDialogProps> = ({
  // ダイアログの表示フラグを受け取る
  isOpen,
  // 競合情報を受け取る
  conflictInfo,
  // 継続コールバックを受け取る
  onContinueEditing,
  // 中断コールバックを受け取る
  onAbortEditing,
}) => {
  // ダイアログが閉じている場合は何もレンダリングしない
  if (!isOpen || conflictInfo === null) {
    return null;
  }

  // 継続ボタンのクリックハンドラー
  const handleContinue = useCallback(() => {
    // 編集継続コールバックを呼び出す
    onContinueEditing();
  }, [onContinueEditing]);

  // 中断ボタンのクリックハンドラー
  const handleAbort = useCallback(() => {
    // 編集中断コールバックを呼び出す
    onAbortEditing();
  }, [onAbortEditing]);

  return (
    // dialog ロールで ARIA ダイアログとして宣言する
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="concurrent-edit-dialog-title"
      aria-describedby="concurrent-edit-dialog-desc"
    >
      {/* ダイアログタイトル（スクリーンリーダーが読み上げる） */}
      <h2 id="concurrent-edit-dialog-title">
        同時編集の競合が発生しました
      </h2>
      {/* 競合の詳細説明 */}
      <p id="concurrent-edit-dialog-desc">
        {/* 競合を引き起こした actor の情報を表示する */}
        別のユーザー（ID: {conflictInfo.conflictingActorId}）が
        {/* aggregate の種別がある場合は表示する */}
        {conflictInfo.aggregateType !== undefined ? `「${conflictInfo.aggregateType}」` : '同じデータ'}
        を編集しています。
      </p>
      {/* 選択肢の説明 */}
      <p>
        編集を継続するか、または中断して最新データを取得してください。
      </p>
      {/* アクションボタングループ */}
      <div role="group" aria-label="競合解決の選択">
        {/* 編集継続ボタン（presence を更新して編集を続ける） */}
        <button
          type="button"
          onClick={handleContinue}
          aria-label="編集を継続する"
          // 継続は主要なアクションとして autofocus を付与する
          autoFocus
        >
          編集を継続する
        </button>
        {/* 編集中断ボタン（draft を破棄して server_truth に戻る） */}
        <button
          type="button"
          onClick={handleAbort}
          aria-label="編集を中断して最新データを取得する"
        >
          中断して最新データを取得
        </button>
      </div>
    </div>
  );
};
