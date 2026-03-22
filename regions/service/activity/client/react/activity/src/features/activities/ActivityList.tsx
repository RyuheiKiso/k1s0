import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useActivities } from '../../hooks/useActivities';
import type { ActivityStatus, ActivityType } from '../../types/activity';
import styles from './ActivityList.module.css';

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

// アクティビティ種別のドットCSSクラス名マッピング
const dotClassMap: Record<ActivityType, string> = {
  comment: 'dotComment',
  time_entry: 'dotTimeEntry',
  status_change: 'dotStatusChange',
  assignment: 'dotAssignment',
};

// アクティビティ一覧コンポーネント: タイムライン形式で表示
export function ActivityList() {
  // フィルタの状態管理
  const [statusFilter, setStatusFilter] = useState<ActivityStatus | undefined>(undefined);
  const [typeFilter, setTypeFilter] = useState<ActivityType | undefined>(undefined);
  const navigate = useNavigate();

  const { data: activities, isLoading, error } = useActivities(undefined, undefined, typeFilter);

  // 表示用のフィルタリング: ステータスフィルタをクライアントサイドで適用
  const filtered = activities?.filter((a) => !statusFilter || a.status === statusFilter);

  // アイテムクリック時に詳細画面へ遷移
  const handleItemClick = (id: string) => {
    navigate({ to: '/activities/$id', params: { id } });
  };

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

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  return (
    <main>
      <h1>アクティビティ一覧</h1>

      {/* フィルタツールバー */}
      <div className={styles.toolbar}>
        {/* ステータスフィルタードロップダウン */}
        <label htmlFor="status-filter">ステータス:</label>
        <select
          id="status-filter"
          value={statusFilter ?? ''}
          onChange={(e) =>
            setStatusFilter(e.target.value ? (e.target.value as ActivityStatus) : undefined)
          }
          aria-label="ステータスでフィルター"
        >
          <option value="">すべて</option>
          <option value="active">アクティブ</option>
          <option value="submitted">申請中</option>
          <option value="approved">承認済み</option>
          <option value="rejected">却下済み</option>
        </select>

        {/* 種別フィルタードロップダウン */}
        <label htmlFor="type-filter">種別:</label>
        <select
          id="type-filter"
          value={typeFilter ?? ''}
          onChange={(e) =>
            setTypeFilter(e.target.value ? (e.target.value as ActivityType) : undefined)
          }
          aria-label="種別でフィルター"
        >
          <option value="">すべて</option>
          <option value="comment">コメント</option>
          <option value="time_entry">作業時間</option>
          <option value="status_change">ステータス変更</option>
          <option value="assignment">担当割当</option>
        </select>
      </div>

      {/* アクティビティタイムライン */}
      <ul className={styles.timeline} aria-label="アクティビティ一覧">
        {filtered?.map((activity) => (
          <li
            key={activity.id}
            className={styles.timelineItem}
            onClick={() => handleItemClick(activity.id)}
            role="button"
            tabIndex={0}
            aria-label={`アクティビティ ${activity.id.substring(0, 8)} の詳細を表示`}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') handleItemClick(activity.id);
            }}
          >
            {/* 種別を示すカラードット */}
            <span
              className={`${styles.dot} ${styles[dotClassMap[activity.activity_type]]}`}
              aria-label={typeLabels[activity.activity_type]}
            />
            <div className={styles.content}>
              {/* メタ情報: アクターID・種別・日時・ステータス */}
              <div className={styles.meta}>
                <strong>{activity.actor_id}</strong>
                {' — '}
                {typeLabels[activity.activity_type]}
                {activity.duration_minutes != null && ` (${activity.duration_minutes}分)`}
                {' · '}
                {formatDate(activity.created_at)}
                {/* ステータスバッジ */}
                <span className={`${styles.statusBadge} ${styles[statusClassMap[activity.status]]}`}>
                  {statusLabels[activity.status]}
                </span>
              </div>
              {/* アクティビティ本文 */}
              {activity.content && <p className={styles.body}>{activity.content}</p>}
            </div>
          </li>
        ))}
      </ul>

      {/* データが空の場合のメッセージ */}
      {filtered?.length === 0 && <p>アクティビティがありません。</p>}
    </main>
  );
}
