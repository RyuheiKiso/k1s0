import type { MasterItem } from '../../types/domain-master';

// ツリーコンポーネントのProps
interface ItemTreeProps {
  items: MasterItem[];
  categoryCode: string;
}

// ツリーノードのProps: 再帰的に子アイテムを表示
interface TreeNodeProps {
  item: MasterItem;
  items: MasterItem[];
  categoryCode: string;
  level: number;
}

// 再帰的ツリーノードコンポーネント: 親子関係を階層表示
function TreeNode({ item, items, categoryCode, level }: TreeNodeProps) {
  // 現在のアイテムを親とする子アイテムを抽出
  const children = items.filter((i) => i.parent_item_id === item.id);

  return (
    <li style={{ listStyle: 'none' }}>
      {/* アイテム情報の表示（インデントで階層を表現） */}
      <div style={{ paddingLeft: `${level * 20}px`, padding: '4px 0' }}>
        <span style={{ fontWeight: children.length > 0 ? 'bold' : 'normal' }}>
          {item.display_name}
        </span>
        <span style={{ color: '#666', marginLeft: '8px' }}>({item.code})</span>
        {!item.is_active && (
          <span style={{ color: 'red', marginLeft: '8px', fontSize: '0.85em' }}>無効</span>
        )}
        <a
          href={`/categories/${categoryCode}/items/${item.code}/versions`}
          style={{ marginLeft: '8px', fontSize: '0.85em' }}
        >
          履歴
        </a>
      </div>
      {/* 子アイテムがある場合は再帰的にレンダリング */}
      {children.length > 0 && (
        <ul style={{ margin: 0, padding: 0 }}>
          {children.map((child) => (
            <TreeNode
              key={child.id}
              item={child}
              items={items}
              categoryCode={categoryCode}
              level={level + 1}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

// アイテム階層ツリーコンポーネント: parent_item_idによる木構造表示
export function ItemTree({ items, categoryCode }: ItemTreeProps) {
  // ルートアイテム（親を持たないアイテム）を抽出
  const rootItems = items.filter((item) => item.parent_item_id === null);

  if (items.length === 0) {
    return <p>アイテムがありません。</p>;
  }

  return (
    <div>
      <h2>階層構造</h2>
      <ul style={{ margin: 0, padding: 0 }}>
        {rootItems.map((item) => (
          <TreeNode
            key={item.id}
            item={item}
            items={items}
            categoryCode={categoryCode}
            level={0}
          />
        ))}
      </ul>
    </div>
  );
}
