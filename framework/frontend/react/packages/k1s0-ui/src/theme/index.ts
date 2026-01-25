/**
 * k1s0 共通テーマ
 *
 * @packageDocumentation
 */

// パレット
export { palette, darkPalette, type K1s0Palette, type K1s0DarkPalette } from './palette.js';

// タイポグラフィ
export { typography, monoFontFamily, type K1s0Typography } from './typography.js';

// スペーシング
export {
  spacingUnit,
  spacing,
  containerPadding,
  cardPadding,
  formSpacing,
  breakpoints,
  type K1s0Spacing,
  type K1s0Breakpoints,
} from './spacing.js';

// コンポーネント
export { components, type K1s0Components } from './components.js';

// テーマ作成
export {
  createK1s0Theme,
  k1s0LightTheme,
  k1s0DarkTheme,
  type K1s0ThemeOptions,
} from './createK1s0Theme.js';

// プロバイダー
export {
  K1s0ThemeProvider,
  useK1s0Theme,
  type K1s0ThemeProviderProps,
} from './K1s0ThemeProvider.js';
