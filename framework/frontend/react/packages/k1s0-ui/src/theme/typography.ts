import type { TypographyOptions } from '@mui/material/styles/createTypography';

/**
 * k1s0 共通タイポグラフィ設定
 *
 * デザイン方針:
 * - 日本語対応を考慮したフォントファミリー
 * - 読みやすさを重視したラインハイト
 * - 一貫したスケール
 */

// フォントファミリー
const fontFamily = [
  '-apple-system',
  'BlinkMacSystemFont',
  '"Segoe UI"',
  'Roboto',
  '"Hiragino Sans"',
  '"Noto Sans JP"',
  '"Yu Gothic UI"',
  'Meiryo',
  'sans-serif',
].join(',');

// モノスペースフォント（コード表示用）
const monoFontFamily = [
  '"SF Mono"',
  'Monaco',
  'Consolas',
  '"Liberation Mono"',
  '"Courier New"',
  'monospace',
].join(',');

export const typography: TypographyOptions = {
  fontFamily,

  // 見出し
  h1: {
    fontSize: '2.5rem',
    fontWeight: 500,
    lineHeight: 1.2,
    letterSpacing: '-0.01562em',
  },
  h2: {
    fontSize: '2rem',
    fontWeight: 500,
    lineHeight: 1.2,
    letterSpacing: '-0.00833em',
  },
  h3: {
    fontSize: '1.75rem',
    fontWeight: 500,
    lineHeight: 1.2,
    letterSpacing: '0em',
  },
  h4: {
    fontSize: '1.5rem',
    fontWeight: 500,
    lineHeight: 1.2,
    letterSpacing: '0.00735em',
  },
  h5: {
    fontSize: '1.25rem',
    fontWeight: 500,
    lineHeight: 1.3,
    letterSpacing: '0em',
  },
  h6: {
    fontSize: '1.125rem',
    fontWeight: 500,
    lineHeight: 1.3,
    letterSpacing: '0.0075em',
  },

  // 字幕
  subtitle1: {
    fontSize: '1rem',
    fontWeight: 500,
    lineHeight: 1.5,
    letterSpacing: '0.00938em',
  },
  subtitle2: {
    fontSize: '0.875rem',
    fontWeight: 500,
    lineHeight: 1.5,
    letterSpacing: '0.00714em',
  },

  // 本文
  body1: {
    fontSize: '1rem',
    fontWeight: 400,
    lineHeight: 1.7,
    letterSpacing: '0.00938em',
  },
  body2: {
    fontSize: '0.875rem',
    fontWeight: 400,
    lineHeight: 1.6,
    letterSpacing: '0.01071em',
  },

  // ボタン
  button: {
    fontSize: '0.875rem',
    fontWeight: 500,
    lineHeight: 1.75,
    letterSpacing: '0.02857em',
    textTransform: 'none', // 大文字変換しない
  },

  // キャプション
  caption: {
    fontSize: '0.75rem',
    fontWeight: 400,
    lineHeight: 1.5,
    letterSpacing: '0.03333em',
  },

  // オーバーライン
  overline: {
    fontSize: '0.625rem',
    fontWeight: 500,
    lineHeight: 2,
    letterSpacing: '0.08333em',
    textTransform: 'uppercase',
  },
};

/**
 * モノスペースフォント（コード表示用）
 */
export { monoFontFamily };

export type K1s0Typography = typeof typography;
