import React, { type ReactNode } from 'react';
import { Box, Typography, Button, type SxProps, type Theme } from '@mui/material';

/**
 * EmptyState のプロパティ
 */
export interface EmptyStateProps {
  /** アイコン */
  icon?: ReactNode;
  /** タイトル */
  title: string;
  /** 説明文 */
  description?: string;
  /** アクションボタンのラベル */
  actionLabel?: string;
  /** アクションボタンクリック時のコールバック */
  onAction?: () => void;
  /** 追加のアクション */
  children?: ReactNode;
  /** スタイル */
  sx?: SxProps<Theme>;
}

/**
 * 空状態コンポーネント
 *
 * データがない場合やエラー時などに表示する空状態のUI。
 *
 * @example
 * ```tsx
 * // 基本的な使用
 * <EmptyState
 *   title="データがありません"
 *   description="まだデータが登録されていません。"
 * />
 *
 * // アクション付き
 * <EmptyState
 *   icon={<AddIcon sx={{ fontSize: 48 }} />}
 *   title="アイテムがありません"
 *   description="新しいアイテムを追加してください。"
 *   actionLabel="アイテムを追加"
 *   onAction={() => navigate('/items/new')}
 * />
 *
 * // カスタムアクション
 * <EmptyState
 *   title="検索結果がありません"
 *   description="条件を変更して再度検索してください。"
 * >
 *   <Button variant="outlined" onClick={resetFilters}>
 *     フィルタをリセット
 *   </Button>
 * </EmptyState>
 * ```
 */
export function EmptyState({
  icon,
  title,
  description,
  actionLabel,
  onAction,
  children,
  sx,
}: EmptyStateProps) {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        textAlign: 'center',
        py: 6,
        px: 2,
        ...sx,
      }}
    >
      {icon && (
        <Box
          sx={{
            color: 'text.secondary',
            mb: 2,
          }}
        >
          {icon}
        </Box>
      )}

      <Typography variant="h6" gutterBottom>
        {title}
      </Typography>

      {description && (
        <Typography
          variant="body2"
          color="text.secondary"
          sx={{ maxWidth: 400, mb: actionLabel || children ? 3 : 0 }}
        >
          {description}
        </Typography>
      )}

      {actionLabel && onAction && (
        <Button variant="contained" onClick={onAction}>
          {actionLabel}
        </Button>
      )}

      {children}
    </Box>
  );
}

/**
 * NoData のプロパティ
 */
export interface NoDataProps {
  /** メッセージ */
  message?: string;
}

/**
 * データなし表示
 *
 * シンプルなデータなし表示。
 *
 * @example
 * ```tsx
 * {items.length === 0 && <NoData />}
 * ```
 */
export function NoData({ message = 'データがありません' }: NoDataProps) {
  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        py: 4,
        color: 'text.secondary',
      }}
    >
      <Typography variant="body2">{message}</Typography>
    </Box>
  );
}

/**
 * NoSearchResults のプロパティ
 */
export interface NoSearchResultsProps {
  /** 検索クエリ */
  query?: string;
  /** リセットのコールバック */
  onReset?: () => void;
}

/**
 * 検索結果なし表示
 *
 * 検索結果が0件の場合に表示。
 *
 * @example
 * ```tsx
 * {searchResults.length === 0 && (
 *   <NoSearchResults
 *     query={searchQuery}
 *     onReset={() => setSearchQuery('')}
 *   />
 * )}
 * ```
 */
export function NoSearchResults({ query, onReset }: NoSearchResultsProps) {
  return (
    <EmptyState
      title="検索結果がありません"
      description={
        query
          ? `「${query}」に一致する結果が見つかりませんでした。`
          : '条件に一致する結果が見つかりませんでした。'
      }
      actionLabel={onReset ? '検索条件をリセット' : undefined}
      onAction={onReset}
    />
  );
}

/**
 * ErrorState のプロパティ
 */
export interface ErrorStateProps {
  /** エラーメッセージ */
  message?: string;
  /** リトライのコールバック */
  onRetry?: () => void;
}

/**
 * エラー状態表示
 *
 * エラーが発生した場合に表示。
 *
 * @example
 * ```tsx
 * {error && (
 *   <ErrorState
 *     message={error.message}
 *     onRetry={() => refetch()}
 *   />
 * )}
 * ```
 */
export function ErrorState({
  message = 'エラーが発生しました',
  onRetry,
}: ErrorStateProps) {
  return (
    <EmptyState
      title="エラーが発生しました"
      description={message}
      actionLabel={onRetry ? '再試行' : undefined}
      onAction={onRetry}
    />
  );
}
