import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { createActivityInputSchema } from '../../types/activity';
import { useCreateActivity } from '../../hooks/useActivities';
import type { ActivityType } from '../../types/activity';
import styles from './ActivityForm.module.css';

// アクティビティ作成フォームコンポーネント: コメント・作業時間などの入力フォーム
export function ActivityForm() {
  const navigate = useNavigate();

  // フォーム入力値の状態管理
  const [taskId, setTaskId] = useState('');
  const [actorId, setActorId] = useState('');
  const [activityType, setActivityType] = useState<ActivityType>('comment');
  const [content, setContent] = useState('');
  const [durationMinutes, setDurationMinutes] = useState<number | ''>('');
  const [idempotencyKey, setIdempotencyKey] = useState('');

  // バリデーションエラーメッセージ
  const [errors, setErrors] = useState<Record<string, string>>({});

  const createActivity = useCreateActivity();

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    const input = {
      task_id: taskId,
      actor_id: actorId,
      activity_type: activityType,
      content: content || undefined,
      duration_minutes: durationMinutes !== '' ? durationMinutes : undefined,
      idempotency_key: idempotencyKey || undefined,
    };

    // Zodスキーマでバリデーション実行
    const result = createActivityInputSchema.safeParse(input);

    if (!result.success) {
      // バリデーションエラーをフィールド別に整理
      const fieldErrors: Record<string, string> = {};
      result.error.issues.forEach((err) => {
        const field = err.path.join('.');
        fieldErrors[field] = err.message;
      });
      setErrors(fieldErrors);
      return;
    }

    // API呼び出し: 成功時にアクティビティ詳細画面へ遷移
    createActivity.mutate(result.data, {
      onSuccess: (activity) => {
        navigate({ to: '/activities/$id', params: { id: activity.id } });
      },
    });
  };

  return (
    <main>
      <h1>アクティビティ作成</h1>
      <form onSubmit={handleSubmit}>
        {/* タスクID入力欄 */}
        <div className={styles.field}>
          <label htmlFor="task_id">タスクID</label>
          <input
            id="task_id"
            value={taskId}
            onChange={(e) => setTaskId(e.target.value)}
            required
            aria-required="true"
          />
          {errors.task_id && <span className={styles.error} role="alert">{errors.task_id}</span>}
        </div>

        {/* アクターID入力欄 */}
        <div className={styles.field}>
          <label htmlFor="actor_id">アクターID</label>
          <input
            id="actor_id"
            value={actorId}
            onChange={(e) => setActorId(e.target.value)}
            required
            aria-required="true"
          />
          {errors.actor_id && <span className={styles.error} role="alert">{errors.actor_id}</span>}
        </div>

        {/* アクティビティ種別選択 */}
        <div className={styles.field}>
          <label htmlFor="activity_type">種別</label>
          <select
            id="activity_type"
            value={activityType}
            onChange={(e) => setActivityType(e.target.value as ActivityType)}
            aria-label="アクティビティ種別を選択"
          >
            <option value="comment">コメント</option>
            <option value="time_entry">作業時間</option>
            <option value="status_change">ステータス変更</option>
            <option value="assignment">担当割当</option>
          </select>
          {errors.activity_type && <span className={styles.error} role="alert">{errors.activity_type}</span>}
        </div>

        {/* 内容入力欄（コメント・ステータス変更などで使用） */}
        <div className={styles.field}>
          <label htmlFor="content">内容</label>
          <textarea
            id="content"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            rows={4}
            placeholder="アクティビティの内容を入力してください"
          />
          {errors.content && <span className={styles.error} role="alert">{errors.content}</span>}
        </div>

        {/* 作業時間入力欄（time_entry 種別で使用） */}
        {activityType === 'time_entry' && (
          <div className={styles.field}>
            <label htmlFor="duration_minutes">作業時間（分）</label>
            <input
              id="duration_minutes"
              type="number"
              min={0}
              value={durationMinutes}
              onChange={(e) =>
                setDurationMinutes(e.target.value !== '' ? Number(e.target.value) : '')
              }
              className={styles.durationInput}
              aria-label="作業時間（分単位）"
            />
            {errors.duration_minutes && (
              <span className={styles.error} role="alert">{errors.duration_minutes}</span>
            )}
          </div>
        )}

        {/* 冪等性キー入力欄（重複登録防止） */}
        <div className={styles.field}>
          <label htmlFor="idempotency_key">冪等性キー（任意）</label>
          <input
            id="idempotency_key"
            value={idempotencyKey}
            onChange={(e) => setIdempotencyKey(e.target.value)}
            placeholder="重複登録防止のためのユニークキー"
          />
        </div>

        {/* 送信・キャンセルボタン */}
        <div className={styles.actions}>
          <button type="submit" disabled={createActivity.isPending} aria-label="アクティビティを作成">
            アクティビティを作成
          </button>
          <button type="button" onClick={() => navigate({ to: '/' })} aria-label="キャンセル">
            キャンセル
          </button>
        </div>

        {/* APIエラー表示 */}
        {createActivity.error && (
          <p className={styles.error} role="alert">
            アクティビティの作成に失敗しました: {(createActivity.error as Error).message}
          </p>
        )}
      </form>
    </main>
  );
}
