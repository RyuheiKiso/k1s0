// k1s0 FA tier3 — 計量装置連続データ画面
// 製造業 pack 適用例 scenario 5: 計量装置連続データ（v1_bulk_upload、continuous push）
// 既存 spa/src/screens/floor/FloorScreen.tsx のフロア管理ロジックを継承・活用する
// 計量装置から連続的に収集されるデータをリアルタイムで表示する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 計量装置データポイントの型（tier2 生成 stub の型定義に合わせる）
export interface GaugeDataPoint {
  // データポイント ID（UUID）
  readonly dataPointId: string;
  // 計量装置番号（業務キー）
  readonly gaugeNumber: string;
  // 計量装置名（表示用）
  readonly gaugeName: string;
  // 計量値（連続データの最新値）
  readonly measuredValue: number;
  // 単位（g / kg / t 等）
  readonly unit: string;
  // 品目名（計量対象品目）
  readonly itemName: string;
  // 測定タイムスタンプ（ISO 8601 文字列）
  readonly measuredAt: string;
  // 上限アラーム閾値
  readonly upperAlarmThreshold: number;
  // 下限アラーム閾値
  readonly lowerAlarmThreshold: number;
  // アラーム状態（normal / warning / alarm）
  readonly alarmState: "normal" | "warning" | "alarm";
  // フロア情報（FloorScreen から活用する: フロア番号）
  readonly floorNumber: number;
}

// GaugeScreen のプロパティ型
export interface GaugeScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 計量データポイント一覧（undefined の場合はローディング中）
  readonly dataPoints?: readonly GaugeDataPoint[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// アラーム状態の日本語ラベルを返す
function gaugeAlarmLabel(state: GaugeDataPoint["alarmState"]): string {
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
    default: {
      // 網羅性チェック（新アラーム状態追加時はコンパイルエラーで検知する）
      const _exhaustive: never = state;
      return String(_exhaustive);
    }
  }
}

// 計量装置連続データ画面コンポーネント
export function GaugeScreen({
  dataPoints,
  isLoading = false,
  errorMessage,
}: GaugeScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="計量装置連続データ">
        {/* ページ見出し */}
        <h1>計量装置連続データ</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="計量データを読み込み中">
          <p>計量データを読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="計量装置連続データ">
        {/* ページ見出し */}
        <h1>計量装置連続データ</h1>
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

  // 計量装置連続データ一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="計量装置連続データ">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>計量装置連続データ</h1>
      {/* アクティブアラーム警報バナー */}
      {activeAlarmCount > 0 && (
        // role="alert" で即時読み上げを行う
        <div
          role="alert"
          aria-label={`計量アラーム ${activeAlarmCount} 件`}
          aria-live="assertive"
        >
          <p>計量アラームが {activeAlarmCount} 件あります。</p>
        </div>
      )}
      {/* データポイント件数サマリ（aria-live でリアルタイム更新を通知する）*/}
      <p aria-live="polite" aria-label="計量装置件数">
        {dataPoints !== undefined ? `${dataPoints.length} 台の計量装置` : ""}
      </p>
      {/* 計量データ一覧または空状態 */}
      {dataPoints === undefined || dataPoints.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="計量データがありません">
          <p>計量データがありません。</p>
        </div>
      ) : (
        // 計量データリストを表示する
        <ul aria-label="計量装置リスト">
          {dataPoints.map((dp) => (
            // 計量データアイテムを key 付きで展開する
            <li
              key={dp.dataPointId}
              // aria-label で計量装置番号とアラーム状態を説明する
              aria-label={`計量装置 ${dp.gaugeNumber}（${gaugeAlarmLabel(dp.alarmState)}）`}
            >
              {/* 計量装置番号（見出し h3）*/}
              <h3>{dp.gaugeNumber}</h3>
              {/* 計量装置名 */}
              <p aria-label="装置名">{dp.gaugeName}</p>
              {/* 品目名 */}
              <p aria-label="品目">{dp.itemName}</p>
              {/* フロア番号（FloorScreen から活用する）*/}
              <p aria-label="フロア">{dp.floorNumber} 階</p>
              {/* 計量値（連続データの最新値、aria-live でリアルタイム更新を通知する）*/}
              <p aria-label="計量値" aria-live="polite">
                {dp.measuredValue} {dp.unit}
              </p>
              {/* 閾値範囲 */}
              <p aria-label="閾値範囲">
                閾値: {dp.lowerAlarmThreshold}〜{dp.upperAlarmThreshold} {dp.unit}
              </p>
              {/* アラーム状態 */}
              <p role="status" aria-label="アラーム状態">
                {gaugeAlarmLabel(dp.alarmState)}
              </p>
              {/* 測定タイムスタンプ */}
              <p aria-label="測定日時">{dp.measuredAt}</p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
