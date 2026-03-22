import { useState } from 'react';
import {
  createStatusDefinitionSchema,
  updateStatusDefinitionSchema,
} from '../../types/projectMaster';
import {
  useCreateStatusDefinition,
  useUpdateStatusDefinition,
} from '../../hooks/useStatusDefinitions';
import type { StatusDefinition } from '../../types/projectMaster';

// ステータス定義フォームのProps
interface StatusDefinitionFormProps {
  projectTypeId: string;
  statusDefinition?: StatusDefinition;
  onClose: () => void;
}

// ステータス定義作成・編集フォームコンポーネント: Zodバリデーション付き
export function StatusDefinitionForm({
  projectTypeId,
  statusDefinition,
  onClose,
}: StatusDefinitionFormProps) {
  const isEditing = !!statusDefinition;

  // フォーム入力値の状態管理
  const [code, setCode] = useState(statusDefinition?.code ?? '');
  const [displayName, setDisplayName] = useState(statusDefinition?.display_name ?? '');
  const [description, setDescription] = useState(statusDefinition?.description ?? '');
  const [color, setColor] = useState(statusDefinition?.color ?? '');
  const [allowedTransitions, setAllowedTransitions] = useState(
    statusDefinition?.allowed_transitions?.join(', ') ?? ''
  );
  const [isInitial, setIsInitial] = useState(statusDefinition?.is_initial ?? false);
  const [isTerminal, setIsTerminal] = useState(statusDefinition?.is_terminal ?? false);
  const [sortOrder, setSortOrder] = useState(statusDefinition?.sort_order ?? 0);
  // バリデーションエラーメッセージ
  const [errors, setErrors] = useState<Record<string, string>>({});

  const createStatusDef = useCreateStatusDefinition(projectTypeId);
  const updateStatusDef = useUpdateStatusDefinition(
    projectTypeId,
    statusDefinition?.id ?? ''
  );

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    // カンマ区切りの遷移先コードをパース
    const parsedTransitions = allowedTransitions.trim()
      ? allowedTransitions.split(',').map((s) => s.trim()).filter(Boolean)
      : null;

    const input = {
      code,
      display_name: displayName,
      description: description || null,
      color: color || null,
      allowed_transitions: parsedTransitions,
      is_initial: isInitial,
      is_terminal: isTerminal,
      sort_order: sortOrder,
    };

    // 編集・作成で個別にバリデーション＋API呼び出しを実行（union型回避）
    if (isEditing) {
      const result = updateStatusDefinitionSchema.safeParse(input);
      if (!result.success) {
        const fieldErrors: Record<string, string> = {};
        result.error.issues.forEach((err) => {
          fieldErrors[err.path.join('.')] = err.message;
        });
        setErrors(fieldErrors);
        return;
      }
      updateStatusDef.mutate(result.data, { onSuccess: () => onClose() });
    } else {
      const result = createStatusDefinitionSchema.safeParse(input);
      if (!result.success) {
        const fieldErrors: Record<string, string> = {};
        result.error.issues.forEach((err) => {
          fieldErrors[err.path.join('.')] = err.message;
        });
        setErrors(fieldErrors);
        return;
      }
      createStatusDef.mutate(result.data, { onSuccess: () => onClose() });
    }
  };

  return (
    <div style={{ border: '1px solid #ccc', padding: '16px', marginBottom: '16px' }}>
      <h3>{isEditing ? 'ステータス定義編集' : 'ステータス定義新規作成'}</h3>
      <form onSubmit={handleSubmit}>
        {/* コード入力欄: 編集時は変更不可 */}
        <div style={fieldStyle}>
          <label htmlFor="status-code">コード</label>
          <input
            id="status-code"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            disabled={isEditing}
            required
          />
          {errors.code && <span style={errorStyle}>{errors.code}</span>}
        </div>

        {/* 表示名入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="status-display-name">表示名</label>
          <input
            id="status-display-name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            required
          />
          {errors.display_name && <span style={errorStyle}>{errors.display_name}</span>}
        </div>

        {/* 説明入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="status-description">説明</label>
          <textarea
            id="status-description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>

        {/* 色入力欄: CSSカラーコード */}
        <div style={fieldStyle}>
          <label htmlFor="status-color">色 (CSSカラー)</label>
          <input
            id="status-color"
            value={color}
            onChange={(e) => setColor(e.target.value)}
            placeholder="#3498db"
          />
        </div>

        {/* 許可される遷移先ステータスコードのカンマ区切り入力 */}
        <div style={fieldStyle}>
          <label htmlFor="status-transitions">許可される遷移先 (カンマ区切り)</label>
          <input
            id="status-transitions"
            value={allowedTransitions}
            onChange={(e) => setAllowedTransitions(e.target.value)}
            placeholder="IN_PROGRESS, DONE"
          />
        </div>

        {/* 初期状態フラグ */}
        <div style={fieldStyle}>
          <label>
            <input
              type="checkbox"
              checked={isInitial}
              onChange={(e) => setIsInitial(e.target.checked)}
            />
            初期ステータス
          </label>
        </div>

        {/* 終了状態フラグ */}
        <div style={fieldStyle}>
          <label>
            <input
              type="checkbox"
              checked={isTerminal}
              onChange={(e) => setIsTerminal(e.target.checked)}
            />
            終了ステータス
          </label>
        </div>

        {/* 並び順入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="status-sort-order">並び順</label>
          <input
            id="status-sort-order"
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
            disabled={createStatusDef.isPending || updateStatusDef.isPending}
          >
            {isEditing ? '更新' : '作成'}
          </button>
          <button type="button" onClick={onClose}>
            キャンセル
          </button>
        </div>

        {/* APIエラー表示 */}
        {(createStatusDef.error || updateStatusDef.error) && (
          <p style={errorStyle}>
            保存に失敗しました:{' '}
            {((createStatusDef.error || updateStatusDef.error) as Error).message}
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
