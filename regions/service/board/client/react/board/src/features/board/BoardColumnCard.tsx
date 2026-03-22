import type { BoardColumn } from '../../types/board';
import { useIncrementColumn, useDecrementColumn } from '../../hooks/useBoardColumns';
import styles from './BoardColumnCard.module.css';

// BoardColumnCardのProps
interface BoardColumnCardProps {
  column: BoardColumn;
  // WIP制限編集ボタンクリック時のコールバック
  onEditWipLimit: (column: BoardColumn) => void;
}

// WIPゲージのカラー区分: 使用率に応じて色を変える
function getGaugeColor(taskCount: number, wipLimit: number): string {
  if (wipLimit === 0) return '#28a745';
  const ratio = taskCount / wipLimit;
  if (ratio >= 1) return '#dc3545';
  if (ratio >= 0.8) return '#ffc107';
  return '#28a745';
}

// カラム1枚のカードコンポーネント: WIPゲージとタスク増減ボタンを表示
export function BoardColumnCard({ column, onEditWipLimit }: BoardColumnCardProps) {
  const increment = useIncrementColumn();
  const decrement = useDecrementColumn();

  // WIPゲージのパーセンテージを計算する（WIP制限0は無制限扱い）
  const gaugePercent =
    column.wip_limit > 0
      ? Math.min((column.task_count / column.wip_limit) * 100, 100)
      : 0;

  const gaugeColor = getGaugeColor(column.task_count, column.wip_limit);

  // タスク数をインクリメントする
  const handleIncrement = () => {
    increment.mutate({
      project_id: column.project_id,
      status_code: column.status_code,
    });
  };

  // タスク数をデクリメントする（0未満にはならない）
  const handleDecrement = () => {
    if (column.task_count <= 0) return;
    decrement.mutate({
      project_id: column.project_id,
      status_code: column.status_code,
    });
  };

  return (
    <article className={styles.card} aria-label={`カラム: ${column.status_code}`}>
      {/* カラムヘッダー: ステータスコードとWIP制限編集ボタン */}
      <div className={styles.header}>
        <h2 className={styles.title}>{column.status_code}</h2>
        <button
          className={styles.editButton}
          onClick={() => onEditWipLimit(column)}
          aria-label={`${column.status_code} のWIP制限を編集`}
        >
          編集
        </button>
      </div>

      {/* WIPゲージ: タスク数 / WIP制限を視覚化 */}
      <div className={styles.gaugeContainer} aria-label="WIP使用率ゲージ">
        <div
          className={styles.gaugeBar}
          style={{ width: `${gaugePercent}%`, backgroundColor: gaugeColor }}
          role="progressbar"
          aria-valuenow={column.task_count}
          aria-valuemin={0}
          aria-valuemax={column.wip_limit > 0 ? column.wip_limit : undefined}
        />
      </div>

      {/* タスク数とWIP制限の表示 */}
      <div className={styles.countArea}>
        <span className={styles.taskCount}>{column.task_count}</span>
        <span className={styles.wipLabel}>
          / {column.wip_limit > 0 ? column.wip_limit : '∞'} WIP
        </span>
      </div>

      {/* タスク数の増減ボタン */}
      <div className={styles.actions}>
        <button
          className={styles.decrementButton}
          onClick={handleDecrement}
          disabled={column.task_count <= 0 || decrement.isPending}
          aria-label={`${column.status_code} のタスク数を減らす`}
        >
          −
        </button>
        <button
          className={styles.incrementButton}
          onClick={handleIncrement}
          disabled={increment.isPending}
          aria-label={`${column.status_code} のタスク数を増やす`}
        >
          ＋
        </button>
      </div>

      {/* エラー表示 */}
      {(increment.error || decrement.error) && (
        <p className={styles.error} role="alert">
          操作に失敗しました:{' '}
          {((increment.error ?? decrement.error) as Error).message}
        </p>
      )}
    </article>
  );
}
