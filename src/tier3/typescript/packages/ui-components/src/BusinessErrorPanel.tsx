// k1s0 tier3 SPA 業務エラーパネルコンポーネント
// 業務ロジックエラー（バリデーション / 権限不足 / ルール違反）を表示する
// WCAG 2.1 AA: role="alert" / aria-describedby で構造化してスクリーンリーダー対応する

// React をインポートする
import React from "react";

// 業務エラーの重大度
export type BusinessErrorSeverity = "error" | "warning" | "info";

// 業務エラーの型
export interface BusinessError {
  // エラーコード（一意識別子）
  readonly code: string;
  // ユーザー向けメッセージ
  readonly message: string;
  // 重大度（error / warning / info）
  readonly severity: BusinessErrorSeverity;
  // フィールド名（フォームエラーの場合に使用する）
  readonly field?: string;
}

// BusinessErrorPanel に渡すプロパティ型
export interface BusinessErrorPanelProps {
  // 表示するエラー一覧
  readonly errors: readonly BusinessError[];
  // パネルの見出し（省略時は "エラー"）
  readonly title?: string;
  // パネルを非表示にするコールバック（省略時はクローズボタンを非表示にする）
  readonly onDismiss?: () => void;
}

// 重大度に応じた aria-live の設定を返す
function getAriaLive(
  severity: BusinessErrorSeverity,
): "assertive" | "polite" {
  // error / warning は即時通知、info は後回しで通知する
  return severity === "info" ? "polite" : "assertive";
}

// BusinessErrorPanel コンポーネント本体
export function BusinessErrorPanel({
  errors,
  title = "エラー",
  onDismiss,
}: BusinessErrorPanelProps): React.JSX.Element | null {
  // エラーが空の場合は何も表示しない
  if (errors.length === 0) {
    // 空のフラグメントを返す（DOM に何も追加しない）
    return null;
  }

  // エラー一覧の最も高い重大度を判定する（aria-live 値を決定するため）
  const maxSeverity: BusinessErrorSeverity = errors.some(
    (e) => e.severity === "error",
  )
    ? "error"
    : errors.some((e) => e.severity === "warning")
      ? "warning"
      : "info";

  // aria-live 値を決定する
  const ariaLive = getAriaLive(maxSeverity);

  // パネルのユニーク ID を生成する（aria-describedby に使用する）
  const panelId = "business-error-panel";

  // パネルを描画する
  return (
    // role="alert" でスクリーンリーダーにエラーを通知する
    <section
      // ランドマーク: region にしてスクリーンリーダーのナビゲーションを支援する
      role="alert"
      // aria-live: 重大度に応じた通知タイミングを設定する
      aria-live={ariaLive}
      // aria-label: パネル全体のラベル
      aria-label={title}
      // id: 他要素から aria-describedby で参照できるようにする
      id={panelId}
    >
      {/* パネル見出し */}
      <h2 id={`${panelId}-title`}>{title}</h2>
      {/* クローズボタン（onDismiss が指定されている場合のみ表示する）*/}
      {onDismiss !== undefined && (
        // クローズボタン（role="button" は button 要素で自動付与される）
        <button
          // スクリーンリーダー向けラベル
          aria-label="エラーパネルを閉じる"
          // クリックでパネルを非表示にする
          onClick={onDismiss}
          // type="button" で form submit を防ぐ
          type="button"
        >
          {/* ×記号 */}
          ×
        </button>
      )}
      {/* エラー一覧を ul で表示する */}
      <ul aria-label="エラー一覧" role="list">
        {errors.map((error) => (
          // エラーアイテム（code を key に使用する）
          <li
            // code + field でユニーク key を生成する
            key={`${error.code}-${error.field ?? "global"}`}
            // aria-label: フィールド名とメッセージを組み合わせてラベル付けする
            aria-label={
              error.field !== undefined
                ? `${error.field}: ${error.message}`
                : error.message
            }
          >
            {/* フィールド名がある場合は表示する */}
            {error.field !== undefined && (
              // フィールド名を強調表示する
              <strong aria-label="対象フィールド">{error.field}: </strong>
            )}
            {/* エラーメッセージ本文 */}
            <span>{error.message}</span>
            {/* エラーコードを補足として表示する */}
            <small aria-label="エラーコード">（{error.code}）</small>
          </li>
        ))}
      </ul>
    </section>
  );
}
