/**
 * k1s0 共通カラーパレット
 *
 * デザイン方針:
 * - プライマリ: 信頼性・専門性を表すブルー系
 * - セカンダリ: アクセントとして使用するティール系
 * - エラー/警告/成功/情報: セマンティックカラー
 */

export const palette = {
  // プライマリカラー（ブルー系）
  primary: {
    main: '#1976d2',
    light: '#42a5f5',
    dark: '#1565c0',
    contrastText: '#ffffff',
  },

  // セカンダリカラー（ティール系）
  secondary: {
    main: '#009688',
    light: '#4db6ac',
    dark: '#00796b',
    contrastText: '#ffffff',
  },

  // エラー（レッド系）
  error: {
    main: '#d32f2f',
    light: '#ef5350',
    dark: '#c62828',
    contrastText: '#ffffff',
  },

  // 警告（オレンジ系）
  warning: {
    main: '#ed6c02',
    light: '#ff9800',
    dark: '#e65100',
    contrastText: '#ffffff',
  },

  // 情報（ライトブルー系）
  info: {
    main: '#0288d1',
    light: '#03a9f4',
    dark: '#01579b',
    contrastText: '#ffffff',
  },

  // 成功（グリーン系）
  success: {
    main: '#2e7d32',
    light: '#4caf50',
    dark: '#1b5e20',
    contrastText: '#ffffff',
  },

  // グレースケール
  grey: {
    50: '#fafafa',
    100: '#f5f5f5',
    200: '#eeeeee',
    300: '#e0e0e0',
    400: '#bdbdbd',
    500: '#9e9e9e',
    600: '#757575',
    700: '#616161',
    800: '#424242',
    900: '#212121',
  },

  // 背景色
  background: {
    default: '#fafafa',
    paper: '#ffffff',
  },

  // テキスト色
  text: {
    primary: 'rgba(0, 0, 0, 0.87)',
    secondary: 'rgba(0, 0, 0, 0.6)',
    disabled: 'rgba(0, 0, 0, 0.38)',
  },

  // 区切り線
  divider: 'rgba(0, 0, 0, 0.12)',

  // アクション状態
  action: {
    active: 'rgba(0, 0, 0, 0.54)',
    hover: 'rgba(0, 0, 0, 0.04)',
    selected: 'rgba(0, 0, 0, 0.08)',
    disabled: 'rgba(0, 0, 0, 0.26)',
    disabledBackground: 'rgba(0, 0, 0, 0.12)',
    focus: 'rgba(0, 0, 0, 0.12)',
  },
} as const;

/**
 * ダークモード用パレット
 */
export const darkPalette = {
  ...palette,
  mode: 'dark' as const,

  primary: {
    main: '#90caf9',
    light: '#e3f2fd',
    dark: '#42a5f5',
    contrastText: 'rgba(0, 0, 0, 0.87)',
  },

  secondary: {
    main: '#80cbc4',
    light: '#b2dfdb',
    dark: '#4db6ac',
    contrastText: 'rgba(0, 0, 0, 0.87)',
  },

  background: {
    default: '#121212',
    paper: '#1e1e1e',
  },

  text: {
    primary: '#ffffff',
    secondary: 'rgba(255, 255, 255, 0.7)',
    disabled: 'rgba(255, 255, 255, 0.5)',
  },

  divider: 'rgba(255, 255, 255, 0.12)',

  action: {
    active: '#ffffff',
    hover: 'rgba(255, 255, 255, 0.08)',
    selected: 'rgba(255, 255, 255, 0.16)',
    disabled: 'rgba(255, 255, 255, 0.3)',
    disabledBackground: 'rgba(255, 255, 255, 0.12)',
    focus: 'rgba(255, 255, 255, 0.12)',
  },
} as const;

export type K1s0Palette = typeof palette;
export type K1s0DarkPalette = typeof darkPalette;
