// k1s0 tier3 フロア一覧 UI（製造業 pack — フロア管理画面）
// 製造業 pack における「フロア」（工場内の物理的な区画）の一覧を表示する
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

import React from "react";

// フロアエンティティの型（tier2 生成 stub の代替として最小定義）
export interface FloorSummary {
  // フロア ID（UUID）
  readonly floorId: string;
  // フロア名
  readonly name: string;
  // 所属現場 ID
  readonly fieldId: string;
  // 階数（1 以上）
  readonly floorNumber: number;
  // 面積（m2）
  readonly areaSqm: number;
  // 設備数
  readonly equipmentCount: number;
}

// FloorScreen のプロパティ型
export interface FloorScreenProps {
  // フロア一覧データ（undefined の場合はローディング中）
  readonly floors?: readonly FloorSummary[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ
  readonly errorMessage?: string;
}

// フロアリストアイテムコンポーネント
function FloorListItem({ floor }: { readonly floor: FloorSummary }): React.JSX.Element {
  // フロアアイテムを li 要素として返す
  return (
    // li: リストアイテム
    <li aria-label={`フロア: ${floor.name}（${floor.floorNumber} 階）`}>
      {/* フロア名（見出し h3）*/}
      <h3>{floor.name}</h3>
      {/* 階数 */}
      <p aria-label="階数">{floor.floorNumber} 階</p>
      {/* 面積 */}
      <p aria-label="面積">{floor.areaSqm.toFixed(1)} m²</p>
      {/* 設備数 */}
      <p aria-label="設備数">{floor.equipmentCount} 台</p>
    </li>
  );
}

// フロア一覧画面コンポーネント
export function FloorScreen({
  floors,
  isLoading = false,
  errorMessage,
}: FloorScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="フロア一覧">
        {/* ページ見出し */}
        <h1>フロア一覧</h1>
        {/* ローディング状態を通知する */}
        <div role="status" aria-live="polite" aria-label="フロア一覧を読み込み中">
          <p>フロア一覧を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="フロア一覧">
        {/* ページ見出し */}
        <h1>フロア一覧</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // フロア一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="フロア一覧">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>フロア一覧</h1>
      {/* フロア数サマリ */}
      <p aria-live="polite" aria-label="フロア数">
        {floors !== undefined ? `${floors.length} 件のフロア` : ""}
      </p>
      {/* フロアリストまたは空状態 */}
      {floors === undefined || floors.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="フロアが登録されていません">
          <p>フロアが登録されていません。</p>
        </div>
      ) : (
        // フロアリストを表示する
        <ul aria-label="フロアリスト">
          {floors.map((floor) => (
            // フロアアイテムを key 付きで直接展開する（インライン化で TS2322 を回避する）
            <li
              key={floor.floorId}
              aria-label={`フロア: ${floor.name}（${floor.floorNumber} 階）`}
            >
              {/* フロア名（見出し h3）*/}
              <h3>{floor.name}</h3>
              {/* 階数 */}
              <p aria-label="階数">{floor.floorNumber} 階</p>
              {/* 面積 */}
              <p aria-label="面積">{floor.areaSqm.toFixed(1)} m²</p>
              {/* 設備数 */}
              <p aria-label="設備数">{floor.equipmentCount} 台</p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
