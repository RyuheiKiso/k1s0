import { useState } from 'react';
import { createCategorySchema, updateCategorySchema } from '../../types/domain-master';
import { useCreateCategory, useUpdateCategory } from '../../hooks/useDomainMaster';
import type { MasterCategory } from '../../types/domain-master';

// カテゴリフォームのProps: categoryがある場合は編集モード
interface CategoryFormProps {
  category?: MasterCategory;
  onClose: () => void;
}

// カテゴリ作成・編集フォームコンポーネント: Zodバリデーション付き
export function CategoryForm({ category, onClose }: CategoryFormProps) {
  const isEditing = !!category;

  // フォーム入力値の状態管理
  const [code, setCode] = useState(category?.code ?? '');
  const [displayName, setDisplayName] = useState(category?.display_name ?? '');
  const [description, setDescription] = useState(category?.description ?? '');
  const [isActive, setIsActive] = useState(category?.is_active ?? true);
  const [sortOrder, setSortOrder] = useState(category?.sort_order ?? 0);
  // バリデーションエラーメッセージ
  const [errors, setErrors] = useState<Record<string, string>>({});

  const createCategory = useCreateCategory();
  const updateCategory = useUpdateCategory(category?.code ?? '');

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    const input = {
      code,
      display_name: displayName,
      description: description || null,
      is_active: isActive,
      sort_order: sortOrder,
    };

    // Zodスキーマでバリデーション実行
    const schema = isEditing ? updateCategorySchema : createCategorySchema;
    const result = schema.safeParse(input);

    if (!result.success) {
      // バリデーションエラーをフィールド別に整理
      const fieldErrors: Record<string, string> = {};
      result.error.errors.forEach((err) => {
        const field = err.path.join('.');
        fieldErrors[field] = err.message;
      });
      setErrors(fieldErrors);
      return;
    }

    // 編集・作成に応じたAPI呼び出し
    const mutation = isEditing ? updateCategory : createCategory;
    mutation.mutate(result.data, {
      onSuccess: () => onClose(),
    });
  };

  return (
    <div style={{ border: '1px solid #ccc', padding: '16px', marginBottom: '16px' }}>
      <h2>{isEditing ? 'カテゴリ編集' : 'カテゴリ新規作成'}</h2>
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
          <button type="submit" disabled={createCategory.isPending || updateCategory.isPending}>
            {isEditing ? '更新' : '作成'}
          </button>
          <button type="button" onClick={onClose}>
            キャンセル
          </button>
        </div>

        {/* API エラー表示 */}
        {(createCategory.error || updateCategory.error) && (
          <p style={errorStyle}>
            保存に失敗しました: {((createCategory.error || updateCategory.error) as Error).message}
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
