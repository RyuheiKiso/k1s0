// k1s0 tier3 SPA ローディングスピナーコンポーネント
// 非同期処理中のローディング状態をユーザーに通知する
// WCAG 2.1 AA: role="status" / aria-label でスクリーンリーダーに状態を伝える

// React をインポートする
import React from "react";

// LoadingSpinner に渡すプロパティ型
export interface LoadingSpinnerProps {
  // スクリーンリーダー向けラベル（省略時は "読み込み中"）
  readonly label?: string;
  // スピナーのサイズ（px 単位、省略時は 24）
  readonly size?: number;
}

// LoadingSpinner コンポーネント本体
export function LoadingSpinner({
  label = "読み込み中",
  size = 24,
}: LoadingSpinnerProps): React.JSX.Element {
  // スピナーの SVG サイズを決定する
  const svgSize = size;

  // スピナーを描画する
  return (
    // role="status": スクリーンリーダーに状態変化を通知する（assertive ではなく polite）
    <span
      // role="status" で動的な状態変化を通知する
      role="status"
      // aria-label: スクリーンリーダー向けに読み込み状態を説明する
      aria-label={label}
      // aria-live="polite": 会話の区切りで読み上げる（assertive は使わない）
      aria-live="polite"
    >
      {/* SVG ベースのアニメーションスピナー（CSS アニメーション非依存）*/}
      <svg
        // 幅はプロパティで設定する
        width={svgSize}
        // 高さはプロパティで設定する
        height={svgSize}
        // 円形スピナーの viewBox
        viewBox="0 0 24 24"
        // スピナーの色（currentColor で親要素の text color を継承する）
        fill="none"
        // スクリーンリーダーには aria-label で情報を提供するため svg は装飾として扱う
        aria-hidden="true"
        // フォーカス不可にする（装飾 SVG のため）
        focusable="false"
      >
        {/* 背景の薄い円 */}
        <circle
          // 中心 X 座標
          cx="12"
          // 中心 Y 座標
          cy="12"
          // 半径
          r="10"
          // 背景円のストローク（薄い色で表示する）
          stroke="currentColor"
          // ストローク幅
          strokeWidth="3"
          // 透明度: 背景円は薄く表示する
          opacity="0.25"
        />
        {/* アニメーション用の弧（animateTransform で回転させる）*/}
        <path
          // アニメーション用の弧パスデータ（1/4 円弧）
          d="M12 2a10 10 0 0 1 10 10"
          // ストローク色
          stroke="currentColor"
          // ストローク幅
          strokeWidth="3"
          // 先端を丸くする
          strokeLinecap="round"
        >
          {/* SVG アニメーション: 360 度回転を繰り返す */}
          <animateTransform
            // 変換の属性名（回転に使用する）
            attributeName="transform"
            // 回転変換を指定する
            type="rotate"
            // 開始角度 → 終了角度 + 回転中心
            from="0 12 12"
            // 1 周回転させる
            to="360 12 12"
            // 1 周にかかる時間（1 秒）
            dur="1s"
            // 無限ループ
            repeatCount="indefinite"
          />
        </path>
      </svg>
      {/* 視覚的に非表示だがスクリーンリーダーには読まれるテキスト */}
      <span
        // ビジュアル非表示（WCAG 準拠の sr-only パターン）
        style={{
          // 絶対配置で通常フローから切り離す
          position: "absolute",
          // 幅 1px にする
          width: "1px",
          // 高さ 1px にする
          height: "1px",
          // 溢れたコンテンツを非表示にする
          overflow: "hidden",
          // クリップ領域を 1px の矩形に制限する
          clip: "rect(0,0,0,0)",
          // ホワイトスペース折り返しを禁止する
          whiteSpace: "nowrap",
        }}
      >
        {/* スクリーンリーダー向けテキスト */}
        {label}
      </span>
    </span>
  );
}
