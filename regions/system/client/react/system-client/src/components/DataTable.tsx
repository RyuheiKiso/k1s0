import { useState, useMemo, type ReactNode } from 'react';

// テーブルカラムの定義
export interface DataTableColumn<T> {
  // カラムの識別キー
  key: string;
  // カラムのヘッダー表示ラベル
  header: string;
  // 行データからセルの値を取得する関数
  accessor: (row: T) => ReactNode;
  // ソート時に使用する値を取得する関数（未指定の場合ソート不可）
  sortValue?: (row: T) => string | number;
  // フィルター時に使用する値を取得する関数（未指定の場合フィルター対象外）
  filterValue?: (row: T) => string;
}

// DataTableのProps定義
interface DataTableProps<T> {
  // テーブルに表示するデータの配列
  data: T[];
  // カラム定義の配列
  columns: DataTableColumn<T>[];
  // 行の一意キーを取得する関数
  rowKey: (row: T) => string;
  // 行クリック時のコールバック（任意）
  onRowClick?: (row: T) => void;
  // フィルター入力のプレースホルダー（任意）
  filterPlaceholder?: string;
  // テーブルのaria-label（任意）
  ariaLabel?: string;
  // データが空の場合のメッセージ（任意）
  emptyMessage?: string;
}

// ソート方向の型
type SortDirection = 'asc' | 'desc';

// 汎用データテーブルコンポーネント: ソートとフィルター機能を提供
export function DataTable<T>({
  data,
  columns,
  rowKey,
  onRowClick,
  filterPlaceholder = '検索...',
  ariaLabel = 'データテーブル',
  emptyMessage = 'データがありません。',
}: DataTableProps<T>) {
  // フィルターテキストの状態
  const [filterText, setFilterText] = useState('');
  // ソート対象のカラムキー
  const [sortKey, setSortKey] = useState<string | null>(null);
  // ソート方向
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');

  // フィルター適用後のデータ
  const filteredData = useMemo(() => {
    if (!filterText) return data;
    const lower = filterText.toLowerCase();
    return data.filter((row) =>
      columns.some((col) => col.filterValue?.(row)?.toLowerCase().includes(lower))
    );
  }, [data, filterText, columns]);

  // ソート適用後のデータ
  const sortedData = useMemo(() => {
    if (!sortKey) return filteredData;
    const col = columns.find((c) => c.key === sortKey);
    if (!col?.sortValue) return filteredData;
    const sorted = [...filteredData].sort((a, b) => {
      const aVal = col.sortValue!(a);
      const bVal = col.sortValue!(b);
      if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1;
      if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });
    return sorted;
  }, [filteredData, sortKey, sortDirection, columns]);

  // ソートカラムのクリックハンドラ: 同じカラムなら方向を切り替え、異なるカラムなら昇順で開始
  const handleSort = (key: string) => {
    const col = columns.find((c) => c.key === key);
    if (!col?.sortValue) return;
    if (sortKey === key) {
      setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortKey(key);
      setSortDirection('asc');
    }
  };

  // フィルター可能なカラムが存在するかどうか
  const hasFilterableColumns = columns.some((col) => col.filterValue);

  return (
    <div>
      {/* フィルター入力欄（フィルター可能なカラムがある場合のみ表示） */}
      {hasFilterableColumns && (
        <div style={{ marginBottom: '12px' }}>
          <input
            type="text"
            value={filterText}
            onChange={(e) => setFilterText(e.target.value)}
            placeholder={filterPlaceholder}
            aria-label={filterPlaceholder}
          />
        </div>
      )}

      {/* データテーブル */}
      <table style={{ width: '100%', borderCollapse: 'collapse' }} aria-label={ariaLabel}>
        <thead>
          <tr>
            {columns.map((col) => (
              <th
                key={col.key}
                style={{
                  borderBottom: '2px solid #ccc',
                  padding: '8px',
                  textAlign: 'left',
                  cursor: col.sortValue ? 'pointer' : 'default',
                }}
                onClick={() => handleSort(col.key)}
                aria-sort={
                  sortKey === col.key
                    ? sortDirection === 'asc'
                      ? 'ascending'
                      : 'descending'
                    : undefined
                }
              >
                {col.header}
                {/* ソート方向インジケーター */}
                {sortKey === col.key && (sortDirection === 'asc' ? ' ▲' : ' ▼')}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sortedData.map((row) => (
            <tr
              key={rowKey(row)}
              onClick={onRowClick ? () => onRowClick(row) : undefined}
              style={{ cursor: onRowClick ? 'pointer' : 'default' }}
              // クリック可能な行には role="row" を維持しつつ tabIndex とキーボード操作を付与する
              // role="button" は <tr> に不適切なため使用しない（アクセシビリティ準拠）
              tabIndex={onRowClick ? 0 : undefined}
              aria-label={onRowClick ? `行を選択: ${rowKey(row)}` : undefined}
              onKeyDown={
                onRowClick
                  ? (e) => {
                      if (e.key === 'Enter') {
                        onRowClick(row);
                      } else if (e.key === ' ') {
                        // Space キーによるデフォルトスクロール動作を防止してクリックイベントを発火する
                        e.preventDefault();
                        onRowClick(row);
                      }
                    }
                  : undefined
              }
            >
              {columns.map((col) => (
                <td
                  key={col.key}
                  style={{ borderBottom: '1px solid #eee', padding: '8px' }}
                >
                  {col.accessor(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {sortedData.length === 0 && <p>{emptyMessage}</p>}
    </div>
  );
}
