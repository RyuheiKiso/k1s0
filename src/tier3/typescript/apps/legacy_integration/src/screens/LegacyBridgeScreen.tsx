// k1s0 Legacy 連携 tier3 — ERP 並行運用 entry point 画面
// 製造業 pack 適用例 レガシー資産統合シナリオ: ERP 並行運用
// レガシー .NET Framework 4.8 ERP との並行運用を管理する業務 UI
// HTTP/1.1 + SSE（v1_legacy_http11 専用 listener、別ポート 8443-legacy）を使用する
// per-tab 6 subscription 縮退モードで動作する
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// レガシー ERP 接続状態の型（tier2 生成 stub の型定義に合わせる）
export interface LegacyErpConnectionStatus {
  // 接続 ID（UUID）
  readonly connectionId: string;
  // ERP システム名（表示用）
  readonly erpSystemName: string;
  // 接続状態（connected / disconnected / error / degraded）
  readonly connectionState: "connected" | "disconnected" | "error" | "degraded";
  // HTTP/1.1 + SSE ポート（legacy listener）
  readonly legacyListenerPort: number;
  // アクティブ subscription 数（per-tab 6 縮退モード）
  readonly activeSubscriptionCount: number;
  // 最大 subscription 数（縮退上限）
  readonly maxSubscriptionCount: number;
  // 最終ハートビート日時（ISO 8601 文字列）
  readonly lastHeartbeatAt: string;
}

// レガシー ERP 業務データサマリの型（tier2 生成 stub の型定義に合わせる）
export interface LegacyErpDataSummary {
  // サマリ ID（UUID）
  readonly summaryId: string;
  // データ種別（master / transaction / event）
  readonly dataType: "master" | "transaction" | "event";
  // エンティティ名（ERP 側のエンティティ名）
  readonly entityName: string;
  // 最終同期日時（ISO 8601 文字列）
  readonly lastSyncAt: string;
  // 同期レコード数
  readonly syncedRecordCount: number;
  // 同期状態（synced / pending / error）
  readonly syncStatus: "synced" | "pending" | "error";
}

// LegacyBridgeScreen のプロパティ型
export interface LegacyBridgeScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // ERP 接続状態（undefined の場合はローディング中）
  readonly connectionStatus?: LegacyErpConnectionStatus;
  // ERP 業務データサマリ一覧（undefined の場合はローディング中）
  readonly dataSummaries?: readonly LegacyErpDataSummary[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 接続状態の日本語ラベルを返す
function connectionStateLabel(state: LegacyErpConnectionStatus["connectionState"]): string {
  // 接続状態に応じたラベルを返す
  switch (state) {
    case "connected":
      // 接続中
      return "接続中";
    case "disconnected":
      // 切断中
      return "切断中";
    case "error":
      // 接続エラー
      return "接続エラー";
    case "degraded":
      // 縮退モード
      return "縮退モード";
    default: {
      // 網羅性チェック（新状態追加時はコンパイルエラーで検知する）
      const _exhaustive: never = state;
      return String(_exhaustive);
    }
  }
}

// データ種別の日本語ラベルを返す
function dataTypeLabel(type: LegacyErpDataSummary["dataType"]): string {
  // データ種別に応じたラベルを返す
  switch (type) {
    case "master":
      // マスタデータ
      return "マスタデータ";
    case "transaction":
      // トランザクションデータ
      return "トランザクションデータ";
    case "event":
      // イベントデータ
      return "イベントデータ";
    default: {
      // 網羅性チェック（新種別追加時はコンパイルエラーで検知する）
      const _exhaustive: never = type;
      return String(_exhaustive);
    }
  }
}

// 同期状態の日本語ラベルを返す
function syncStatusLabel(status: LegacyErpDataSummary["syncStatus"]): string {
  // 同期状態に応じたラベルを返す
  switch (status) {
    case "synced":
      // 同期済み
      return "同期済み";
    case "pending":
      // 同期待ち
      return "同期待ち";
    case "error":
      // 同期エラー
      return "同期エラー";
    default: {
      // 網羅性チェック（新状態追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// ERP 並行運用 entry point 画面コンポーネント
export function LegacyBridgeScreen({
  connectionStatus,
  dataSummaries,
  isLoading = false,
  errorMessage,
}: LegacyBridgeScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="ERP 並行運用">
        {/* ページ見出し */}
        <h1>ERP 並行運用（レガシー連携）</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="ERP 接続状態を確認中">
          <p>ERP 接続状態を確認中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="ERP 並行運用">
        {/* ページ見出し */}
        <h1>ERP 並行運用（レガシー連携）</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // ERP 並行運用 entry point を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="ERP 並行運用">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>ERP 並行運用（レガシー連携）</h1>
      {/* ERP 接続状態パネル */}
      {connectionStatus !== undefined && (
        // ERP 接続状態を section として表示する
        <section aria-label="ERP 接続状態">
          {/* 接続状態見出し */}
          <h2>ERP 接続状態</h2>
          {/* ERP システム名 */}
          <p aria-label="ERP システム">{connectionStatus.erpSystemName}</p>
          {/* 接続状態（aria-live でリアルタイム更新を通知する）*/}
          <p role="status" aria-label="接続状態" aria-live="polite">
            {connectionStateLabel(connectionStatus.connectionState)}
          </p>
          {/* HTTP/1.1 + SSE レガシーポート */}
          <p aria-label="レガシー listener ポート">
            ポート: {connectionStatus.legacyListenerPort}（HTTP/1.1 + SSE）
          </p>
          {/* アクティブ subscription 数（per-tab 6 縮退モード）*/}
          <p aria-label="アクティブ subscription 数">
            subscription: {connectionStatus.activeSubscriptionCount} /
            {connectionStatus.maxSubscriptionCount}（縮退上限）
          </p>
          {/* 最終ハートビート日時 */}
          <p aria-label="最終ハートビート">{connectionStatus.lastHeartbeatAt}</p>
          {/* 縮退モード警告バナー（接続状態が degraded の場合に表示する）*/}
          {connectionStatus.connectionState === "degraded" && (
            // role="alert" で即時読み上げを行う
            <div role="alert" aria-label="縮退モード警告" aria-live="assertive">
              <p>
                縮退モードで動作中です。per-tab {connectionStatus.maxSubscriptionCount} subscription
                上限で動作しています。
              </p>
            </div>
          )}
        </section>
      )}
      {/* ERP 業務データサマリ一覧 */}
      <section aria-label="ERP データ同期状態">
        {/* サマリ見出し */}
        <h2>ERP データ同期状態</h2>
        {/* データサマリ件数 */}
        <p aria-live="polite" aria-label="同期エンティティ件数">
          {dataSummaries !== undefined ? `${dataSummaries.length} エンティティ` : ""}
        </p>
        {/* データサマリ一覧または空状態 */}
        {dataSummaries === undefined || dataSummaries.length === 0 ? (
          // 空状態を表示する
          <div role="status" aria-label="同期データがありません">
            <p>同期データがありません。</p>
          </div>
        ) : (
          // データサマリリストを表示する
          <ul aria-label="ERP データサマリリスト">
            {dataSummaries.map((summary) => (
              // データサマリアイテムを key 付きで展開する
              <li
                key={summary.summaryId}
                // aria-label でエンティティ名と同期状態を説明する
                aria-label={`${summary.entityName}（${syncStatusLabel(summary.syncStatus)}）`}
              >
                {/* エンティティ名（見出し h3）*/}
                <h3>{summary.entityName}</h3>
                {/* データ種別 */}
                <p aria-label="データ種別">{dataTypeLabel(summary.dataType)}</p>
                {/* 最終同期日時 */}
                <p aria-label="最終同期日時">{summary.lastSyncAt}</p>
                {/* 同期レコード数 */}
                <p aria-label="同期レコード数">{summary.syncedRecordCount} レコード</p>
                {/* 同期状態 */}
                <p role="status" aria-label="同期状態">
                  {syncStatusLabel(summary.syncStatus)}
                </p>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
