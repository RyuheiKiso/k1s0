import { useState } from 'react';
import { createProjectTypeSchema, updateProjectTypeSchema } from '../../types/projectMaster';
import { useCreateProjectType, useUpdateProjectType } from '../../hooks/useProjectTypes';
import type { ProjectType } from '../../types/projectMaster';

// プロジェクトタイプフォームのProps: projectTypeがある場合は編集モード
interface ProjectTypeFormProps {
  projectType?: ProjectType;
  onClose: () => void;
}

// プロジェクトタイプ作成・編集フォームコンポーネント: Zodバリデーション付き
export function ProjectTypeForm({ projectType, onClose }: ProjectTypeFormProps) {
  const isEditing = !!projectType;

  // フォーム入力値の状態管理
  const [code, setCode] = useState(projectType?.code ?? '');
  const [displayName, setDisplayName] = useState(projectType?.display_name ?? '');
  const [description, setDescription] = useState(projectType?.description ?? '');
  const [defaultWorkflow, setDefaultWorkflow] = useState(projectType?.default_workflow ?? '');
  const [isActive, setIsActive] = useState(projectType?.is_active ?? true);
  const [sortOrder, setSortOrder] = useState(projectType?.sort_order ?? 0);
  // バリデーションエラーメッセージ
  const [errors, setErrors] = useState<Record<string, string>>({});

  const createProjectType = useCreateProjectType();
  const updateProjectType = useUpdateProjectType(projectType?.id ?? '');

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    const input = {
      code,
      display_name: displayName,
      description: description || null,
      default_workflow: defaultWorkflow || null,
      is_active: isActive,
      sort_order: sortOrder,
    };

    // 編集・作成で個別にバリデーション＋API呼び出しを実行（union型回避）
    if (isEditing) {
      const result = updateProjectTypeSchema.safeParse(input);
      if (!result.success) {
        const fieldErrors: Record<string, string> = {};
        result.error.issues.forEach((err) => {
          fieldErrors[err.path.join('.')] = err.message;
        });
        setErrors(fieldErrors);
        return;
      }
      updateProjectType.mutate(result.data, { onSuccess: () => onClose() });
    } else {
      const result = createProjectTypeSchema.safeParse(input);
      if (!result.success) {
        const fieldErrors: Record<string, string> = {};
        result.error.issues.forEach((err) => {
          fieldErrors[err.path.join('.')] = err.message;
        });
        setErrors(fieldErrors);
        return;
      }
      createProjectType.mutate(result.data, { onSuccess: () => onClose() });
    }
  };

  return (
    <div style={{ border: '1px solid #ccc', padding: '16px', marginBottom: '16px' }}>
      <h2>{isEditing ? 'プロジェクトタイプ編集' : 'プロジェクトタイプ新規作成'}</h2>
      <form onSubmit={handleSubmit}>
        {/* コード入力欄: 編集時は変更不可 */}
        <div style={fieldStyle}>
          <label htmlFor="code">コード</label>
          <input
            id="code"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            disabled={isEditing}
            required
          />
          {errors.code && <span style={errorStyle}>{errors.code}</span>}
        </div>

        {/* 表示名入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="display_name">表示名</label>
          <input
            id="display_name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            required
          />
          {errors.display_name && <span style={errorStyle}>{errors.display_name}</span>}
        </div>

        {/* 説明入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="description">説明</label>
          <textarea
            id="description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>

        {/* デフォルトワークフロー入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="default_workflow">デフォルトワークフロー</label>
          <input
            id="default_workflow"
            value={defaultWorkflow}
            onChange={(e) => setDefaultWorkflow(e.target.value)}
            placeholder="ワークフロー識別子（省略可）"
          />
        </div>

        {/* アクティブ状態チェックボックス */}
        <div style={fieldStyle}>
          <label>
            <input
              type="checkbox"
              checked={isActive}
              onChange={(e) => setIsActive(e.target.checked)}
            />
            アクティブ
          </label>
        </div>

        {/* 並び順入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="sort_order">並び順</label>
          <input
            id="sort_order"
            type="number"
            value={sortOrder}
            onChange={(e) => setSortOrder(Number(e.target.value))}
            min={0}
          />
        </div>

        {/* 送信・キャンセルボタン */}
        <div style={{ display: 'flex', gap: '8px' }}>
          <button
            type="submit"
            disabled={createProjectType.isPending || updateProjectType.isPending}
          >
            {isEditing ? '更新' : '作成'}
          </button>
          <button type="button" onClick={onClose}>
            キャンセル
          </button>
        </div>

        {/* API エラー表示 */}
        {(createProjectType.error || updateProjectType.error) && (
          <p style={errorStyle}>
            保存に失敗しました:{' '}
            {((createProjectType.error || updateProjectType.error) as Error).message}
          </p>
        )}
      </form>
    </div>
  );
}

// フォームフィールドの共通スタイル
const fieldStyle: React.CSSProperties = {
  marginBottom: '12px',
  display: 'flex',
  flexDirection: 'column',
  gap: '4px',
};

// エラーメッセージのスタイル
const errorStyle: React.CSSProperties = {
  color: 'red',
  fontSize: '0.85em',
};
