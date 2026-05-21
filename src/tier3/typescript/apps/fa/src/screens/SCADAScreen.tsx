// k1s0 FA tier3 — SCADA テレメトリ収集画面
// 製造業 pack 適用例 scenario 4: SCADA テレメトリ収集（v1_bulk_upload、at-least-once）
// 設備センサーからの大量テレメトリデータを受信・表示する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// SCADA テレメトリポイントの型（tier2 生成 stub の型定義に合わせる）
export interface ScadaDataPoint {
  // データポイント ID（UUID）
  readonly dataPointId: string;
  // タグ名（SCADA システムのタグ識別子）
  readonly tagName: string;
  // 設備番号（対象設備の業務キー）
  readonly equipmentNumber: string;
  // 測定値（数値）
  readonly value: number;
  // 測定単位（℃ / Pa / rpm 等）
  readonly unit: string;
  // 測定品質（good / bad / uncertain）
  readonly quality: "good" | "bad" | "uncertain";
  // 測定タイムスタンプ（ISO 8601 文字列）
  readonly measuredAt: string;
  // アラーム状態（normal / warning / alarm / acknowledged）
  readonly alarmState: "normal" | "warning" | "alarm" | "acknowledged";
}

// SCADAScreen のプロパティ型
export interface SCADAScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // テレメトリデータポイント一覧（undefined の場合はローディング中）
  readonly dataPoints?: readonly ScadaDataPoint[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 測定品質の日本語ラベルを返す
function qualityLabel(quality: ScadaDataPoint["quality"]): string {
  // 品質に応じたラベルを返す
  switch (quality) {
    case "good":
      // 良好
      return "良好";
    case "bad":
      // 不良
      return "不良";
    case "uncertain":
      // 不確実
      return "不確実";
    default: {
      // 網羅性チェック（新品質値追加時はコンパイルエラーで検知する）
      const _exhaustive: never = quality;
      return String(_exhaustive);
    }
  }
}

// アラーム状態の日本語ラベルを返す
function alarmStateLabel(state: ScadaDataPoint["alarmState"]): string {
  // アラーム状態に応じたラベルを返す
  switch (state) {
    case "normal":
      // 正常
      return "正常";
    case "warning":
      // 警告
      return "警告";
    case "alarm":
      // アラーム
      return "アラーム";
    case "acknowledged":
      // 確認済み
      return "確認済み";
    default: {
      // 網羅性チェック（新アラーム状態追加時はコンパイルエラーで検知する）
      const _exhaustive: never = state;
      return String(_exhaustive);
    }
  }
}

// SCADA テレメトリ収集画面コンポーネント
export function SCADAScreen({
  dataPoints,
  isLoading = false,
  errorMessage,
}: SCADAScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="SCADA テレメトリ">
        {/* ページ見出し */}
        <h1>SCADA テレメトリ収集</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="テレメトリデータを読み込み中">
          <p>テレメトリデータを読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="SCADA テレメトリ">
        {/* ページ見出し */}
        <h1>SCADA テレメトリ収集</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // アクティブなアラーム件数をカウントする（警報表示用）
  const activeAlarmCount =
    dataPoints?.filter((d) => d.alarmState === "alarm").length ?? 0;

  // SCADA テレメトリデータ一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="SCADA テレメトリ">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>SCADA テレメトリ収集</h1>
      {/* アクティブアラーム警報バナー */}
      {activeAlarmCount > 0 && (
        // role="alert" で即時読み上げを行う
        <div
          role="alert"
          aria-label={`アクティブアラーム ${activeAlarmCount} 件`}
          aria-live="assertive"
        >
          <p>アクティブなアラームが {activeAlarmCount} 件あります。</p>
        </div>
      )}
      {/* データポイント件数サマリ（aria-live でリアルタイム更新を通知する）*/}
      <p aria-live="polite" aria-label="データポイント件数">
        {dataPoints !== undefined ? `${dataPoints.length} データポイント` : ""}
      </p>
      {/* テレメトリデータ一覧または空状態 */}
      {dataPoints === undefined || dataPoints.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="テレメトリデータがありません">
          <p>テレメトリデータがありません。</p>
        </div>
      ) : (
        // テレメトリデータテーブルを表示する
        <table aria-label="SCADA テレメトリテーブル">
          {/* テーブルキャプション（アクセシビリティ要件） */}
          <caption>SCADA テレメトリデータポイント一覧</caption>
          {/* テーブルヘッダー */}
          <thead>
            <tr>
              {/* タグ名列ヘッダー */}
              <th scope="col">タグ名</th>
              {/* 設備番号列ヘッダー */}
              <th scope="col">設備番号</th>
              {/* 測定値列ヘッダー */}
              <th scope="col">測定値</th>
              {/* 品質列ヘッダー */}
              <th scope="col">品質</th>
              {/* アラーム状態列ヘッダー */}
              <th scope="col">アラーム状態</th>
              {/* 測定タイムスタンプ列ヘッダー */}
              <th scope="col">測定日時</th>
            </tr>
          </thead>
          {/* テーブルボディ */}
          <tbody>
            {dataPoints.map((dp) => (
              // データポイント行を key 付きで展開する
              <tr
                key={dp.dataPointId}
                // aria-label でタグ名とアラーム状態を説明する
                aria-label={`タグ ${dp.tagName}（${alarmStateLabel(dp.alarmState)}）`}
              >
                {/* タグ名セル */}
                <td>{dp.tagName}</td>
                {/* 設備番号セル */}
                <td>{dp.equipmentNumber}</td>
                {/* 測定値セル（単位付き、aria-live でリアルタイム更新を通知する）*/}
                <td aria-label="測定値" aria-live="polite">
                  {dp.value} {dp.unit}
                </td>
                {/* 品質セル */}
                <td aria-label="品質">{qualityLabel(dp.quality)}</td>
                {/* アラーム状態セル */}
                <td role="status" aria-label="アラーム状態">
                  {alarmStateLabel(dp.alarmState)}
                </td>
                {/* 測定タイムスタンプセル */}
                <td>{dp.measuredAt}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </main>
  );
}
