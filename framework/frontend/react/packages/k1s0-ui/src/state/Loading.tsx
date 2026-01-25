import React from 'react';
import {
  Box,
  CircularProgress,
  LinearProgress,
  Skeleton,
  Typography,
  type CircularProgressProps,
  type LinearProgressProps,
  type SkeletonProps,
} from '@mui/material';

/**
 * LoadingSpinner のプロパティ
 */
export interface LoadingSpinnerProps extends CircularProgressProps {
  /** ローディング中のメッセージ */
  message?: string;
  /** 中央配置するか */
  centered?: boolean;
  /** オーバーレイとして表示するか */
  overlay?: boolean;
}

/**
 * ローディングスピナー
 *
 * @example
 * ```tsx
 * // 基本的な使用
 * <LoadingSpinner />
 *
 * // メッセージ付き
 * <LoadingSpinner message="読み込み中..." />
 *
 * // 中央配置
 * <LoadingSpinner centered />
 *
 * // オーバーレイ
 * <LoadingSpinner overlay />
 * ```
 */
export function LoadingSpinner({
  message,
  centered = false,
  overlay = false,
  size = 40,
  ...props
}: LoadingSpinnerProps) {
  const content = (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 2,
      }}
    >
      <CircularProgress size={size} {...props} />
      {message && (
        <Typography variant="body2" color="text.secondary">
          {message}
        </Typography>
      )}
    </Box>
  );

  if (overlay) {
    return (
      <Box
        sx={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: 'rgba(255, 255, 255, 0.8)',
          zIndex: 1000,
        }}
      >
        {content}
      </Box>
    );
  }

  if (centered) {
    return (
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: 200,
          width: '100%',
        }}
      >
        {content}
      </Box>
    );
  }

  return content;
}

/**
 * LoadingBar のプロパティ
 */
export interface LoadingBarProps extends LinearProgressProps {
  /** ローディング中かどうか */
  loading?: boolean;
}

/**
 * ローディングバー
 *
 * ページ上部などに表示する線形プログレスバー。
 *
 * @example
 * ```tsx
 * <LoadingBar loading={isLoading} />
 * ```
 */
export function LoadingBar({ loading = true, ...props }: LoadingBarProps) {
  if (!loading) {
    return null;
  }

  return (
    <LinearProgress
      {...props}
      sx={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        zIndex: 9999,
        ...props.sx,
      }}
    />
  );
}

/**
 * SkeletonLoader のプロパティ
 */
export interface SkeletonLoaderProps {
  /** 行数 */
  lines?: number;
  /** アバター付きか */
  avatar?: boolean;
  /** カード形式か */
  card?: boolean;
  /** 幅 */
  width?: SkeletonProps['width'];
  /** 高さ */
  height?: SkeletonProps['height'];
}

/**
 * スケルトンローダー
 *
 * コンテンツのプレースホルダーとして表示するスケルトン。
 *
 * @example
 * ```tsx
 * // テキスト行
 * <SkeletonLoader lines={3} />
 *
 * // アバター付き
 * <SkeletonLoader avatar lines={2} />
 *
 * // カード形式
 * <SkeletonLoader card />
 * ```
 */
export function SkeletonLoader({
  lines = 3,
  avatar = false,
  card = false,
  width,
  height,
}: SkeletonLoaderProps) {
  if (card) {
    return (
      <Box sx={{ width: width ?? '100%' }}>
        <Skeleton
          variant="rectangular"
          width="100%"
          height={height ?? 140}
          sx={{ borderRadius: 1 }}
        />
        <Box sx={{ pt: 1 }}>
          <Skeleton width="60%" />
          <Skeleton width="80%" />
        </Box>
      </Box>
    );
  }

  return (
    <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 2, width: width ?? '100%' }}>
      {avatar && (
        <Skeleton variant="circular" width={40} height={40} />
      )}
      <Box sx={{ flex: 1 }}>
        {Array.from({ length: lines }).map((_, index) => (
          <Skeleton
            key={index}
            width={index === lines - 1 ? '60%' : '100%'}
            height={height}
          />
        ))}
      </Box>
    </Box>
  );
}

/**
 * PageLoading のプロパティ
 */
export interface PageLoadingProps {
  /** ローディング中のメッセージ */
  message?: string;
}

/**
 * ページローディング
 *
 * ページ全体のローディング表示。
 *
 * @example
 * ```tsx
 * if (isLoading) {
 *   return <PageLoading message="ページを読み込んでいます..." />;
 * }
 * ```
 */
export function PageLoading({ message = '読み込み中...' }: PageLoadingProps) {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '50vh',
        gap: 3,
      }}
    >
      <CircularProgress size={48} />
      <Typography variant="body1" color="text.secondary">
        {message}
      </Typography>
    </Box>
  );
}
