import { useState } from 'react';
import { createItemSchema, updateItemSchema } from '../../types/domain-master';
import { useCreateItem, useUpdateItem } from '../../hooks/useDomainMaster';
import type { MasterItem } from '../../types/domain-master';

// アイテムフォームのProps
interface ItemFormProps {
  categoryCode: string;
  item?: MasterItem;
  items: MasterItem[];
  onClose: () => void;
}

// アイテム作成・編集フォームコンポーネント: Zodバリデーション付き
export function ItemForm({ categoryCode, item, items, onClose }: ItemFormProps) {
  const isEditing = !!item;

  // フォーム入力値の状態管理
  const [code, setCode] = useState(item?.code ?? '');
  const [displayName, setDisplayName] = useState(item?.display_name ?? '');
  const [description, setDescription] = useState(item?.description ?? '');
  const [parentItemId, setParentItemId] = useState(item?.parent_item_id ?? '');
  const [effectiveFrom, setEffectiveFrom] = useState(item?.effective_from ?? '');
  const [effectiveUntil, setEffectiveUntil] = useState(item?.effective_until ?? '');
  const [isActive, setIsActive] = useState(item?.is_active ?? true);
  const [sortOrder, setSortOrder] = useState(item?.sort_order ?? 0);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const createItem = useCreateItem(categoryCode);
  const updateItem = useUpdateItem(categoryCode, item?.code ?? '');

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    const input = {
      code,
      display_name: displayName,
      description: description || null,
      parent_item_id: parentItemId || null,
      effective_from: effectiveFrom || null,
      effective_until: effectiveUntil || null,
      is_active: isActive,
      sort_order: sortOrder,
    };

    // 編集・作成で個別にバリデーション＋API呼び出しを実行（union型回避）
    if (isEditing) {
      const result = updateItemSchema.safeParse(input);
      if (!result.success) {
        const fieldErrors: Record<string, string> = {};
        result.error.errors.forEach((err) => {
          fieldErrors[err.path.join('.')] = err.message;
        });
        setErrors(fieldErrors);
        return;
      }
      updateItem.mutate(result.data, { onSuccess: () => onClose() });
    } else {
      const result = createItemSchema.safeParse(input);
      if (!result.success) {
        const fieldErrors: Record<string, string> = {};
        result.error.errors.forEach((err) => {
          fieldErrors[err.path.join('.')] = err.message;
        });
        setErrors(fieldErrors);
        return;
      }
      createItem.mutate(result.data, { onSuccess: () => onClose() });
    }
  };

  return (
    <div style={{ border: '1px solid #ccc', padding: '16px', marginBottom: '16px' }}>
      <h2>{isEditing ? 'アイテム編集' : 'アイテム新規作成'}</h2>
      <form onSubmit={handleSubmit}>
        {/* コード入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="item-code">コード</label>
          <input
            id="item-code"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            disabled={isEditing}
            required
          />
          {errors.code && <span style={errorStyle}>{errors.code}</span>}
        </div>

        {/* 表示名入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="item-display-name">表示名</label>
          <input
            id="item-display-name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            required
          />
          {errors.display_name && <span style={errorStyle}>{errors.display_name}</span>}
        </div>

        {/* 説明入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="item-description">説明</label>
          <textarea
            id="item-description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>

        {/* 親アイテム選択: 階層構造を構築するための親子関係指定 */}
        <div style={fieldStyle}>
          <label htmlFor="item-parent">親アイテム</label>
          <select
            id="item-parent"
            value={parentItemId}
            onChange={(e) => setParentItemId(e.target.value)}
          >
            <option value="">なし</option>
            {items
              .filter((i) => i.id !== item?.id)
              .map((i) => (
                <option key={i.id} value={i.id}>
                  {i.display_name} ({i.code})
                </option>
              ))}
          </select>
        </div>

        {/* 有効期間の開始日 */}
        <div style={fieldStyle}>
          <label htmlFor="item-effective-from">有効開始日</label>
          <input
            id="item-effective-from"
            type="date"
            value={effectiveFrom}
            onChange={(e) => setEffectiveFrom(e.target.value)}
          />
        </div>

        {/* 有効期間の終了日 */}
        <div style={fieldStyle}>
          <label htmlFor="item-effective-until">有効終了日</label>
          <input
            id="item-effective-until"
            type="date"
            value={effectiveUntil}
            onChange={(e) => setEffectiveUntil(e.target.value)}
          />
        </div>

        {/* アクティブ状態 */}
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

        {/* 並び順 */}
        <div style={fieldStyle}>
          <label htmlFor="item-sort-order">並び順</label>
          <input
            id="item-sort-order"
            type="number"
            value={sortOrder}
            onChange={(e) => setSortOrder(Number(e.target.value))}
            min={0}
          />
        </div>

        {/* 送信・キャンセルボタン */}
        <div style={{ display: 'flex', gap: '8px' }}>
          <button type="submit" disabled={createItem.isPending || updateItem.isPending}>
            {isEditing ? '更新' : '作成'}
          </button>
          <button type="button" onClick={onClose}>
            キャンセル
          </button>
        </div>

        {/* APIエラー表示 */}
        {(createItem.error || updateItem.error) && (
          <p style={errorStyle}>
            保存に失敗しました: {((createItem.error || updateItem.error) as Error).message}
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
