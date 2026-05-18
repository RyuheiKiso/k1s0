// k1s0 tier3 SPA ページネーションコンポーネント
// 大量データ一覧のページ切り替え UI を提供する
// WCAG 2.1 AA: aria-label / aria-current / nav ランドマークでアクセシビリティを確保する

// React をインポートする
import React from "react";

// Pagination に渡すプロパティ型
export interface PaginationProps {
  // 現在のページ番号（1 始まり）
  readonly currentPage: number;
  // 総ページ数
  readonly totalPages: number;
  // ページ変更コールバック
  readonly onPageChange: (page: number) => void;
  // ナビゲーションの aria-label（省略時は "ページネーション"）
  readonly navLabel?: string;
}

// Pagination コンポーネント本体
export function Pagination({
  currentPage,
  totalPages,
  onPageChange,
  navLabel = "ページネーション",
}: PaginationProps): React.JSX.Element | null {
  // 総ページ数が 1 以下の場合はページネーションを表示しない
  if (totalPages <= 1) {
    // 表示不要のため null を返す
    return null;
  }

  // 前ページに移動するハンドラ
  function handlePrev(): void {
    // 現在ページが 1 より大きい場合のみ前ページに移動する
    if (currentPage > 1) {
      // 前ページに移動する
      onPageChange(currentPage - 1);
    }
  }

  // 次ページに移動するハンドラ
  function handleNext(): void {
    // 現在ページが最終ページ未満の場合のみ次ページに移動する
    if (currentPage < totalPages) {
      // 次ページに移動する
      onPageChange(currentPage + 1);
    }
  }

  // 表示するページ番号のリストを生成する（前後 2 ページ分を表示する）
  const pageNumbers: number[] = [];
  // 表示開始ページ（1 以上になるよう clamp する）
  const startPage = Math.max(1, currentPage - 2);
  // 表示終了ページ（totalPages 以下になるよう clamp する）
  const endPage = Math.min(totalPages, currentPage + 2);
  // startPage から endPage まで連番を生成する
  for (let p = startPage; p <= endPage; p++) {
    // ページ番号を追加する
    pageNumbers.push(p);
  }

  // ページネーションを描画する
  return (
    // nav ランドマーク: スクリーンリーダーがナビゲーション領域として認識する
    <nav aria-label={navLabel}>
      {/* ページネーションリスト */}
      <ul
        // role="list": 明示的なリスト
        role="list"
        // aria-label: リスト全体のラベル
        aria-label="ページ一覧"
      >
        {/* 前ページボタン */}
        <li role="listitem">
          <button
            // 前ページへ移動する
            onClick={handlePrev}
            // 最初のページでは無効にする
            disabled={currentPage === 1}
            // スクリーンリーダー向けラベル
            aria-label="前のページ"
            // type="button" で form submit を防ぐ
            type="button"
          >
            {/* 前ページ記号 */}
            &laquo; 前へ
          </button>
        </li>
        {/* ページ番号ボタン一覧 */}
        {pageNumbers.map((page) => (
          // 各ページ番号をリストアイテムとして表示する
          <li key={page} role="listitem">
            <button
              // クリックで該当ページに移動する
              onClick={() => { onPageChange(page); }}
              // 現在ページは aria-current で示す
              aria-current={page === currentPage ? "page" : undefined}
              // スクリーンリーダー向けラベル
              aria-label={`${page} ページ${page === currentPage ? "（現在のページ）" : ""}`}
              // type="button" で form submit を防ぐ
              type="button"
            >
              {/* ページ番号テキスト */}
              {page}
            </button>
          </li>
        ))}
        {/* 次ページボタン */}
        <li role="listitem">
          <button
            // 次ページへ移動する
            onClick={handleNext}
            // 最後のページでは無効にする
            disabled={currentPage === totalPages}
            // スクリーンリーダー向けラベル
            aria-label="次のページ"
            // type="button" で form submit を防ぐ
            type="button"
          >
            {/* 次ページ記号 */}
            次へ &raquo;
          </button>
        </li>
      </ul>
      {/* 現在ページ位置を aria-live で通知する */}
      <p
        // aria-live="polite": ページ変更を会話の区切りで読み上げる
        aria-live="polite"
        // aria-atomic: テキスト全体を一括読み上げする
        aria-atomic="true"
        // aria-label: ページ位置情報であることをラベル付けする
        aria-label="現在のページ位置"
      >
        {/* ページ位置テキスト */}
        {currentPage} / {totalPages} ページ
      </p>
    </nav>
  );
}
