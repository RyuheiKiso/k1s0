// k1s0 tier3 SPA オプティミスティック更新バッジコンポーネント
// クライアント側で先行反映した状態（送信中 / 確定済 / 失敗）を視覚的に示す
// WCAG 2.1 AA: role="status" / aria-label で状態をスクリーンリーダーに通知する

// React をインポートする
import React from "react";

// オプティミスティック更新の状態型
export type OptimisticStatus = "pending" | "confirmed" | "failed";

// OptimisticBadge に渡すプロパティ型
export interface OptimisticBadgeProps {
  // 表示する状態（pending / confirmed / failed）
  readonly status: OptimisticStatus;
  // 補足ラベル（省略時は状態名をそのまま使う）
  readonly label?: string;
}

// 状態に応じた表示テキストを返す
function statusLabel(status: OptimisticStatus): string {
  // 各状態に対応する日本語ラベルを返す
  switch (status) {
    case "pending":
      // 送信中状態のラベル
      return "送信中";
    case "confirmed":
      // 確定済み状態のラベル
      return "確定済み";
    case "failed":
      // 失敗状態のラベル
      return "送信失敗";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラー）
      const _exhaustive: never = status;
      throw new Error(`Unknown status: ${String(_exhaustive)}`);
    }
  }
}

// OptimisticBadge コンポーネント本体
export function OptimisticBadge({
  status,
  label,
}: OptimisticBadgeProps): React.JSX.Element {
  // 表示テキストを決定する
  const displayText = label ?? statusLabel(status);

  // aria-live の値を状態に応じて決定する
  // pending / failed は即時通知、confirmed は後回しで通知する
  const ariaLive: "assertive" | "polite" =
    status === "failed" ? "assertive" : "polite";

  // バッジを描画する
  return (
    // role="status" で動的な状態をスクリーンリーダーに通知する
    <span
      // role="status": 動的変化をスクリーンリーダーに通知する
      role="status"
      // aria-live: 重大度に応じた通知タイミングを設定する
      aria-live={ariaLive}
      // aria-label: バッジの意味をスクリーンリーダーに伝える
      aria-label={`オプティミスティック状態: ${displayText}`}
      // data-status: CSS セレクタで色を変えるためのデータ属性
      data-status={status}
    >
      {/* 状態ラベルテキスト */}
      {displayText}
    </span>
  );
}
