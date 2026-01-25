import { createTheme, type ThemeOptions } from '@mui/material/styles';
import { palette, darkPalette } from './palette.js';
import { typography } from './typography.js';
import { spacingUnit, breakpoints } from './spacing.js';
import { components } from './components.js';

/**
 * k1s0 テーマ作成オプション
 */
export interface K1s0ThemeOptions {
  /** ダークモードを使用するか */
  darkMode?: boolean;
  /** 追加のテーマオプション（MUIのThemeOptions） */
  overrides?: Partial<ThemeOptions>;
}

/**
 * k1s0 共通テーマを作成
 *
 * @param options - テーマオプション
 * @returns MUI Theme
 *
 * @example
 * ```tsx
 * import { ThemeProvider } from '@mui/material/styles';
 * import { createK1s0Theme } from '@k1s0/ui/theme';
 *
 * const theme = createK1s0Theme();
 *
 * function App() {
 *   return (
 *     <ThemeProvider theme={theme}>
 *       <MyApp />
 *     </ThemeProvider>
 *   );
 * }
 * ```
 */
export function createK1s0Theme(options: K1s0ThemeOptions = {}) {
  const { darkMode = false, overrides = {} } = options;

  const baseTheme: ThemeOptions = {
    palette: darkMode
      ? { ...darkPalette, mode: 'dark' }
      : { ...palette, mode: 'light' },
    typography,
    spacing: spacingUnit,
    breakpoints: {
      values: breakpoints,
    },
    shape: {
      borderRadius: 8,
    },
    components,
  };

  // ベーステーマとオーバーライドをマージ
  const mergedTheme = deepMerge(baseTheme, overrides);

  return createTheme(mergedTheme);
}

/**
 * ライトモード用のデフォルトテーマ
 */
export const k1s0LightTheme = createK1s0Theme({ darkMode: false });

/**
 * ダークモード用のデフォルトテーマ
 */
export const k1s0DarkTheme = createK1s0Theme({ darkMode: true });

/**
 * オブジェクトの深いマージ
 */
function deepMerge<T extends object>(target: T, source: Partial<T>): T {
  const result = { ...target };

  for (const key in source) {
    const sourceValue = source[key];
    const targetValue = (target as Record<string, unknown>)[key];

    if (
      sourceValue &&
      typeof sourceValue === 'object' &&
      !Array.isArray(sourceValue) &&
      targetValue &&
      typeof targetValue === 'object' &&
      !Array.isArray(targetValue)
    ) {
      (result as Record<string, unknown>)[key] = deepMerge(
        targetValue as object,
        sourceValue as object
      );
    } else if (sourceValue !== undefined) {
      (result as Record<string, unknown>)[key] = sourceValue;
    }
  }

  return result;
}
