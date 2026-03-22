import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { createTaskInputSchema } from '../../types/task';
import { useCreateTask } from '../../hooks/useTasks';
import styles from './TaskForm.module.css';

// タスク作成フォームコンポーネント: Zodバリデーション付き
export function TaskForm() {
  const navigate = useNavigate();

  // フォーム入力値の状態管理
  const [projectId, setProjectId] = useState('');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<'low' | 'medium' | 'high' | 'critical'>('medium');
  const [assigneeId, setAssigneeId] = useState('');
  const [dueDate, setDueDate] = useState('');
  const [labels, setLabels] = useState('');

  // バリデーションエラーメッセージ
  const [errors, setErrors] = useState<Record<string, string>>({});

  const createTask = useCreateTask();

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    const input = {
      project_id: projectId,
      title,
      description: description || undefined,
      priority,
      assignee_id: assigneeId || undefined,
      due_date: dueDate || undefined,
      labels: labels
        ? labels
            .split(',')
            .map((l) => l.trim())
            .filter((l) => l.length > 0)
        : undefined,
    };

    // Zodスキーマでバリデーション実行
    const result = createTaskInputSchema.safeParse(input);

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

    // API呼び出し: 成功時にタスク詳細画面へ遷移
    createTask.mutate(result.data, {
      onSuccess: (task) => {
        navigate({ to: '/tasks/$id', params: { id: task.id } });
      },
    });
  };

  return (
    <main>
      <h1>新規タスク作成</h1>
      <form onSubmit={handleSubmit}>
        {/* プロジェクトID入力欄 */}
        <div className={styles.field}>
          <label htmlFor="project_id">プロジェクトID</label>
          <input
            id="project_id"
            value={projectId}
            onChange={(e) => setProjectId(e.target.value)}
            required
            aria-required="true"
          />
          {errors.project_id && (
            <span className={styles.error} role="alert">
              {errors.project_id}
            </span>
          )}
        </div>

        {/* タイトル入力欄 */}
        <div className={styles.field}>
          <label htmlFor="title">タイトル</label>
          <input
            id="title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
            aria-required="true"
          />
          {errors.title && (
            <span className={styles.error} role="alert">
              {errors.title}
            </span>
          )}
        </div>

        {/* 説明入力欄 */}
        <div className={styles.field}>
          <label htmlFor="description">説明</label>
          <textarea
            id="description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
          />
        </div>

        {/* 優先度選択 */}
        <div className={styles.field}>
          <label htmlFor="priority">優先度</label>
          <select
            id="priority"
            value={priority}
            onChange={(e) => setPriority(e.target.value as 'low' | 'medium' | 'high' | 'critical')}
            aria-label="優先度を選択"
          >
            <option value="low">低</option>
            <option value="medium">中</option>
            <option value="high">高</option>
            <option value="critical">緊急</option>
          </select>
        </div>

        {/* 担当者ID入力欄 */}
        <div className={styles.field}>
          <label htmlFor="assignee_id">担当者ID（任意）</label>
          <input
            id="assignee_id"
            value={assigneeId}
            onChange={(e) => setAssigneeId(e.target.value)}
          />
        </div>

        {/* 期日入力欄 */}
        <div className={styles.field}>
          <label htmlFor="due_date">期日（任意）</label>
          <input
            id="due_date"
            type="date"
            value={dueDate}
            onChange={(e) => setDueDate(e.target.value)}
          />
        </div>

        {/* ラベル入力欄（カンマ区切り） */}
        <div className={styles.field}>
          <label htmlFor="labels">ラベル（カンマ区切り、任意）</label>
          <input
            id="labels"
            value={labels}
            onChange={(e) => setLabels(e.target.value)}
            placeholder="例: bug, frontend, urgent"
          />
        </div>

        {/* 送信・キャンセルボタン */}
        <div className={styles.actions}>
          <button type="submit" disabled={createTask.isPending} aria-label="タスクを作成">
            タスクを作成
          </button>
          <button type="button" onClick={() => navigate({ to: '/' })} aria-label="キャンセル">
            キャンセル
          </button>
        </div>

        {/* APIエラー表示 */}
        {createTask.error && (
          <p className={styles.error} role="alert">
            タスクの作成に失敗しました: {(createTask.error as Error).message}
          </p>
        )}
      </form>
    </main>
  );
}
