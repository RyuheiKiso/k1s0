// k1s0 発注 tier3 — 検収画面
// 製造業 pack 適用例 scenario 6: 発注 / 検収（v1_event_feed、業務 workflow）の検収サブフロー
// 受領した品目を検収し、発注ステータスを "received" に遷移させる業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 検収対象エンティティの型（tier2 生成 stub の型定義に合わせる）
export interface InspectionAcceptanceItem {
  // 発注 ID（UUID）
  readonly purchaseOrderId: string;
  // 発注番号（業務キー）
  readonly orderNumber: string;
  // 品目名（表示用）
  readonly itemName: string;
  // 発注数量
  readonly orderedQuantity: number;
  // 受領数量（検収時に確定する）
  readonly receivedQuantity: number | null;
  // 検収ステータス（pending / accepted / rejected）
  readonly acceptanceStatus: "pending" | "accepted" | "rejected";
}

// InspectionAcceptanceScreen のプロパティ型
export interface InspectionAcceptanceScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 検収対象一覧（undefined の場合はローディング中）
  readonly items?: readonly InspectionAcceptanceItem[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 検収ステータスの日本語ラベルを返す
function acceptanceStatusLabel(
  status: InspectionAcceptanceItem["acceptanceStatus"],
): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "pending":
      // 検収待ち
      return "検収待ち";
    case "accepted":
      // 検収済み
      return "検収済み";
    case "rejected":
      // 却下
      return "却下";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 検収アイテムコンポーネント
function AcceptanceItemRow({
  item,
}: {
  readonly item: InspectionAcceptanceItem;
}): React.JSX.Element {
  // 検収アイテムを tr 要素として返す
  return (
    // tr: テーブル行
    <tr
      // aria-label で品目名とステータスを説明する
      aria-label={`品目: ${item.itemName}（${acceptanceStatusLabel(item.acceptanceStatus)}）`}
    >
      {/* 発注番号セル */}
      <td>{item.orderNumber}</td>
      {/* 品目名セル */}
      <td>{item.itemName}</td>
      {/* 発注数量セル */}
      <td aria-label="発注数量">{item.orderedQuantity}</td>
      {/* 受領数量セル（null の場合は「未確定」と表示する）*/}
      <td aria-label="受領数量">
        {item.receivedQuantity !== null ? item.receivedQuantity : "未確定"}
      </td>
      {/* 検収ステータスセル */}
      <td role="status" aria-label="検収ステータス">
        {acceptanceStatusLabel(item.acceptanceStatus)}
      </td>
    </tr>
  );
}

// 検収画面コンポーネント
export function InspectionAcceptanceScreen({
  items,
  isLoading = false,
  errorMessage,
}: InspectionAcceptanceScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="検収">
        {/* ページ見出し */}
        <h1>検収</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="検収一覧を読み込み中">
          <p>検収一覧を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="検収">
        {/* ページ見出し */}
        <h1>検収</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 検収一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="検収">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>検収</h1>
      {/* 検収件数サマリ */}
      <p aria-live="polite" aria-label="検収件数">
        {items !== undefined ? `${items.length} 件の検収対象` : ""}
      </p>
      {/* 検収一覧テーブルまたは空状態 */}
      {items === undefined || items.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="検収対象がありません">
          <p>検収対象がありません。</p>
        </div>
      ) : (
        // 検収一覧テーブルを表示する
        <table aria-label="検収一覧テーブル">
          {/* テーブルキャプション（アクセシビリティ要件） */}
          <caption>検収対象品目一覧</caption>
          {/* テーブルヘッダー */}
          <thead>
            <tr>
              {/* 発注番号列ヘッダー */}
              <th scope="col">発注番号</th>
              {/* 品目名列ヘッダー */}
              <th scope="col">品目名</th>
              {/* 発注数量列ヘッダー */}
              <th scope="col">発注数量</th>
              {/* 受領数量列ヘッダー */}
              <th scope="col">受領数量</th>
              {/* 検収ステータス列ヘッダー */}
              <th scope="col">検収ステータス</th>
            </tr>
          </thead>
          {/* テーブルボディ */}
          <tbody>
            {items.map((item) => (
              // 検収アイテム行を key 付きで展開する
              <AcceptanceItemRow
                key={item.purchaseOrderId}
                // 検収アイテムデータを渡す
                item={item}
              />
            ))}
          </tbody>
        </table>
      )}
    </main>
  );
}
