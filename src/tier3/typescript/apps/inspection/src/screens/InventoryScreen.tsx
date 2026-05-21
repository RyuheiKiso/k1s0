// k1s0 検査 tier3 — 在庫最新値表示画面
// 製造業 pack 適用例 scenario 12: 在庫最新値表示（v1_live_snapshot）
// 在庫の最新スナップショットをリアルタイムで表示する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 在庫スナップショットエンティティの型（tier2 生成 stub の型定義に合わせる）
export interface InventorySnapshot {
  // 在庫 ID（UUID）
  readonly inventoryId: string;
  // 品目コード（業務キー）
  readonly itemCode: string;
  // 品目名（表示用）
  readonly itemName: string;
  // 保管場所表示名
  readonly locationDisplayName: string;
  // 在庫数量（最新値）
  readonly currentQuantity: number;
  // 在庫単位（個 / kg / m 等）
  readonly unit: string;
  // 最終更新日時（ISO 8601 文字列）
  readonly lastUpdatedAt: string;
  // 在庫ステータス（normal / low / critical / out_of_stock）
  readonly stockStatus: "normal" | "low" | "critical" | "out_of_stock";
}

// InventoryScreen のプロパティ型
export interface InventoryScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 在庫スナップショット一覧（undefined の場合はローディング中）
  readonly snapshots?: readonly InventorySnapshot[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 在庫ステータスの日本語ラベルを返す
function stockStatusLabel(status: InventorySnapshot["stockStatus"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "normal":
      // 正常
      return "正常";
    case "low":
      // 少量
      return "少量";
    case "critical":
      // 危険水準
      return "危険水準";
    case "out_of_stock":
      // 在庫切れ
      return "在庫切れ";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 在庫最新値表示画面コンポーネント
export function InventoryScreen({
  snapshots,
  isLoading = false,
  errorMessage,
}: InventoryScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="在庫最新値">
        {/* ページ見出し */}
        <h1>在庫最新値</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="在庫データを読み込み中">
          <p>在庫データを読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="在庫最新値">
        {/* ページ見出し */}
        <h1>在庫最新値</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 在庫切れ・危険水準の件数をカウントする（警報表示用）
  const alertCount =
    snapshots?.filter(
      (s) => s.stockStatus === "out_of_stock" || s.stockStatus === "critical",
    ).length ?? 0;

  // 在庫最新値一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="在庫最新値">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>在庫最新値</h1>
      {/* 在庫切れ・危険水準の警報バナー */}
      {alertCount > 0 && (
        // role="alert" で即時読み上げを行う
        <div role="alert" aria-label={`在庫アラート ${alertCount} 件`} aria-live="assertive">
          <p>在庫切れまたは危険水準の品目が {alertCount} 件あります。</p>
        </div>
      )}
      {/* 在庫品目件数サマリ（aria-live でリアルタイム更新を通知する）*/}
      <p aria-live="polite" aria-label="在庫品目数">
        {snapshots !== undefined ? `${snapshots.length} 品目` : ""}
      </p>
      {/* 在庫一覧または空状態 */}
      {snapshots === undefined || snapshots.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="在庫データがありません">
          <p>在庫データがありません。</p>
        </div>
      ) : (
        // 在庫リストを表示する
        <ul aria-label="在庫リスト">
          {snapshots.map((snapshot) => (
            // 在庫アイテムを key 付きで展開する
            <li
              key={snapshot.inventoryId}
              // aria-label で品目コードと在庫ステータスを説明する
              aria-label={`品目 ${snapshot.itemCode}（${stockStatusLabel(snapshot.stockStatus)}）`}
            >
              {/* 品目コード（見出し h3）*/}
              <h3>{snapshot.itemCode}</h3>
              {/* 品目名 */}
              <p aria-label="品目名">{snapshot.itemName}</p>
              {/* 保管場所 */}
              <p aria-label="保管場所">{snapshot.locationDisplayName}</p>
              {/* 在庫数量（最新値、aria-live でリアルタイム更新を通知する）*/}
              <p aria-label="在庫数量" aria-live="polite">
                {snapshot.currentQuantity} {snapshot.unit}
              </p>
              {/* 在庫ステータス */}
              <p role="status" aria-label="在庫ステータス">
                {stockStatusLabel(snapshot.stockStatus)}
              </p>
              {/* 最終更新日時 */}
              <p aria-label="最終更新">{snapshot.lastUpdatedAt}</p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
