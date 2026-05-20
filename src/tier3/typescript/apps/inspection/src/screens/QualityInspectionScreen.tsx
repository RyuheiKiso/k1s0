// k1s0 検査 tier3 — 品質検査結果配信画面
// 製造業 pack 適用例 scenario 9: 品質検査結果配信（v1_event_feed、順序 + replay）
// 品質検査の結果をリアルタイムで配信・一覧表示する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 品質検査結果エンティティの型（tier2 生成 stub の型定義に合わせる）
export interface QualityInspectionResult {
  // 検査結果 ID（UUID）
  readonly inspectionResultId: string;
  // ロット番号（業務キー）
  readonly lotNumber: string;
  // 品目名（表示用）
  readonly itemName: string;
  // 検査日時（ISO 8601 文字列）
  readonly inspectedAt: string;
  // 検査判定（pass / fail / conditional）
  readonly judgement: "pass" | "fail" | "conditional";
  // 測定値（品質特性値）
  readonly measuredValue: number;
  // 規格上限値
  readonly upperLimit: number;
  // 規格下限値
  readonly lowerLimit: number;
  // 担当者表示名（PII は含まない）
  readonly inspectorDisplayName: string;
}

// QualityInspectionScreen のプロパティ型
export interface QualityInspectionScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 検査結果一覧データ（undefined の場合はローディング中）
  readonly results?: readonly QualityInspectionResult[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 検査判定の日本語ラベルを返す
function judgementLabel(judgement: QualityInspectionResult["judgement"]): string {
  // 判定に応じたラベルを返す
  switch (judgement) {
    case "pass":
      // 合格
      return "合格";
    case "fail":
      // 不合格
      return "不合格";
    case "conditional":
      // 条件付き合格
      return "条件付き合格";
    default: {
      // 網羅性チェック（新判定追加時はコンパイルエラーで検知する）
      const _exhaustive: never = judgement;
      return String(_exhaustive);
    }
  }
}

// 品質検査結果配信画面コンポーネント
export function QualityInspectionScreen({
  results,
  isLoading = false,
  errorMessage,
}: QualityInspectionScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="品質検査結果">
        {/* ページ見出し */}
        <h1>品質検査結果配信</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="検査結果を読み込み中">
          <p>品質検査結果を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="品質検査結果">
        {/* ページ見出し */}
        <h1>品質検査結果配信</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 品質検査結果一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="品質検査結果">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>品質検査結果配信</h1>
      {/* 検査結果件数サマリ（aria-live でリアルタイム更新を通知する）*/}
      <p aria-live="polite" aria-label="検査結果件数">
        {results !== undefined ? `${results.length} 件の検査結果` : ""}
      </p>
      {/* 検査結果一覧または空状態 */}
      {results === undefined || results.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="検査結果がありません">
          <p>品質検査結果がありません。</p>
        </div>
      ) : (
        // 検査結果リストを表示する
        <ul aria-label="品質検査結果リスト">
          {results.map((result) => (
            // 検査結果アイテムを key 付きで展開する
            <li
              key={result.inspectionResultId}
              // aria-label でロット番号と判定を説明する
              aria-label={`ロット ${result.lotNumber}（${judgementLabel(result.judgement)}）`}
            >
              {/* ロット番号（見出し h3）*/}
              <h3>{result.lotNumber}</h3>
              {/* 品目名 */}
              <p aria-label="品目">{result.itemName}</p>
              {/* 検査日時 */}
              <p aria-label="検査日時">{result.inspectedAt}</p>
              {/* 検査判定（合否を status で通知する）*/}
              <p role="status" aria-label="検査判定">
                {judgementLabel(result.judgement)}
              </p>
              {/* 測定値と規格範囲 */}
              <p aria-label="測定値">
                {result.measuredValue}（規格: {result.lowerLimit}〜{result.upperLimit}）
              </p>
              {/* 担当者 */}
              <p aria-label="担当者">{result.inspectorDisplayName}</p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
