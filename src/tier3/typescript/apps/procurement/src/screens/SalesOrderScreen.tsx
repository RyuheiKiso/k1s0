// k1s0 発注 tier3 — 受注 sub 画面
// 製造業 pack 適用例 scenario 7: 受注 sub（基幹 → 製造管理）（v1_event_feed、順序 + replay）
// 基幹システムから連携された受注を一覧表示する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 受注エンティティの型（tier2 生成 stub の型定義に合わせる）
export interface SalesOrderSummary {
  // 受注 ID（UUID）
  readonly salesOrderId: string;
  // 受注番号（業務キー）
  readonly orderNumber: string;
  // 顧客表示名（PII は含まない）
  readonly customerDisplayName: string;
  // 受注日（ISO 8601 文字列）
  readonly orderedAt: string;
  // 受注ステータス（received / in_production / shipped / delivered）
  readonly status: "received" | "in_production" | "shipped" | "delivered";
  // 受注金額合計（通貨単位: 円）
  readonly totalAmountJpy: number;
  // 基幹連携ステータス（基幹システムからの連携状態）
  readonly erpSyncStatus: "synced" | "pending" | "error";
}

// SalesOrderScreen のプロパティ型
export interface SalesOrderScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 受注一覧データ（undefined の場合はローディング中）
  readonly orders?: readonly SalesOrderSummary[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 受注ステータスの日本語ラベルを返す
function salesOrderStatusLabel(status: SalesOrderSummary["status"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "received":
      // 受注済み
      return "受注済み";
    case "in_production":
      // 生産中
      return "生産中";
    case "shipped":
      // 出荷済み
      return "出荷済み";
    case "delivered":
      // 納品完了
      return "納品完了";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 基幹連携ステータスの日本語ラベルを返す
function erpSyncStatusLabel(status: SalesOrderSummary["erpSyncStatus"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "synced":
      // 連携済み
      return "連携済み";
    case "pending":
      // 連携待ち
      return "連携待ち";
    case "error":
      // 連携エラー
      return "連携エラー";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 受注金額を日本円フォーマットで返す
function formatJpy(amount: number): string {
  // Intl.NumberFormat で日本円フォーマットに変換する
  return new Intl.NumberFormat("ja-JP", {
    // 通貨スタイルを指定する
    style: "currency",
    // 日本円を指定する
    currency: "JPY",
  }).format(amount);
}

// 受注 sub 画面コンポーネント
export function SalesOrderScreen({
  orders,
  isLoading = false,
  errorMessage,
}: SalesOrderScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="受注一覧">
        {/* ページ見出し */}
        <h1>受注一覧（基幹連携）</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="受注一覧を読み込み中">
          <p>受注一覧を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="受注一覧">
        {/* ページ見出し */}
        <h1>受注一覧（基幹連携）</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 受注一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="受注一覧">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>受注一覧（基幹連携）</h1>
      {/* 受注数サマリ */}
      <p aria-live="polite" aria-label="受注数">
        {orders !== undefined ? `${orders.length} 件の受注` : ""}
      </p>
      {/* 受注一覧または空状態 */}
      {orders === undefined || orders.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="受注がありません">
          <p>受注がありません。</p>
        </div>
      ) : (
        // 受注リストを表示する
        <ul aria-label="受注リスト">
          {orders.map((order) => (
            // 受注アイテムを key 付きで展開する
            <li
              key={order.salesOrderId}
              // aria-label で受注番号とステータスを説明する
              aria-label={`受注 ${order.orderNumber}（${salesOrderStatusLabel(order.status)}）`}
            >
              {/* 受注番号（見出し h3）*/}
              <h3>{order.orderNumber}</h3>
              {/* 顧客表示名 */}
              <p aria-label="顧客">{order.customerDisplayName}</p>
              {/* 受注日 */}
              <p aria-label="受注日">{order.orderedAt}</p>
              {/* 受注ステータス */}
              <p role="status" aria-label="受注ステータス">
                {salesOrderStatusLabel(order.status)}
              </p>
              {/* 受注金額合計 */}
              <p aria-label="受注金額">{formatJpy(order.totalAmountJpy)}</p>
              {/* 基幹連携ステータス */}
              <p aria-label="基幹連携ステータス">
                {erpSyncStatusLabel(order.erpSyncStatus)}
              </p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
