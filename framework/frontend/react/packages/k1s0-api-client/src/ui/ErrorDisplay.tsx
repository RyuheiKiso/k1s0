import React, { type ReactNode } from 'react';
import {
  Alert,
  AlertTitle,
  Box,
  Button,
  Collapse,
  IconButton,
  Typography,
} from '@mui/material';
import { ApiError } from '../error/ApiError.js';

interface ErrorDisplayProps {
  /** 表示するエラー */
  error: ApiError | Error | null;
  /** リトライボタンクリック時のコールバック */
  onRetry?: () => void;
  /** 閉じるボタンクリック時のコールバック */
  onDismiss?: () => void;
  /** 詳細情報（trace_id等）を表示するか */
  showDetails?: boolean;
  /** カスタムアクション */
  actions?: ReactNode;
  /** コンパクト表示 */
  compact?: boolean;
}

/**
 * APIエラーを標準UIで表示するコンポーネント
 * - エラー種別に応じた表示
 * - trace_id/error_codeの表示（デバッグ用）
 * - リトライボタン（リトライ可能なエラーの場合）
 */
export function ErrorDisplay({
  error,
  onRetry,
  onDismiss,
  showDetails = false,
  actions,
  compact = false,
}: ErrorDisplayProps) {
  if (!error) {
    return null;
  }

  const apiError = error instanceof ApiError ? error : null;

  // エラー種別に応じたseverity
  const severity = getSeverity(apiError);

  // ユーザー向けメッセージ
  const message = apiError?.userMessage ?? error.message;

  // リトライ可能かどうか
  const canRetry = apiError?.isRetryable ?? false;

  return (
    <Alert
      severity={severity}
      onClose={onDismiss}
      action={
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          {canRetry && onRetry && (
            <Button color="inherit" size="small" onClick={onRetry}>
              再試行
            </Button>
          )}
          {actions}
        </Box>
      }
      sx={{ mb: compact ? 1 : 2 }}
    >
      {!compact && <AlertTitle>{getTitle(apiError)}</AlertTitle>}
      <Typography variant="body2">{message}</Typography>

      {/* フィールドエラー */}
      {apiError?.hasFieldErrors && (
        <Box component="ul" sx={{ mt: 1, mb: 0, pl: 2 }}>
          {apiError.fieldErrors.map((fe, index) => (
            <li key={`${fe.field}-${index}`}>
              <Typography variant="body2" color="text.secondary">
                {fe.field}: {fe.message}
              </Typography>
            </li>
          ))}
        </Box>
      )}

      {/* 詳細情報（開発/デバッグ用） */}
      {showDetails && apiError && (
        <ErrorDetails
          errorCode={apiError.errorCode}
          traceId={apiError.traceId}
          status={apiError.status}
        />
      )}
    </Alert>
  );
}

/**
 * エラー詳細表示コンポーネント
 */
function ErrorDetails({
  errorCode,
  traceId,
  status,
}: {
  errorCode: string;
  traceId?: string;
  status: number;
}) {
  const [expanded, setExpanded] = React.useState(false);

  return (
    <Box sx={{ mt: 1 }}>
      <Button
        size="small"
        onClick={() => setExpanded(!expanded)}
        sx={{ p: 0, minWidth: 'auto', textTransform: 'none' }}
      >
        {expanded ? '詳細を隠す' : '詳細を表示'}
      </Button>
      <Collapse in={expanded}>
        <Box
          sx={{
            mt: 1,
            p: 1,
            bgcolor: 'action.hover',
            borderRadius: 1,
            fontFamily: 'monospace',
            fontSize: '0.75rem',
          }}
        >
          <div>error_code: {errorCode}</div>
          {traceId && <div>trace_id: {traceId}</div>}
          {status > 0 && <div>status: {status}</div>}
        </Box>
      </Collapse>
    </Box>
  );
}

/**
 * エラー種別に応じたseverityを取得
 */
function getSeverity(error: ApiError | null): 'error' | 'warning' | 'info' {
  if (!error) return 'error';

  switch (error.kind) {
    case 'validation':
    case 'conflict':
      return 'warning';
    case 'temporary':
    case 'dependency':
    case 'timeout':
    case 'rate_limit':
      return 'info';
    default:
      return 'error';
  }
}

/**
 * エラー種別に応じたタイトルを取得
 */
function getTitle(error: ApiError | null): string {
  if (!error) return 'エラー';

  switch (error.kind) {
    case 'validation':
      return '入力エラー';
    case 'authentication':
      return '認証エラー';
    case 'authorization':
      return '権限エラー';
    case 'not_found':
      return 'リソースが見つかりません';
    case 'conflict':
      return 'データ競合';
    case 'dependency':
    case 'temporary':
      return 'サービスエラー';
    case 'rate_limit':
      return 'リクエスト制限';
    case 'timeout':
      return 'タイムアウト';
    case 'network':
      return 'ネットワークエラー';
    default:
      return 'エラー';
  }
}

/**
 * インラインエラー表示（フォームフィールド等）
 */
export function InlineError({
  error,
  field,
}: {
  error: ApiError | null;
  field: string;
}) {
  if (!error) return null;

  const fieldError = error.getFieldError(field);
  if (!fieldError) return null;

  return (
    <Typography variant="caption" color="error">
      {fieldError}
    </Typography>
  );
}
