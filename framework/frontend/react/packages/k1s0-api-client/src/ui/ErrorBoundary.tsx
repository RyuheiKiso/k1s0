import React, { Component, type ReactNode, type ErrorInfo } from 'react';
import { Box, Button, Container, Typography } from '@mui/material';
import { ApiError } from '../error/ApiError.js';

interface ErrorBoundaryProps {
  children: ReactNode;
  /** フォールバックUIのカスタマイズ */
  fallback?: ReactNode | ((error: Error, reset: () => void) => ReactNode);
  /** エラー発生時のコールバック */
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  /** リセット時のコールバック */
  onReset?: () => void;
}

interface ErrorBoundaryState {
  error: Error | null;
}

/**
 * APIエラーおよびレンダリングエラーをキャッチするErrorBoundary
 */
export class ApiErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    this.props.onError?.(error, errorInfo);
  }

  reset = (): void => {
    this.setState({ error: null });
    this.props.onReset?.();
  };

  render(): ReactNode {
    const { error } = this.state;
    const { children, fallback } = this.props;

    if (error) {
      // カスタムフォールバック
      if (fallback) {
        if (typeof fallback === 'function') {
          return fallback(error, this.reset);
        }
        return fallback;
      }

      // デフォルトフォールバック
      return <DefaultErrorFallback error={error} onReset={this.reset} />;
    }

    return children;
  }
}

/**
 * デフォルトのエラーフォールバックUI
 */
function DefaultErrorFallback({
  error,
  onReset,
}: {
  error: Error;
  onReset: () => void;
}) {
  const apiError = error instanceof ApiError ? error : null;

  return (
    <Container maxWidth="sm">
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '50vh',
          textAlign: 'center',
          py: 4,
        }}
      >
        <Typography variant="h5" gutterBottom color="error">
          エラーが発生しました
        </Typography>

        <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
          {apiError?.userMessage ?? error.message}
        </Typography>

        {apiError && (
          <Box
            sx={{
              mb: 3,
              p: 2,
              bgcolor: 'grey.100',
              borderRadius: 1,
              fontFamily: 'monospace',
              fontSize: '0.75rem',
            }}
          >
            <div>error_code: {apiError.errorCode}</div>
            {apiError.traceId && <div>trace_id: {apiError.traceId}</div>}
          </Box>
        )}

        <Box sx={{ display: 'flex', gap: 2 }}>
          <Button variant="contained" onClick={onReset}>
            再試行
          </Button>
          <Button
            variant="outlined"
            onClick={() => window.location.reload()}
          >
            ページを再読み込み
          </Button>
        </Box>
      </Box>
    </Container>
  );
}

/**
 * 関数コンポーネント用のErrorBoundaryラッパー
 */
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  errorBoundaryProps?: Omit<ErrorBoundaryProps, 'children'>
): React.FC<P> {
  const Wrapped: React.FC<P> = (props) => (
    <ApiErrorBoundary {...errorBoundaryProps}>
      <Component {...props} />
    </ApiErrorBoundary>
  );

  Wrapped.displayName = `withErrorBoundary(${
    Component.displayName || Component.name || 'Component'
  })`;

  return Wrapped;
}
