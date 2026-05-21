// k1s0 発注 tier3 — 発注一覧・発注詳細画面
// 製造業 pack 適用例 scenario 6: 発注 / 検収（v1_event_feed、業務 workflow）
// tier2 pack から発注データを fetch して一覧表示 + 詳細表示を行う
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React, { useState } from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 発注エンティティの型（tier2 生成 stub の型定義に合わせる）
export interface PurchaseOrderSummary {
  // 発注 ID（UUID）
  readonly purchaseOrderId: string;
  // 発注番号（業務キー）
  readonly orderNumber: string;
  // 発注先業者名（表示名のみ、PII は含まない）
  readonly supplierDisplayName: string;
  // 発注日（ISO 8601 文字列）
  readonly orderedAt: string;
  // 発注ステータス（draft / submitted / approved / received / closed）
  readonly status: "draft" | "submitted" | "approved" | "received" | "closed";
  // 発注金額合計（通貨単位: 円）
  readonly totalAmountJpy: number;
}

// PurchaseOrderScreen のプロパティ型
export interface PurchaseOrderScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 発注一覧データ（undefined の場合はローディング中）
  readonly orders?: readonly PurchaseOrderSummary[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 発注ステータスの日本語ラベルを返す
function orderStatusLabel(status: PurchaseOrderSummary["status"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "draft":
      // 下書き
      return "下書き";
    case "submitted":
      // 提出済み
      return "提出済み";
    case "approved":
      // 承認済み
      return "承認済み";
    case "received":
      // 検収済み
      return "検収済み";
    case "closed":
      // 完了
      return "完了";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 発注金額を日本円フォーマットで返す
function formatJpy(amount: number): string {
  // Intl.NumberFormat で日本円フォーマットに変換する
  return new Intl.NumberFormat("ja-JP", {
    // 通貨スタイルを指定する
    style: "currency",
    // 日本円を指定する
    currency: "JPY",
  }).format(amount);
}

// 発注リストアイテムコンポーネント
function PurchaseOrderListItem({
  order,
  onSelect,
}: {
  readonly order: PurchaseOrderSummary;
  // 発注選択ハンドラ（詳細表示用）
  readonly onSelect: (orderId: string) => void;
}): React.JSX.Element {
  // 発注アイテムを li 要素として返す
  return (
    // li: リストアイテム
    <li
      // aria-label で発注番号とステータスを説明する
      aria-label={`発注 ${order.orderNumber}（${orderStatusLabel(order.status)}）`}
    >
      {/* 発注番号（見出し h3）*/}
      <h3>{order.orderNumber}</h3>
      {/* 発注先業者名 */}
      <p aria-label="発注先">{order.supplierDisplayName}</p>
      {/* 発注日 */}
      <p aria-label="発注日">{order.orderedAt}</p>
      {/* 発注ステータス */}
      <p role="status" aria-label="ステータス">{orderStatusLabel(order.status)}</p>
      {/* 発注金額合計 */}
      <p aria-label="発注金額">{formatJpy(order.totalAmountJpy)}</p>
      {/* 詳細表示ボタン */}
      <button
        // クリック時に発注 ID を上位コンポーネントへ通知する
        onClick={() => { onSelect(order.purchaseOrderId); }}
        // aria-label でボタンの目的を説明する
        aria-label={`発注 ${order.orderNumber} の詳細を表示する`}
        // type="button" を明示して submit と区別する
        type="button"
      >
        詳細
      </button>
    </li>
  );
}

// 発注詳細パネルコンポーネント（最小実装）
function PurchaseOrderDetail({
  order,
  onClose,
}: {
  readonly order: PurchaseOrderSummary;
  // 詳細パネルを閉じるハンドラ
  readonly onClose: () => void;
}): React.JSX.Element {
  // 詳細パネルを section 要素として返す
  return (
    // section: 詳細コンテンツの region ランドマーク
    <section
      // role="region" で詳細パネルを示す
      aria-label={`発注 ${order.orderNumber} 詳細`}
    >
      {/* 詳細見出し */}
      <h2>発注詳細: {order.orderNumber}</h2>
      {/* 発注先業者名 */}
      <p aria-label="発注先">{order.supplierDisplayName}</p>
      {/* 発注日 */}
      <p aria-label="発注日">{order.orderedAt}</p>
      {/* ステータス */}
      <p role="status" aria-label="ステータス">{orderStatusLabel(order.status)}</p>
      {/* 発注金額合計 */}
      <p aria-label="発注金額合計">{formatJpy(order.totalAmountJpy)}</p>
      {/* 閉じるボタン */}
      <button
        // クリック時に詳細パネルを閉じる
        onClick={onClose}
        // aria-label でボタンの目的を説明する
        aria-label="発注詳細を閉じる"
        // type="button" を明示する
        type="button"
      >
        閉じる
      </button>
    </section>
  );
}

// 発注一覧・発注詳細画面コンポーネント
export function PurchaseOrderScreen({
  orders,
  isLoading = false,
  errorMessage,
}: PurchaseOrderScreenProps): React.JSX.Element {
  // 選択中の発注 ID を state として保持する（null の場合は詳細非表示）
  const [selectedOrderId, setSelectedOrderId] = useState<string | null>(null);

  // 選択中の発注データを orders から取得する
  const selectedOrder =
    selectedOrderId !== null
      ? (orders?.find((o) => o.purchaseOrderId === selectedOrderId) ?? null)
      : null;

  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="発注一覧">
        {/* ページ見出し */}
        <h1>発注一覧</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="発注一覧を読み込み中">
          <p>発注一覧を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="発注一覧">
        {/* ページ見出し */}
        <h1>発注一覧</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 発注一覧と詳細を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="発注一覧">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>発注一覧</h1>
      {/* 発注数サマリ（aria-live で動的更新を通知する）*/}
      <p aria-live="polite" aria-label="発注数">
        {orders !== undefined ? `${orders.length} 件の発注` : ""}
      </p>
      {/* 発注一覧または空状態 */}
      {orders === undefined || orders.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="発注が登録されていません">
          <p>発注が登録されていません。</p>
        </div>
      ) : (
        // 発注リストを表示する
        <ul aria-label="発注リスト">
          {orders.map((order) => (
            // 発注アイテムを key 付きで展開する
            <PurchaseOrderListItem
              key={order.purchaseOrderId}
              // 発注データを渡す
              order={order}
              // 選択ハンドラを渡す
              onSelect={setSelectedOrderId}
            />
          ))}
        </ul>
      )}
      {/* 選択中の発注詳細パネルを表示する（詳細が選択されている場合のみ）*/}
      {selectedOrder !== null && (
        // 発注詳細パネルをレンダリングする
        <PurchaseOrderDetail
          // 選択中の発注データを渡す
          order={selectedOrder}
          // 閉じるハンドラで選択状態をリセットする
          onClose={() => { setSelectedOrderId(null); }}
        />
      )}
    </main>
  );
}
