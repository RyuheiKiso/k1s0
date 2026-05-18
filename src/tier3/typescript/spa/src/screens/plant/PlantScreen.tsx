// k1s0 tier3 植物/工場一覧 UI（設計方針 34: 製造業 pack 適用例）
// manufacturing pack の工場（plant）一覧を表示する画面コンポーネント
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

import React from "react";

// 工場エンティティの型（tier2 生成 stub の代替として最小定義）
// 実際の実装では tier2 生成 stub を使用する（独自再宣言型禁止ポリシーに注意）
export interface PlantSummary {
  // 工場 ID（UUID）
  readonly plantId: string;
  // 工場名
  readonly name: string;
  // 所在地（都市名）
  readonly location: string;
  // アクティブかどうか
  readonly active: boolean;
}

// PlantScreen のプロパティ型
export interface PlantScreenProps {
  // 工場一覧データ（undefined の場合はローディング中）
  readonly plants?: readonly PlantSummary[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 工場一覧アイテムコンポーネント（再利用可能な最小実装）
function PlantListItem({ plant }: { readonly plant: PlantSummary }): React.JSX.Element {
  // 工場アイテムを li 要素として返す
  return (
    // li: リストアイテムのランドマーク
    <li
      // plant ID を key に使用する
      key={plant.plantId}
      // アクティブ状態に応じたクラスを設定する
      aria-label={`工場: ${plant.name}（${plant.active ? "稼働中" : "停止中"}）`}
    >
      {/* 工場名（見出し h3）*/}
      <h3>{plant.name}</h3>
      {/* 所在地 */}
      <p aria-label="所在地">{plant.location}</p>
      {/* アクティブ状態バッジ */}
      <span
        // アクティブ状態に応じた aria-label を設定する
        aria-label={plant.active ? "稼働中" : "停止中"}
        // role="status" で状態を示す
        role="status"
      >
        {plant.active ? "稼働中" : "停止中"}
      </span>
    </li>
  );
}

// ローディング表示コンポーネント
function PlantListLoading(): React.JSX.Element {
  // ローディング状態を aria-live で通知する
  return (
    // aria-live: スクリーンリーダーに動的変化を通知する
    <div role="status" aria-live="polite" aria-label="工場一覧を読み込み中">
      {/* ローディングメッセージ */}
      <p>工場一覧を読み込み中...</p>
    </div>
  );
}

// エラー表示コンポーネント
function PlantListError({ message }: { readonly message: string }): React.JSX.Element {
  // エラーを role="alert" で通知する（スクリーンリーダー即時読み上げ）
  return (
    // role="alert": スクリーンリーダーに即時通知する
    <div role="alert" aria-label="エラー">
      {/* エラーメッセージ */}
      <p>{message}</p>
    </div>
  );
}

// 工場一覧が空の場合の表示コンポーネント
function PlantListEmpty(): React.JSX.Element {
  // 空状態を通知する
  return (
    // status で空状態を通知する
    <div role="status" aria-label="工場が登録されていません">
      {/* 空状態メッセージ */}
      <p>工場が登録されていません。</p>
    </div>
  );
}

// 工場一覧画面コンポーネント（製造業 pack 適用例）
export function PlantScreen({
  plants,
  isLoading = false,
  errorMessage,
}: PlantScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    // ローディング表示を返す
    return (
      // main 要素: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="工場一覧">
        {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
        <h1>工場一覧</h1>
        {/* ローディングコンポーネントを表示する */}
        <PlantListLoading />
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    // エラー表示を返す
    return (
      // main 要素: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="工場一覧">
        {/* ページ見出し */}
        <h1>工場一覧</h1>
        {/* エラーコンポーネントを表示する */}
        <PlantListError message={errorMessage} />
      </main>
    );
  }

  // 工場一覧を表示する
  return (
    // main 要素: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="工場一覧">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件を満たす）*/}
      <h1>工場一覧</h1>
      {/* 工場数のサマリ（aria-live で更新通知する）*/}
      <p aria-live="polite" aria-label="工場数">
        {plants !== undefined ? `${plants.length} 件の工場` : ""}
      </p>
      {/* 工場一覧（空の場合は空状態を表示する）*/}
      {plants === undefined || plants.length === 0 ? (
        // 空状態を表示する
        <PlantListEmpty />
      ) : (
        // 工場リストを表示する
        <ul aria-label="工場リスト">
          {plants.map((plant) => (
            // 工場アイテムを key 付きで直接展開する（PlantListItem 関数を直接呼ぶ）
            // React の key は JSX 要素に対して設定する（型チェックのためインライン化する）
            <li
              key={plant.plantId}
              aria-label={`工場: ${plant.name}（${plant.active ? "稼働中" : "停止中"}）`}
            >
              {/* 工場名（見出し h3）*/}
              <h3>{plant.name}</h3>
              {/* 所在地 */}
              <p aria-label="所在地">{plant.location}</p>
              {/* アクティブ状態バッジ */}
              <span
                aria-label={plant.active ? "稼働中" : "停止中"}
                role="status"
              >
                {plant.active ? "稼働中" : "停止中"}
              </span>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
