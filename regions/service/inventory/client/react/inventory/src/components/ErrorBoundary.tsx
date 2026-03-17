import { Component, type ReactNode } from 'react';

// ErrorBoundaryのProps定義
interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

// ErrorBoundaryの内部状態
interface State {
  hasError: boolean;
  error?: Error;
}

// エラーバウンダリコンポーネント: 子コンポーネントのレンダリングエラーをキャッチして表示
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
    // エラー発生時はフォールバックUIを表示
    if (this.state.hasError) {
      return this.props.fallback ?? (
        <div role="alert">
          <h2>エラーが発生しました</h2>
          <p>{this.state.error?.message}</p>
        </div>
      );
    }
    return this.props.children;
  }
}
