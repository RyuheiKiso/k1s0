import { useEffect, useRef } from 'react';
import styles from './ConfirmDialog.module.css';

// 確認ダイアログのProps定義
interface ConfirmDialogProps {
  // ダイアログの表示状態
  open: boolean;
  // ダイアログのタイトル
  title: string;
  // ダイアログの本文メッセージ
  message: string;
  // 確認ボタンのラベル（デフォルト: 確認）
  confirmLabel?: string;
  // キャンセルボタンのラベル（デフォルト: キャンセル）
  cancelLabel?: string;
  // 確認ボタン押下時のコールバック
  onConfirm: () => void;
  // キャンセルボタン押下時のコールバック
  onCancel: () => void;
}

// 確認ダイアログコンポーネント: window.confirmの代替としてReactコンポーネントベースで実装
export function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel = '確認',
  cancelLabel = 'キャンセル',
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  // 確認ボタンへのref（フォーカス管理用）
  const confirmRef = useRef<HTMLButtonElement>(null);

  // ダイアログ表示時に確認ボタンへフォーカス
  useEffect(() => {
    if (open) {
      confirmRef.current?.focus();
    }
  }, [open]);

  // 非表示時はレンダリングしない
  if (!open) return null;

  return (
    <div className={styles.overlay} role="dialog" aria-modal="true" aria-labelledby="confirm-dialog-title">
      <div className={styles.dialog}>
        <h2 id="confirm-dialog-title" className={styles.title}>{title}</h2>
        <p className={styles.message}>{message}</p>
        <div className={styles.actions}>
          <button
            type="button"
            className={styles.cancelButton}
            onClick={onCancel}
            aria-label={cancelLabel}
          >
            {cancelLabel}
          </button>
          <button
            ref={confirmRef}
            type="button"
            className={styles.confirmButton}
            onClick={onConfirm}
            aria-label={confirmLabel}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
