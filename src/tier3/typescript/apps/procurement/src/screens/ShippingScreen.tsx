// k1s0 発注 tier3 — 出荷指示双方向確認画面
// 製造業 pack 適用例 scenario 8: 出荷指示双方向確認（v1_interactive、双方向確認）
// 出荷指示に対して出荷担当者と受領担当者が双方向に確認する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 出荷指示エンティティの型（tier2 生成 stub の型定義に合わせる）
export interface ShippingInstruction {
  // 出荷指示 ID（UUID）
  readonly shippingInstructionId: string;
  // 出荷番号（業務キー）
  readonly shippingNumber: string;
  // 出荷先表示名（PII は含まない）
  readonly destinationDisplayName: string;
  // 出荷予定日（ISO 8601 文字列）
  readonly scheduledAt: string;
  // 出荷ステータス（pending / confirmed_by_shipper / confirmed_by_receiver / shipped）
  readonly status: "pending" | "confirmed_by_shipper" | "confirmed_by_receiver" | "shipped";
  // 品目数（梱包数）
  readonly packageCount: number;
}

// ShippingScreen のプロパティ型
export interface ShippingScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 出荷指示一覧データ（undefined の場合はローディング中）
  readonly instructions?: readonly ShippingInstruction[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 出荷ステータスの日本語ラベルを返す
function shippingStatusLabel(status: ShippingInstruction["status"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "pending":
      // 確認待ち
      return "確認待ち";
    case "confirmed_by_shipper":
      // 出荷側確認済み
      return "出荷側確認済み";
    case "confirmed_by_receiver":
      // 受領側確認済み
      return "受領側確認済み";
    case "shipped":
      // 出荷済み
      return "出荷済み";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 出荷指示画面コンポーネント
export function ShippingScreen({
  instructions,
  isLoading = false,
  errorMessage,
}: ShippingScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="出荷指示確認">
        {/* ページ見出し */}
        <h1>出荷指示双方向確認</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="出荷指示一覧を読み込み中">
          <p>出荷指示一覧を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="出荷指示確認">
        {/* ページ見出し */}
        <h1>出荷指示双方向確認</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 出荷指示一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="出荷指示確認">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>出荷指示双方向確認</h1>
      {/* 出荷件数サマリ */}
      <p aria-live="polite" aria-label="出荷指示件数">
        {instructions !== undefined ? `${instructions.length} 件の出荷指示` : ""}
      </p>
      {/* 出荷指示一覧または空状態 */}
      {instructions === undefined || instructions.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="出荷指示がありません">
          <p>出荷指示がありません。</p>
        </div>
      ) : (
        // 出荷指示リストを表示する
        <ul aria-label="出荷指示リスト">
          {instructions.map((instr) => (
            // 出荷指示アイテムを key 付きで展開する
            <li
              key={instr.shippingInstructionId}
              // aria-label で出荷番号とステータスを説明する
              aria-label={`出荷指示 ${instr.shippingNumber}（${shippingStatusLabel(instr.status)}）`}
            >
              {/* 出荷番号（見出し h3）*/}
              <h3>{instr.shippingNumber}</h3>
              {/* 出荷先 */}
              <p aria-label="出荷先">{instr.destinationDisplayName}</p>
              {/* 出荷予定日 */}
              <p aria-label="出荷予定日">{instr.scheduledAt}</p>
              {/* 出荷ステータス（双方向確認状態を表示する）*/}
              <p role="status" aria-label="確認ステータス">
                {shippingStatusLabel(instr.status)}
              </p>
              {/* 梱包数 */}
              <p aria-label="梱包数">{instr.packageCount} 梱包</p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
