// k1s0 検査 tier3 — 不良票・警報配信画面
// 製造業 pack 適用例 scenario 10: 不良票 / 警報配信（v1_alert、低 lag + replay）
// 不良発生の警報をリアルタイムで受信・表示し、不良票の一覧管理を行う業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 不良票エンティティの型（tier2 生成 stub の型定義に合わせる）
export interface DefectRecord {
  // 不良票 ID（UUID）
  readonly defectRecordId: string;
  // 不良票番号（業務キー）
  readonly defectNumber: string;
  // ロット番号（対象ロット）
  readonly lotNumber: string;
  // 不良発生日時（ISO 8601 文字列）
  readonly detectedAt: string;
  // 不良種別（dimensional / surface / functional / material）
  readonly defectType: "dimensional" | "surface" | "functional" | "material";
  // 重大度（critical / major / minor）
  readonly severity: "critical" | "major" | "minor";
  // 不良数量
  readonly defectQuantity: number;
  // 対応状態（open / investigating / resolved / closed）
  readonly status: "open" | "investigating" | "resolved" | "closed";
}

// DefectAlertScreen のプロパティ型
export interface DefectAlertScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 不良票一覧データ（undefined の場合はローディング中）
  readonly defects?: readonly DefectRecord[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 不良種別の日本語ラベルを返す
function defectTypeLabel(type: DefectRecord["defectType"]): string {
  // 不良種別に応じたラベルを返す
  switch (type) {
    case "dimensional":
      // 寸法不良
      return "寸法不良";
    case "surface":
      // 表面不良
      return "表面不良";
    case "functional":
      // 機能不良
      return "機能不良";
    case "material":
      // 材料不良
      return "材料不良";
    default: {
      // 網羅性チェック（新不良種別追加時はコンパイルエラーで検知する）
      const _exhaustive: never = type;
      return String(_exhaustive);
    }
  }
}

// 重大度の日本語ラベルを返す
function severityLabel(severity: DefectRecord["severity"]): string {
  // 重大度に応じたラベルを返す
  switch (severity) {
    case "critical":
      // 重大
      return "重大";
    case "major":
      // 主要
      return "主要";
    case "minor":
      // 軽微
      return "軽微";
    default: {
      // 網羅性チェック（新重大度追加時はコンパイルエラーで検知する）
      const _exhaustive: never = severity;
      return String(_exhaustive);
    }
  }
}

// 対応状態の日本語ラベルを返す
function defectStatusLabel(status: DefectRecord["status"]): string {
  // 対応状態に応じたラベルを返す
  switch (status) {
    case "open":
      // 未対応
      return "未対応";
    case "investigating":
      // 調査中
      return "調査中";
    case "resolved":
      // 解決済み
      return "解決済み";
    case "closed":
      // 完了
      return "完了";
    default: {
      // 網羅性チェック（新状態追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 不良票・警報配信画面コンポーネント
export function DefectAlertScreen({
  defects,
  isLoading = false,
  errorMessage,
}: DefectAlertScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="不良票・警報">
        {/* ページ見出し */}
        <h1>不良票・警報配信</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="不良票を読み込み中">
          <p>不良票・警報を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="不良票・警報">
        {/* ページ見出し */}
        <h1>不良票・警報配信</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 重大度 critical の不良票数をカウントする（警報表示用）
  const criticalCount = defects?.filter((d) => d.severity === "critical").length ?? 0;

  // 不良票一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="不良票・警報">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>不良票・警報配信</h1>
      {/* 重大不良警報バナー（critical が存在する場合のみ表示する）*/}
      {criticalCount > 0 && (
        // role="alert" で即時読み上げを行う（低 lag 要件に対応する）
        <div role="alert" aria-label={`重大不良 ${criticalCount} 件`} aria-live="assertive">
          <p>重大不良が {criticalCount} 件発生しています。即時対応してください。</p>
        </div>
      )}
      {/* 不良票件数サマリ */}
      <p aria-live="polite" aria-label="不良票件数">
        {defects !== undefined ? `${defects.length} 件の不良票` : ""}
      </p>
      {/* 不良票一覧または空状態 */}
      {defects === undefined || defects.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="不良票がありません">
          <p>不良票がありません。</p>
        </div>
      ) : (
        // 不良票リストを表示する
        <ul aria-label="不良票リスト">
          {defects.map((defect) => (
            // 不良票アイテムを key 付きで展開する
            <li
              key={defect.defectRecordId}
              // aria-label で不良票番号と重大度を説明する
              aria-label={`不良票 ${defect.defectNumber}（${severityLabel(defect.severity)}）`}
            >
              {/* 不良票番号（見出し h3）*/}
              <h3>{defect.defectNumber}</h3>
              {/* ロット番号 */}
              <p aria-label="ロット番号">{defect.lotNumber}</p>
              {/* 不良発生日時 */}
              <p aria-label="発生日時">{defect.detectedAt}</p>
              {/* 不良種別 */}
              <p aria-label="不良種別">{defectTypeLabel(defect.defectType)}</p>
              {/* 重大度 */}
              <p role="status" aria-label="重大度">{severityLabel(defect.severity)}</p>
              {/* 不良数量 */}
              <p aria-label="不良数量">{defect.defectQuantity} 個</p>
              {/* 対応状態 */}
              <p aria-label="対応状態">{defectStatusLabel(defect.status)}</p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
