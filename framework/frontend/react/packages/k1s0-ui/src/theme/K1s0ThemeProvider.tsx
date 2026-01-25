import React, { createContext, useContext, useMemo, useState, useCallback } from 'react';
import { ThemeProvider, CssBaseline } from '@mui/material';
import { createK1s0Theme, type K1s0ThemeOptions } from './createK1s0Theme.js';

/**
 * テーマコンテキストの値
 */
interface K1s0ThemeContextValue {
  /** 現在ダークモードかどうか */
  darkMode: boolean;
  /** ダークモードを切り替える */
  toggleDarkMode: () => void;
  /** ダークモードを設定する */
  setDarkMode: (dark: boolean) => void;
}

const K1s0ThemeContext = createContext<K1s0ThemeContextValue | null>(null);

/**
 * K1s0ThemeProvider のプロパティ
 */
export interface K1s0ThemeProviderProps {
  children: React.ReactNode;
  /** 初期のダークモード状態 */
  defaultDarkMode?: boolean;
  /** 追加のテーマオプション */
  themeOptions?: Omit<K1s0ThemeOptions, 'darkMode'>;
  /** CSSベースラインを含めるか */
  includeCssBaseline?: boolean;
}

/**
 * k1s0 テーマプロバイダー
 *
 * MUI ThemeProvider をラップし、k1s0 共通テーマを適用する。
 * ダークモードの切り替え機能も提供する。
 *
 * @example
 * ```tsx
 * import { K1s0ThemeProvider } from '@k1s0/ui/theme';
 *
 * function App() {
 *   return (
 *     <K1s0ThemeProvider>
 *       <MyApp />
 *     </K1s0ThemeProvider>
 *   );
 * }
 * ```
 */
export function K1s0ThemeProvider({
  children,
  defaultDarkMode = false,
  themeOptions = {},
  includeCssBaseline = true,
}: K1s0ThemeProviderProps) {
  const [darkMode, setDarkMode] = useState(defaultDarkMode);

  const toggleDarkMode = useCallback(() => {
    setDarkMode((prev) => !prev);
  }, []);

  const theme = useMemo(
    () => createK1s0Theme({ ...themeOptions, darkMode }),
    [darkMode, themeOptions]
  );

  const contextValue = useMemo<K1s0ThemeContextValue>(
    () => ({
      darkMode,
      toggleDarkMode,
      setDarkMode,
    }),
    [darkMode, toggleDarkMode]
  );

  return (
    <K1s0ThemeContext.Provider value={contextValue}>
      <ThemeProvider theme={theme}>
        {includeCssBaseline && <CssBaseline />}
        {children}
      </ThemeProvider>
    </K1s0ThemeContext.Provider>
  );
}

/**
 * テーマコンテキストを使用するフック
 *
 * @example
 * ```tsx
 * function DarkModeToggle() {
 *   const { darkMode, toggleDarkMode } = useK1s0Theme();
 *
 *   return (
 *     <Switch checked={darkMode} onChange={toggleDarkMode} />
 *   );
 * }
 * ```
 */
export function useK1s0Theme(): K1s0ThemeContextValue {
  const context = useContext(K1s0ThemeContext);
  if (!context) {
    throw new Error('useK1s0Theme must be used within a K1s0ThemeProvider');
  }
  return context;
}
