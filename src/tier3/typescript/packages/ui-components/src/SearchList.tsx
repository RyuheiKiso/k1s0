// k1s0 tier3 SPA 検索リストコンポーネント
// 検索フィールド + フィルタリングされたリスト表示を提供する
// WCAG 2.1 AA: role="search" / aria-label / aria-live でアクセシビリティを確保する

// React をインポートする
import React from "react";

// SearchList の各アイテム型（ジェネリック T の id / label を要求する）
export interface SearchListItem {
  // アイテムを識別するユニーク ID
  readonly id: string;
  // 表示ラベル（検索対象となるテキスト）
  readonly label: string;
  // 補足説明（省略可能）
  readonly description?: string;
}

// SearchList に渡すプロパティ型
export interface SearchListProps<T extends SearchListItem> {
  // 全アイテム一覧
  readonly items: readonly T[];
  // 検索クエリ（外部から制御する）
  readonly query: string;
  // 検索クエリ変更コールバック
  readonly onQueryChange: (query: string) => void;
  // アイテム選択コールバック
  readonly onSelect?: (item: T) => void;
  // 検索プレースホルダーテキスト（省略時は "検索..."）
  readonly placeholder?: string;
  // リストの aria-label（省略時は "検索結果")
  readonly listLabel?: string;
}

// SearchList コンポーネント本体
export function SearchList<T extends SearchListItem>({
  items,
  query,
  onQueryChange,
  onSelect,
  placeholder = "検索...",
  listLabel = "検索結果",
}: SearchListProps<T>): React.JSX.Element {
  // クエリでアイテムをフィルタリングする（label と description を対象にする）
  const filteredItems = items.filter((item) => {
    // クエリが空の場合は全件表示する
    if (query.trim() === "") {
      // 空クエリは全件対象とする
      return true;
    }
    // ラベルまたは説明に query が含まれているか確認する（大文字小文字を区別しない）
    const lowerQuery = query.toLowerCase();
    // ラベルで検索する
    const matchesLabel = item.label.toLowerCase().includes(lowerQuery);
    // 説明で検索する（description がある場合のみ）
    const matchesDescription =
      item.description !== undefined &&
      item.description.toLowerCase().includes(lowerQuery);
    // いずれかに一致すれば表示する
    return matchesLabel || matchesDescription;
  });

  // 検索フィールドの変更ハンドラ
  function handleInputChange(e: React.ChangeEvent<HTMLInputElement>): void {
    // 入力値を親コンポーネントに通知する
    onQueryChange(e.currentTarget.value);
  }

  // アイテムクリックハンドラ
  function handleItemClick(item: T): void {
    // onSelect が設定されている場合は選択イベントを発火する
    if (onSelect !== undefined) {
      // 選択されたアイテムを通知する
      onSelect(item);
    }
  }

  // キーボード操作ハンドラ（Enter / Space でアイテムを選択する）
  function handleItemKeyDown(
    e: React.KeyboardEvent<HTMLLIElement>,
    item: T,
  ): void {
    // Enter または Space キーで選択する
    if (e.key === "Enter" || e.key === " ") {
      // イベントのデフォルト動作を防ぐ
      e.preventDefault();
      // 選択ハンドラを呼ぶ
      handleItemClick(item);
    }
  }

  // コンポーネントを描画する
  return (
    // role="search": 検索ランドマークとしてスクリーンリーダーに認識させる
    <div role="search" aria-label={listLabel}>
      {/* 検索入力フィールド */}
      <input
        // type="search": ブラウザの検索 UI 最適化を有効にする
        type="search"
        // 現在のクエリ値を設定する
        value={query}
        // 入力変更ハンドラを接続する
        onChange={handleInputChange}
        // プレースホルダーテキスト
        placeholder={placeholder}
        // aria-label: 検索フィールドの目的をスクリーンリーダーに伝える
        aria-label="検索キーワード入力"
        // aria-controls: このフィールドが制御するリストの ID を指定する
        aria-controls="search-list-results"
      />
      {/* 検索結果件数を aria-live で動的通知する */}
      <p
        // aria-live="polite": 検索結果件数を会話の区切りで読み上げる
        aria-live="polite"
        // aria-atomic="true": テキスト全体を一括読み上げする
        aria-atomic="true"
        // aria-label: 件数情報であることをラベル付けする
        aria-label="検索結果件数"
      >
        {/* 件数テキスト */}
        {filteredItems.length} 件
      </p>
      {/* 検索結果リスト */}
      <ul
        // id: aria-controls から参照される ID
        id="search-list-results"
        // aria-label: リスト全体のラベル
        aria-label={listLabel}
        // role="list": 明示的なリストランドマーク
        role="list"
      >
        {filteredItems.map((item) => (
          // 各アイテムを li として表示する
          <li
            // ユニーク key
            key={item.id}
            // role="listitem": 明示的なリストアイテム
            role="listitem"
            // キーボード操作を有効にするためフォーカス可能にする
            tabIndex={0}
            // aria-label: アイテムの内容をスクリーンリーダーに伝える
            aria-label={
              item.description !== undefined
                ? `${item.label}: ${item.description}`
                : item.label
            }
            // クリックでアイテムを選択する
            onClick={() => { handleItemClick(item); }}
            // キーボードで選択する
            onKeyDown={(e) => { handleItemKeyDown(e, item); }}
          >
            {/* アイテムラベル */}
            <span>{item.label}</span>
            {/* 説明があれば表示する */}
            {item.description !== undefined && (
              // 説明テキスト
              <small aria-label="説明">{item.description}</small>
            )}
          </li>
        ))}
        {/* 結果がない場合のメッセージ */}
        {filteredItems.length === 0 && (
          // 空状態のメッセージ
          <li role="listitem" aria-label="検索結果なし">
            {/* 空状態テキスト */}
            該当する結果がありません
          </li>
        )}
      </ul>
    </div>
  );
}
