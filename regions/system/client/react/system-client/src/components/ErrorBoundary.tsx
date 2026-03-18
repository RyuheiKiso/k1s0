import { Component, type ReactNode } from 'react';

// ErrorBoundary の Props 定義
interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

// ErrorBoundary の内部状態
interface State {
  hasError: boolean;
  error?: Error;
}

// エラーバウンダリコンポーネント: 子コンポーネントのレンダリングエラーをキャッチして表示
// 本番環境ではエラー詳細を隠蔽し、ユーザーフレンドリーなメッセージのみ表示する
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  // レンダリングエラー発生時にエラー状態を更新
  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  override render() {
    // エラー発生時はフォールバック UI を表示
    if (this.state.hasError) {
      // カスタムフォールバックが指定されている場合はそちらを使用
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // 本番環境ではエラー詳細（スタックトレース等）を隠蔽してセキュリティを確保する
      const isDev = import.meta.env?.DEV ?? false;

      return (
        <div role="alert">
          <h2>エラーが発生しました</h2>
          {isDev ? (
            <details>
              <summary>{this.state.error?.message}</summary>
              <pre style={{ whiteSpace: 'pre-wrap', fontSize: '0.85em' }}>
                {this.state.error?.stack}
              </pre>
            </details>
          ) : (
            <p>
              予期しないエラーが発生しました。ページを再読み込みするか、管理者にお問い合わせください。
            </p>
          )}
        </div>
      );
    }
    return this.props.children;
  }
}
