// k1s0 tier3 SPA 状態遷移インジケーターコンポーネント
// 業務エンティティの状態機械（FSM）遷移を視覚的に示す
// WCAG 2.1 AA: aria-label / role="img" でスクリーンリーダーに状態を伝える

// React をインポートする
import React from "react";

// 状態遷移の単一ステップ型
export interface TransitionStep {
  // ステップのユニーク ID
  readonly id: string;
  // 表示ラベル
  readonly label: string;
  // このステップが完了済みかどうか
  readonly completed: boolean;
  // このステップが現在アクティブかどうか
  readonly active: boolean;
}

// StateTransitionIndicator に渡すプロパティ型
export interface StateTransitionIndicatorProps {
  // 遷移ステップ一覧（順序付き）
  readonly steps: readonly TransitionStep[];
  // インジケーターの aria-label（省略時は "状態遷移"）
  readonly label?: string;
}

// StateTransitionIndicator コンポーネント本体
export function StateTransitionIndicator({
  steps,
  label = "状態遷移",
}: StateTransitionIndicatorProps): React.JSX.Element {
  // ステップ一覧が空の場合は何も表示しない（空の ol を返す）
  return (
    // role="list" として ordered list で状態遷移を表現する
    <nav aria-label={label}>
      {/* ol: 順序付きリストで遷移ステップを表現する（順序が意味を持つため ol を使う）*/}
      <ol
        // aria-label: リスト全体のラベル
        aria-label={`${label} ステップ一覧`}
        // role="list": 明示的なリスト
        role="list"
      >
        {steps.map((step, index) => {
          // ステップの状態を文字列で表現する（aria-label に使用する）
          const stepStatus = step.active
            ? "現在のステップ"
            : step.completed
              ? "完了済み"
              : "未完了";
          // ステップ番号（1 始まり）
          const stepNumber = index + 1;
          // 各ステップをリストアイテムとして表示する
          return (
            <li
              // ユニーク key
              key={step.id}
              // role="listitem": 明示的なリストアイテム
              role="listitem"
              // aria-label: ステップの状態をスクリーンリーダーに伝える
              aria-label={`ステップ ${stepNumber}: ${step.label}（${stepStatus}）`}
              // aria-current: 現在のステップを示す
              aria-current={step.active ? "step" : undefined}
            >
              {/* ステップ番号 */}
              <span aria-hidden="true">{stepNumber}</span>
              {/* ステップラベル */}
              <span>{step.label}</span>
              {/* 完了済みアイコン（視覚補助、スクリーンリーダーは読まない）*/}
              {step.completed && (
                // 完了済みチェックマーク（aria-hidden で読み上げをスキップする）
                <span aria-hidden="true"> ✓</span>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
