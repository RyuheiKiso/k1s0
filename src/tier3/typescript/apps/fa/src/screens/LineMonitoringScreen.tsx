// k1s0 FA tier3 — ライン稼働監視 live tile 画面
// 製造業 pack 適用例 scenario 2: ライン稼働監視 live tile（v1_live_snapshot、latest-wins）
// 既存 spa/src/screens/plant/PlantScreen.tsx の plant 一覧ロジックを継承・活用する
// 製造ラインの稼働状況をリアルタイムのタイル形式で表示する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient, PlantSummary } from "@k1s0/tier3-pack";

// ラインスナップショットエンティティの型（tier2 生成 stub の型定義に合わせる）
export interface LineSnapshot {
  // ライン ID（UUID）
  readonly lineId: string;
  // ライン名（表示用）
  readonly lineName: string;
  // 所属工場 ID（PlantSummary との紐付け用）
  readonly plantId: string;
  // 稼働状態（running / stopped / error / changeover）
  readonly status: "running" | "stopped" | "error" | "changeover";
  // 稼働率（0〜100 のパーセント）
  readonly utilizationRate: number;
  // 直近 1 時間の生産数
  readonly outputLastHour: number;
  // 最終更新日時（ISO 8601 文字列）
  readonly lastUpdatedAt: string;
  // アラート件数（未対応の設備アラート数）
  readonly activeAlertCount: number;
}

// LineMonitoringScreen のプロパティ型
export interface LineMonitoringScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // ラインスナップショット一覧（undefined の場合はローディング中）
  readonly lines?: readonly LineSnapshot[];
  // 工場サマリ一覧（PlantScreen から活用する）
  readonly plants?: readonly PlantSummary[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// ライン稼働状態の日本語ラベルを返す
function lineStatusLabel(status: LineSnapshot["status"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "running":
      // 稼働中
      return "稼働中";
    case "stopped":
      // 停止中
      return "停止中";
    case "error":
      // 異常
      return "異常";
    case "changeover":
      // 段取り替え中
      return "段取り替え中";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// ライン live tile コンポーネント（タイル形式で 1 ラインを表示する）
function LineTile({ line }: { readonly line: LineSnapshot }): React.JSX.Element {
  // ライン稼働 live tile を article 要素として返す
  return (
    // article: 独立したコンテンツ（ライン単位のタイル）
    <article
      // aria-label でライン名と稼働状態を説明する
      aria-label={`ライン ${line.lineName}（${lineStatusLabel(line.status)}）`}
    >
      {/* ライン名（見出し h3）*/}
      <h3>{line.lineName}</h3>
      {/* 稼働状態（latest-wins のリアルタイム値）*/}
      <p role="status" aria-label="稼働状態" aria-live="polite">
        {lineStatusLabel(line.status)}
      </p>
      {/* 稼働率 */}
      <p aria-label="稼働率" aria-live="polite">
        {line.utilizationRate.toFixed(1)}%
      </p>
      {/* 直近 1 時間の生産数 */}
      <p aria-label="直近 1 時間生産数" aria-live="polite">
        {line.outputLastHour} 個 / 時
      </p>
      {/* アラート件数（0 以上の場合は警告表示する）*/}
      {line.activeAlertCount > 0 && (
        // アラートを role="alert" で即時通知する
        <p role="alert" aria-label={`アラート ${line.activeAlertCount} 件`}>
          {line.activeAlertCount} 件のアラート
        </p>
      )}
      {/* 最終更新日時 */}
      <p aria-label="最終更新">{line.lastUpdatedAt}</p>
    </article>
  );
}

// ライン稼働監視 live tile 画面コンポーネント
export function LineMonitoringScreen({
  lines,
  isLoading = false,
  errorMessage,
}: LineMonitoringScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="ライン稼働監視">
        {/* ページ見出し */}
        <h1>ライン稼働監視</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="ライン情報を読み込み中">
          <p>ライン情報を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="ライン稼働監視">
        {/* ページ見出し */}
        <h1>ライン稼働監視</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 異常ラインの件数をカウントする（警報表示用）
  const errorLineCount = lines?.filter((l) => l.status === "error").length ?? 0;

  // ライン稼働監視 live tile 一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="ライン稼働監視">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>ライン稼働監視</h1>
      {/* 異常ライン警報バナー（error が存在する場合のみ表示する）*/}
      {errorLineCount > 0 && (
        // role="alert" で即時読み上げを行う
        <div role="alert" aria-label={`異常ライン ${errorLineCount} 件`} aria-live="assertive">
          <p>異常ラインが {errorLineCount} 件あります。</p>
        </div>
      )}
      {/* ライン件数サマリ（aria-live でリアルタイム更新を通知する）*/}
      <p aria-live="polite" aria-label="ライン件数">
        {lines !== undefined ? `${lines.length} ライン` : ""}
      </p>
      {/* ライン live tile 一覧または空状態 */}
      {lines === undefined || lines.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="ラインがありません">
          <p>ラインがありません。</p>
        </div>
      ) : (
        // ライン live tile をグリッド表示する
        <div
          // role="list" で live tile の集合を示す
          role="list"
          // aria-label でリストの目的を説明する
          aria-label="ライン live tile リスト"
        >
          {lines.map((line) => (
            // ライン live tile を role="listitem" で展開する
            <div key={line.lineId} role="listitem">
              {/* ライン live tile コンポーネントを表示する */}
              <LineTile line={line} />
            </div>
          ))}
        </div>
      )}
    </main>
  );
}
