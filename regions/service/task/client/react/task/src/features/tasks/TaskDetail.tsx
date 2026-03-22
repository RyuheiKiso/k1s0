import { useState } from 'react';
import { useTask, useUpdateTaskStatus } from '../../hooks/useTasks';
import type { TaskStatus } from '../../types/task';
import styles from './TaskDetail.module.css';

// タスク詳細コンポーネントのProps
interface TaskDetailProps {
  taskId: string;
}

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

// タスク詳細コンポーネント: タスク情報とステータス更新機能を提供
export function TaskDetail({ taskId }: TaskDetailProps) {
  const { data: task, isLoading, error } = useTask(taskId);
  const updateStatus = useUpdateTaskStatus(taskId);

  // ステータス更新用の選択値
  const [newStatus, setNewStatus] = useState<TaskStatus | ''>('');

  // 日付をフォーマットして表示
  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return '-';
    return new Date(dateStr).toLocaleDateString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // ステータス更新の実行
  const handleStatusUpdate = () => {
    if (!newStatus) return;
    updateStatus.mutate(
      { status: newStatus },
      {
        onSuccess: () => setNewStatus(''),
      }
    );
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  // タスクデータが存在しない場合
  if (!task) return <div>タスクが見つかりませんでした。</div>;

  return (
    <main>
      <h1>タスク詳細</h1>

      {/* ナビゲーションリンク */}
      <nav aria-label="パンくずナビゲーション">
        <a href="/">← タスク一覧に戻る</a>
      </nav>

      {/* タスク基本情報 */}
      <section className={styles.section} aria-label="タスク基本情報">
        <h2>基本情報</h2>
        <table style={{ borderCollapse: 'collapse' }}>
          <tbody>
            <tr>
              <th className={styles.infoTh}>タスクID</th>
              <td className={styles.infoTd}>{task.id}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>プロジェクトID</th>
              <td className={styles.infoTd}>{task.project_id}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>タイトル</th>
              <td className={styles.infoTd}>{task.title}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>説明</th>
              <td className={styles.infoTd}>{task.description ?? '-'}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>ステータス</th>
              <td className={styles.infoTd}>
                {/* ステータスバッジ */}
                <span className={`${styles.statusBadge} ${styles[statusClassMap[task.status]]}`}>
                  {statusLabels[task.status]}
                </span>
              </td>
            </tr>
            <tr>
              <th className={styles.infoTh}>優先度</th>
              <td className={styles.infoTd}>{task.priority}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>担当者ID</th>
              <td className={styles.infoTd}>{task.assignee_id ?? '-'}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>報告者ID</th>
              <td className={styles.infoTd}>{task.reporter_id}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>期日</th>
              <td className={styles.infoTd}>{formatDate(task.due_date)}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>ラベル</th>
              <td className={styles.infoTd}>
                {/* ラベルタグ一覧 */}
                {task.labels.length > 0
                  ? task.labels.map((label) => (
                      <span key={label} className={styles.label}>{label}</span>
                    ))
                  : '-'}
              </td>
            </tr>
            <tr>
              <th className={styles.infoTh}>バージョン</th>
              <td className={styles.infoTd}>{task.version}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>作成日時</th>
              <td className={styles.infoTd}>{formatDate(task.created_at)}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>更新日時</th>
              <td className={styles.infoTd}>{formatDate(task.updated_at)}</td>
            </tr>
          </tbody>
        </table>
      </section>

      {/* ステータス更新セクション */}
      <section className={styles.section} aria-label="ステータス更新">
        <h2>ステータス更新</h2>
        <div className={styles.statusUpdateControls}>
          <label htmlFor="new-status" className="sr-only">新しいステータス</label>
          <select
            id="new-status"
            value={newStatus}
            onChange={(e) => setNewStatus(e.target.value as TaskStatus | '')}
            aria-label="新しいステータスを選択"
          >
            <option value="">選択してください</option>
            <option value="open">オープン</option>
            <option value="in_progress">進行中</option>
            <option value="review">レビュー中</option>
            <option value="done">完了</option>
            <option value="cancelled">キャンセル</option>
          </select>
          <button
            onClick={handleStatusUpdate}
            disabled={!newStatus || updateStatus.isPending}
            aria-label="ステータスを更新"
          >
            更新
          </button>
        </div>

        {/* ステータス更新エラー表示 */}
        {updateStatus.error && (
          <p className={styles.error} role="alert">
            ステータスの更新に失敗しました: {(updateStatus.error as Error).message}
          </p>
        )}
      </section>
    </main>
  );
}
