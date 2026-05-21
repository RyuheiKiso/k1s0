// k1s0 検査 tier3 — 図面 collaborative review 画面
// 製造業 pack 適用例 scenario 11: 図面 collaborative review（v1_interactive、双方向 presence）
// 複数ユーザが同時に図面を参照し、コメントを追加しながら検査レビューを行う業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 図面レビューセッションエンティティの型（tier2 生成 stub の型定義に合わせる）
export interface DrawingReviewSession {
  // セッション ID（UUID）
  readonly sessionId: string;
  // 図面番号（業務キー）
  readonly drawingNumber: string;
  // 図面タイトル（表示用）
  readonly drawingTitle: string;
  // 図面バージョン
  readonly drawingVersion: string;
  // レビューステータス（in_review / approved / rejected / pending）
  readonly reviewStatus: "in_review" | "approved" | "rejected" | "pending";
  // 参加者数（現在アクティブな presence 数）
  readonly activeParticipantCount: number;
  // コメント数
  readonly commentCount: number;
  // 最終更新日時（ISO 8601 文字列）
  readonly lastUpdatedAt: string;
}

// DrawingReviewScreen のプロパティ型
export interface DrawingReviewScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 図面レビューセッション一覧（undefined の場合はローディング中）
  readonly sessions?: readonly DrawingReviewSession[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// レビューステータスの日本語ラベルを返す
function reviewStatusLabel(status: DrawingReviewSession["reviewStatus"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "in_review":
      // レビュー中
      return "レビュー中";
    case "approved":
      // 承認済み
      return "承認済み";
    case "rejected":
      // 否認
      return "否認";
    case "pending":
      // レビュー待ち
      return "レビュー待ち";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 図面 collaborative review 画面コンポーネント
export function DrawingReviewScreen({
  sessions,
  isLoading = false,
  errorMessage,
}: DrawingReviewScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="図面レビュー">
        {/* ページ見出し */}
        <h1>図面 Collaborative Review</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="図面レビューセッションを読み込み中">
          <p>図面レビューセッションを読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="図面レビュー">
        {/* ページ見出し */}
        <h1>図面 Collaborative Review</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 図面レビューセッション一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="図面レビュー">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>図面 Collaborative Review</h1>
      {/* セッション件数サマリ */}
      <p aria-live="polite" aria-label="レビューセッション件数">
        {sessions !== undefined ? `${sessions.length} 件のレビューセッション` : ""}
      </p>
      {/* セッション一覧または空状態 */}
      {sessions === undefined || sessions.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="レビューセッションがありません">
          <p>図面レビューセッションがありません。</p>
        </div>
      ) : (
        // セッションリストを表示する
        <ul aria-label="図面レビューセッションリスト">
          {sessions.map((session) => (
            // セッションアイテムを key 付きで展開する
            <li
              key={session.sessionId}
              // aria-label で図面番号とレビューステータスを説明する
              aria-label={`図面 ${session.drawingNumber}（${reviewStatusLabel(session.reviewStatus)}）`}
            >
              {/* 図面番号（見出し h3）*/}
              <h3>{session.drawingNumber}</h3>
              {/* 図面タイトル */}
              <p aria-label="図面タイトル">{session.drawingTitle}</p>
              {/* 図面バージョン */}
              <p aria-label="バージョン">{session.drawingVersion}</p>
              {/* レビューステータス */}
              <p role="status" aria-label="レビューステータス">
                {reviewStatusLabel(session.reviewStatus)}
              </p>
              {/* アクティブ参加者数（presence インジケーター）*/}
              <p aria-label="参加者数" aria-live="polite">
                {session.activeParticipantCount} 名が参加中
              </p>
              {/* コメント数 */}
              <p aria-label="コメント数">{session.commentCount} 件のコメント</p>
              {/* 最終更新日時 */}
              <p aria-label="最終更新">{session.lastUpdatedAt}</p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
