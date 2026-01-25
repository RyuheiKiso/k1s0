/**
 * k1s0 共通スペーシング設定
 *
 * デザイン方針:
 * - 8px グリッドベース
 * - 一貫した余白システム
 */

/**
 * 基準となるスペーシング単位（px）
 * MUI の spacing 関数で使用される
 * spacing(1) = 8px, spacing(2) = 16px, ...
 */
export const spacingUnit = 8;

/**
 * セマンティックなスペーシング値
 * コンポーネント間の余白に使用
 */
export const spacing = {
  /** 極小: 4px */
  xs: 0.5,
  /** 小: 8px */
  sm: 1,
  /** 中: 16px */
  md: 2,
  /** 大: 24px */
  lg: 3,
  /** 極大: 32px */
  xl: 4,
  /** 特大: 48px */
  xxl: 6,
} as const;

/**
 * コンテナのパディング
 */
export const containerPadding = {
  /** モバイル */
  mobile: 2, // 16px
  /** タブレット */
  tablet: 3, // 24px
  /** デスクトップ */
  desktop: 4, // 32px
} as const;

/**
 * カードのパディング
 */
export const cardPadding = {
  /** 小 */
  sm: 2, // 16px
  /** 中 */
  md: 3, // 24px
  /** 大 */
  lg: 4, // 32px
} as const;

/**
 * フォーム要素間のギャップ
 */
export const formSpacing = {
  /** フィールド間 */
  fieldGap: 3, // 24px
  /** セクション間 */
  sectionGap: 4, // 32px
  /** ラベルとフィールド間 */
  labelGap: 1, // 8px
} as const;

/**
 * レイアウトのブレークポイント
 */
export const breakpoints = {
  xs: 0,
  sm: 600,
  md: 900,
  lg: 1200,
  xl: 1536,
} as const;

export type K1s0Spacing = typeof spacing;
export type K1s0Breakpoints = typeof breakpoints;
