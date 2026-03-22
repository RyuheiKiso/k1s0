import { useState } from 'react';
import {
  useActivity,
  useSubmitActivity,
  useApproveActivity,
  useRejectActivity,
} from '../../hooks/useActivities';
import type { ActivityStatus, ActivityType } from '../../types/activity';
import styles from './ActivityDetail.module.css';

// アクティビティ詳細コンポーネントのProps
interface ActivityDetailProps {
  activityId: string;
}

// ステータス表示ラベルのマッピング
const statusLabels: Record<ActivityStatus, string> = {
  active: 'アクティブ',
  submitted: '申請中',
  approved: '承認済み',
  rejected: '却下済み',
};

// ステータスバッジのCSSクラス名マッピング
const statusClassMap: Record<ActivityStatus, string> = {
  active: 'statusActive',
  submitted: 'statusSubmitted',
  approved: 'statusApproved',
  rejected: 'statusRejected',
};

// アクティビティ種別の表示ラベルマッピング
const typeLabels: Record<ActivityType, string> = {
  comment: 'コメント',
  time_entry: '作業時間',
  status_change: 'ステータス変更',
  assignment: '担当割当',
};

// アクティビティ詳細コンポーネント: アクティビティ情報と承認フロー操作を提供
export function ActivityDetail({ activityId }: ActivityDetailProps) {
  const { data: activity, isLoading, error } = useActivity(activityId);
  const submit = useSubmitActivity(activityId);
  const approve = useApproveActivity(activityId);
  const reject = useRejectActivity(activityId);

  // 却下理由の入力値
  const [rejectReason, setRejectReason] = useState('');

  // 日付をフォーマットして表示
  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // 承認申請の実行
  const handleSubmit = () => {
    submit.mutate(undefined);
  };

  // 承認の実行
  const handleApprove = () => {
    approve.mutate(undefined);
  };

  // 却下の実行
  const handleReject = () => {
    reject.mutate({ reason: rejectReason || undefined }, {
      onSuccess: () => setRejectReason(''),
    });
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  // アクティビティデータが存在しない場合
  if (!activity) return <div>アクティビティが見つかりませんでした。</div>;

  return (
    <main>
      <h1>アクティビティ詳細</h1>

      {/* ナビゲーションリンク */}
      <nav aria-label="パンくずナビゲーション">
        <a href="/">← アクティビティ一覧に戻る</a>
      </nav>

      {/* アクティビティ基本情報 */}
      <section className={styles.section} aria-label="アクティビティ基本情報">
        <h2>基本情報</h2>
        <table style={{ borderCollapse: 'collapse' }}>
          <tbody>
            <tr>
              <th className={styles.infoTh}>アクティビティID</th>
              <td className={styles.infoTd}>{activity.id}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>タスクID</th>
              <td className={styles.infoTd}>{activity.task_id}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>アクターID</th>
              <td className={styles.infoTd}>{activity.actor_id}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>種別</th>
              <td className={styles.infoTd}>{typeLabels[activity.activity_type]}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>ステータス</th>
              <td className={styles.infoTd}>
                {/* ステータスバッジ */}
                <span className={`${styles.statusBadge} ${styles[statusClassMap[activity.status]]}`}>
                  {statusLabels[activity.status]}
                </span>
              </td>
            </tr>
            {activity.content && (
              <tr>
                <th className={styles.infoTh}>内容</th>
                <td className={styles.infoTd}>{activity.content}</td>
              </tr>
            )}
            {activity.duration_minutes != null && (
              <tr>
                <th className={styles.infoTh}>作業時間</th>
                <td className={styles.infoTd}>{activity.duration_minutes}分</td>
              </tr>
            )}
            <tr>
              <th className={styles.infoTh}>バージョン</th>
              <td className={styles.infoTd}>{activity.version}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>作成日時</th>
              <td className={styles.infoTd}>{formatDate(activity.created_at)}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>更新日時</th>
              <td className={styles.infoTd}>{formatDate(activity.updated_at)}</td>
            </tr>
          </tbody>
        </table>
      </section>

      {/* 承認フローセクション */}
      <section className={styles.section} aria-label="承認フロー操作">
        <h2>承認フロー</h2>
        <div className={styles.actionControls}>
          {/* 承認申請ボタン（active のときのみ表示） */}
          {activity.status === 'active' && (
            <button
              onClick={handleSubmit}
              disabled={submit.isPending}
              aria-label="承認申請"
            >
              承認申請
            </button>
          )}

          {/* 承認ボタン（submitted のときのみ表示） */}
          {activity.status === 'submitted' && (
            <button
              onClick={handleApprove}
              disabled={approve.isPending}
              aria-label="承認"
            >
              承認
            </button>
          )}

          {/* 却下ボタンと理由入力（submitted のときのみ表示） */}
          {activity.status === 'submitted' && (
            <div>
              <input
                type="text"
                value={rejectReason}
                onChange={(e) => setRejectReason(e.target.value)}
                placeholder="却下理由（任意）"
                className={styles.rejectReasonInput}
                aria-label="却下理由"
              />
              <button
                onClick={handleReject}
                disabled={reject.isPending}
                aria-label="却下"
              >
                却下
              </button>
            </div>
          )}
        </div>

        {/* 承認フロー操作エラー表示 */}
        {submit.error && (
          <p className={styles.error} role="alert">
            承認申請に失敗しました: {(submit.error as Error).message}
          </p>
        )}
        {approve.error && (
          <p className={styles.error} role="alert">
            承認に失敗しました: {(approve.error as Error).message}
          </p>
        )}
        {reject.error && (
          <p className={styles.error} role="alert">
            却下に失敗しました: {(reject.error as Error).message}
          </p>
        )}
      </section>
    </main>
  );
}
