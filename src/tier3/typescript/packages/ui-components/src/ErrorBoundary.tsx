// k1s0 tier3 SPA 共通 ErrorBoundary コンポーネント
// React 18 Class Component として実装する（ErrorBoundary は class 必須）
// WCAG 2.1 AA: role="alert" でエラーをスクリーンリーダーに即時通知する

// React および必要な型をインポートする
import React from "react";

// ErrorBoundary に渡すプロパティ型
export interface ErrorBoundaryProps {
  // ラップするコンテンツ（React ノード）
  readonly children: React.ReactNode;
  // カスタムフォールバック UI（省略時はデフォルトの error panel を使用する）
  readonly fallback?: React.ReactNode;
}

// ErrorBoundary の内部 state 型
interface ErrorBoundaryState {
  // エラーが発生したかどうかのフラグ
  readonly hasError: boolean;
  // キャッチしたエラーオブジェクト（null は正常状態）
  readonly error: Error | null;
}

// ErrorBoundary コンポーネント本体（React.Component を継承する）
export class ErrorBoundary extends React.Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  // 初期 state: エラーなし
  public override state: ErrorBoundaryState = {
    // エラー未発生状態で初期化する
    hasError: false,
    // エラーオブジェクトは null で初期化する
    error: null,
  };

  // React が提供する静的 getDerivedStateFromError: エラーを state に格納する
  public static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    // エラー発生フラグを立て、エラーオブジェクトを保持する
    return { hasError: true, error };
  }

  // エラーログ出力（必要に応じて外部 error reporter に送信できる）
  public override componentDidCatch(
    error: Error,
    info: React.ErrorInfo,
  ): void {
    // コンポーネントスタックを含むエラー情報をコンソールに出力する
    console.error("[ErrorBoundary] キャッチしたエラー:", error, info.componentStack);
  }

  // render: エラー状態に応じてフォールバック or 通常コンテンツを返す
  public override render(): React.ReactNode {
    // エラーが発生している場合はフォールバック UI を返す
    if (this.state.hasError) {
      // カスタムフォールバックが指定されている場合はそちらを優先する
      if (this.props.fallback !== undefined) {
        // カスタムフォールバックを返す
        return this.props.fallback;
      }
      // デフォルトのエラー表示（role="alert" で即時通知する）
      return (
        // role="alert": スクリーンリーダーに即時エラーを通知する
        <div role="alert" aria-live="assertive" aria-label="アプリケーションエラー">
          {/* エラー見出し */}
          <h2>エラーが発生しました</h2>
          {/* エラーメッセージを表示する */}
          <p aria-label="エラー詳細">
            {this.state.error?.message ?? "不明なエラーが発生しました"}
          </p>
        </div>
      );
    }
    // 正常時は子要素をそのままレンダリングする
    return this.props.children;
  }
}
