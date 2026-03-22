import { useState } from 'react';
import { useUpdateWipLimit } from '../../hooks/useBoardColumns';
import { updateWipLimitInputSchema } from '../../types/board';
import type { BoardColumn } from '../../types/board';
import styles from './WipLimitForm.module.css';

// WipLimitFormのProps
interface WipLimitFormProps {
  column: BoardColumn;
  // フォームを閉じる際のコールバック
  onClose: () => void;
}

// WIP制限編集フォームコンポーネント: Zodバリデーション付き
export function WipLimitForm({ column, onClose }: WipLimitFormProps) {
  // WIP制限値の入力状態管理（0は無制限を意味する）
  const [wipLimit, setWipLimit] = useState<number>(column.wip_limit);
  // バリデーションエラーメッセージの状態管理
  const [error, setError] = useState<string>('');

  const updateWipLimit = useUpdateWipLimit(column.project_id, column.status_code);

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    // Zodスキーマでバリデーション実行
    const result = updateWipLimitInputSchema.safeParse({ wip_limit: wipLimit });

    if (!result.success) {
      setError(result.error.issues[0]?.message ?? 'バリデーションエラー');
      return;
    }

    // API呼び出し: 成功時にフォームを閉じる
    updateWipLimit.mutate(result.data, {
      onSuccess: () => onClose(),
    });
  };

  return (
    <div className={styles.overlay} role="dialog" aria-modal="true" aria-label="WIP制限編集">
      <div className={styles.dialog}>
        <h2 className={styles.title}>WIP制限を編集</h2>
        <p className={styles.subtitle}>
          カラム: <strong>{column.status_code}</strong>
        </p>
        <p className={styles.note}>0 を設定すると無制限になります。</p>

        <form onSubmit={handleSubmit}>
          {/* WIP制限入力欄 */}
          <div className={styles.field}>
            <label htmlFor="wip-limit">WIP制限</label>
            <input
              id="wip-limit"
              type="number"
              value={wipLimit}
              min={0}
              onChange={(e) => setWipLimit(Number(e.target.value))}
              aria-required="true"
              aria-describedby={error ? 'wip-error' : undefined}
            />
            {error && (
              <span id="wip-error" className={styles.error} role="alert">
                {error}
              </span>
            )}
          </div>

          {/* APIエラー表示 */}
          {updateWipLimit.error && (
            <p className={styles.error} role="alert">
              WIP制限の更新に失敗しました: {(updateWipLimit.error as Error).message}
            </p>
          )}

          {/* 送信・キャンセルボタン */}
          <div className={styles.actions}>
            <button
              type="submit"
              disabled={updateWipLimit.isPending}
              aria-label="WIP制限を更新"
            >
              更新
            </button>
            <button
              type="button"
              onClick={onClose}
              aria-label="キャンセル"
            >
              キャンセル
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
