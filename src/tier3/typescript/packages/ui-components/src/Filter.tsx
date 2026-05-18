// k1s0 tier3 SPA フィルターコンポーネント
// 一覧画面のフィルタリング条件を入力 UI として提供する
// WCAG 2.1 AA: role="group" / fieldset / aria-label でアクセシビリティを確保する

// React をインポートする
import React from "react";

// フィルターオプションの型
export interface FilterOption {
  // オプションの値（フィルタリングに使用する）
  readonly value: string;
  // 表示ラベル
  readonly label: string;
}

// Filter に渡すプロパティ型
export interface FilterProps {
  // フィルターのラベル（fieldset の legend に使用する）
  readonly legend: string;
  // 選択肢一覧
  readonly options: readonly FilterOption[];
  // 現在の選択値（複数選択可）
  readonly selectedValues: readonly string[];
  // 選択変更コールバック
  readonly onSelectionChange: (selectedValues: readonly string[]) => void;
}

// Filter コンポーネント本体
export function Filter({
  legend,
  options,
  selectedValues,
  onSelectionChange,
}: FilterProps): React.JSX.Element {
  // チェックボックス変更ハンドラ
  function handleChange(value: string, checked: boolean): void {
    // チェックされた場合は選択値に追加する
    if (checked) {
      // 既存選択値に新しい値を追加する
      onSelectionChange([...selectedValues, value]);
    } else {
      // チェックが外れた場合は選択値から除去する
      onSelectionChange(selectedValues.filter((v) => v !== value));
    }
  }

  // フィルターを描画する
  return (
    // fieldset: 関連するチェックボックスをグループ化する（WCAG 1.3.1 要件）
    <fieldset>
      {/* legend: フィルターグループの説明（スクリーンリーダーが読み上げる）*/}
      <legend>{legend}</legend>
      {/* 選択肢一覧を ul として表示する */}
      <ul
        // role="list": 明示的なリスト
        role="list"
        // aria-label: リスト全体のラベル
        aria-label={`${legend} フィルター選択肢`}
      >
        {options.map((option) => {
          // 現在の選択状態を確認する
          const isChecked = selectedValues.includes(option.value);
          // チェックボックスの ID を生成する（label の htmlFor と一致させる）
          const checkboxId = `filter-${legend}-${option.value}`;
          // 各オプションをリストアイテムとして表示する
          return (
            <li key={option.value} role="listitem">
              {/* チェックボックス入力（label と紐付ける）*/}
              <input
                // type="checkbox": 複数選択可能にする
                type="checkbox"
                // id: label の htmlFor と一致させる（WCAG 1.3.1 要件）
                id={checkboxId}
                // 現在のチェック状態を設定する
                checked={isChecked}
                // 変更ハンドラを接続する
                onChange={(e) => { handleChange(option.value, e.currentTarget.checked); }}
                // value: フォーム送信時に使用する値
                value={option.value}
              />
              {/* ラベル（htmlFor で対応するチェックボックスと紐付ける）*/}
              <label
                // htmlFor: チェックボックスの id と一致させる
                htmlFor={checkboxId}
              >
                {/* 選択肢のラベルテキスト */}
                {option.label}
              </label>
            </li>
          );
        })}
      </ul>
      {/* 選択件数を aria-live で通知する */}
      <p
        // aria-live="polite": 選択変更を会話の区切りで読み上げる
        aria-live="polite"
        // aria-label: 選択件数情報であることをラベル付けする
        aria-label={`${legend} 選択件数`}
      >
        {/* 選択件数テキスト */}
        {selectedValues.length} 件選択中
      </p>
    </fieldset>
  );
}
