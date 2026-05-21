// k1s0 FA tier3 — 生産指示・進捗実績画面
// 製造業 pack 適用例 scenario 3: 生産指示 / 進捗実績（v1_event_feed、順序 + replay）
// 生産指示の一覧と各指示の進捗実績をイベントフィードで表示する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 生産指示エンティティの型（tier2 生成 stub の型定義に合わせる）
export interface ProductionOrder {
  // 生産指示 ID（UUID）
  readonly productionOrderId: string;
  // 生産指示番号（業務キー）
  readonly orderNumber: string;
  // 品目名（表示用）
  readonly itemName: string;
  // 生産ライン名（表示用）
  readonly lineDisplayName: string;
  // 指示数量
  readonly orderedQuantity: number;
  // 完成数量（実績）
  readonly completedQuantity: number;
  // 生産指示ステータス（planned / in_progress / completed / cancelled）
  readonly status: "planned" | "in_progress" | "completed" | "cancelled";
  // 計画開始日時（ISO 8601 文字列）
  readonly plannedStartAt: string;
  // 計画終了日時（ISO 8601 文字列）
  readonly plannedEndAt: string;
}

// ProductionScheduleScreen のプロパティ型
export interface ProductionScheduleScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 生産指示一覧データ（undefined の場合はローディング中）
  readonly orders?: readonly ProductionOrder[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 生産指示ステータスの日本語ラベルを返す
function productionStatusLabel(status: ProductionOrder["status"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "planned":
      // 計画中
      return "計画中";
    case "in_progress":
      // 生産中
      return "生産中";
    case "completed":
      // 完了
      return "完了";
    case "cancelled":
      // 中止
      return "中止";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 達成率を計算して返す（0〜100 のパーセント）
function calcCompletionRate(ordered: number, completed: number): number {
  // 指示数量が 0 の場合は 0 を返す（ゼロ除算を防ぐ）
  if (ordered === 0) {
    return 0;
  }
  // 完成数量 / 指示数量 × 100 で達成率を計算する
  return Math.min(100, Math.round((completed / ordered) * 100));
}

// 生産指示・進捗実績画面コンポーネント
export function ProductionScheduleScreen({
  orders,
  isLoading = false,
  errorMessage,
}: ProductionScheduleScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="生産指示・進捗実績">
        {/* ページ見出し */}
        <h1>生産指示・進捗実績</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="生産指示を読み込み中">
          <p>生産指示を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="生産指示・進捗実績">
        {/* ページ見出し */}
        <h1>生産指示・進捗実績</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 生産指示一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="生産指示・進捗実績">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>生産指示・進捗実績</h1>
      {/* 生産指示件数サマリ */}
      <p aria-live="polite" aria-label="生産指示件数">
        {orders !== undefined ? `${orders.length} 件の生産指示` : ""}
      </p>
      {/* 生産指示一覧または空状態 */}
      {orders === undefined || orders.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="生産指示がありません">
          <p>生産指示がありません。</p>
        </div>
      ) : (
        // 生産指示リストを表示する
        <ul aria-label="生産指示リスト">
          {orders.map((order) => (
            // 生産指示アイテムを key 付きで展開する
            <li
              key={order.productionOrderId}
              // aria-label で指示番号とステータスを説明する
              aria-label={`生産指示 ${order.orderNumber}（${productionStatusLabel(order.status)}）`}
            >
              {/* 生産指示番号（見出し h3）*/}
              <h3>{order.orderNumber}</h3>
              {/* 品目名 */}
              <p aria-label="品目">{order.itemName}</p>
              {/* 生産ライン名 */}
              <p aria-label="ライン">{order.lineDisplayName}</p>
              {/* 指示数量と完成数量 */}
              <p aria-label="数量">
                {order.completedQuantity} / {order.orderedQuantity} 個
              </p>
              {/* 達成率（aria-live でリアルタイム更新を通知する）*/}
              <p aria-label="達成率" aria-live="polite">
                {calcCompletionRate(order.orderedQuantity, order.completedQuantity)}%
              </p>
              {/* 生産指示ステータス */}
              <p role="status" aria-label="ステータス">
                {productionStatusLabel(order.status)}
              </p>
              {/* 計画期間 */}
              <p aria-label="計画期間">
                {order.plannedStartAt} 〜 {order.plannedEndAt}
              </p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
