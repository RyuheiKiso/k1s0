import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useTasks } from '../../hooks/useTasks';
import type { TaskStatus, TaskPriority } from '../../types/task';
import styles from './TaskList.module.css';

// ステータス表示ラベルのマッピング
const statusLabels: Record<TaskStatus, string> = {
  open: 'オープン',
  in_progress: '進行中',
  review: 'レビュー中',
  done: '完了',
  cancelled: 'キャンセル',
};

// ステータスバッジのCSSクラス名マッピング
const statusClassMap: Record<TaskStatus, string> = {
  open: 'statusOpen',
  in_progress: 'statusInProgress',
  review: 'statusReview',
  done: 'statusDone',
  cancelled: 'statusCancelled',
};

// 優先度表示ラベルのマッピング
const priorityLabels: Record<TaskPriority, string> = {
  low: '低',
  medium: '中',
  high: '高',
  critical: '緊急',
};

// 優先度バッジのCSSクラス名マッピング
const priorityClassMap: Record<TaskPriority, string> = {
  low: 'priorityLow',
  medium: 'priorityMedium',
  high: 'priorityHigh',
  critical: 'priorityCritical',
};

// タスク一覧コンポーネント: テーブル表示でステータス・プロジェクトフィルタ機能を提供
export function TaskList() {
  // ステータスフィルターの状態管理
  const [statusFilter, setStatusFilter] = useState<TaskStatus | undefined>(undefined);
  const navigate = useNavigate();

  const { data: tasks, isLoading, error } = useTasks(undefined, statusFilter, undefined);

  // タスク行クリック時に詳細画面へ遷移
  const handleRowClick = (id: string) => {
    navigate({ to: '/tasks/$id', params: { id } });
  };

  // 日付をフォーマットして表示
  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return '-';
    return new Date(dateStr).toLocaleDateString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
    });
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  return (
    <main>
      <h1>タスク一覧</h1>

      {/* ステータスフィルタードロップダウン */}
      <div className={styles.toolbar}>
        <label htmlFor="status-filter">ステータス:</label>
        <select
          id="status-filter"
          value={statusFilter ?? ''}
          onChange={(e) =>
            setStatusFilter(e.target.value ? (e.target.value as TaskStatus) : undefined)
          }
          aria-label="ステータスでフィルター"
        >
          <option value="">すべて</option>
          <option value="open">オープン</option>
          <option value="in_progress">進行中</option>
          <option value="review">レビュー中</option>
          <option value="done">完了</option>
          <option value="cancelled">キャンセル</option>
        </select>
      </div>

      {/* タスク一覧テーブル */}
      <table className={styles.table} aria-label="タスク一覧">
        <thead>
          <tr>
            <th className={styles.th}>タスクID</th>
            <th className={styles.th}>タイトル</th>
            <th className={styles.th}>ステータス</th>
            <th className={styles.th}>優先度</th>
            <th className={styles.th}>担当者</th>
            <th className={styles.th}>期日</th>
            <th className={styles.th}>作成日</th>
          </tr>
        </thead>
        <tbody>
          {tasks?.map((task) => (
            <tr
              key={task.id}
              onClick={() => handleRowClick(task.id)}
              className={styles.clickableRow}
              role="button"
              tabIndex={0}
              aria-label={`タスク ${task.title} の詳細を表示`}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') handleRowClick(task.id);
              }}
            >
              <td className={styles.td}>{task.id.substring(0, 8)}...</td>
              <td className={styles.td}>{task.title}</td>
              <td className={styles.td}>
                {/* ステータスバッジ: ステータスに応じた色で表示 */}
                <span className={`${styles.statusBadge} ${styles[statusClassMap[task.status]]}`}>
                  {statusLabels[task.status]}
                </span>
              </td>
              <td className={styles.td}>
                {/* 優先度バッジ: 優先度に応じた色で表示 */}
                <span className={`${styles.priorityBadge} ${styles[priorityClassMap[task.priority]]}`}>
                  {priorityLabels[task.priority]}
                </span>
              </td>
              <td className={styles.td}>{task.assignee_id ?? '-'}</td>
              <td className={styles.td}>{formatDate(task.due_date)}</td>
              <td className={styles.td}>{formatDate(task.created_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {tasks?.length === 0 && <p>タスクがありません。</p>}
    </main>
  );
}
