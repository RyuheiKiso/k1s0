// src/tier3/typescript/packages/design-tokens/src/tokens.ts
// k1s0 tier3 design token 定義 (CSS variable + TypeScript 型安全アクセス)
// 全 component はこのトークンを参照して WCAG 2.1 AA + Web Vitals を満たす

// カラートークンを定義する (light mode)
export const COLOR_TOKENS = {
  // プライマリカラー: ブランド色
  primary: {
    // メインカラー (コントラスト比 ≥ 4.5:1 を保証する)
    main: "var(--k1s0-color-primary-main, #1a73e8)",
    // ホバー時のカラー
    hover: "var(--k1s0-color-primary-hover, #1557b0)",
    // 無効時のカラー
    disabled: "var(--k1s0-color-primary-disabled, #a8c7fa)",
    // コントラスト (テキスト色)
    contrast: "var(--k1s0-color-primary-contrast, #ffffff)",
  },
  // セマンティックカラー: 意味を持つ色
  semantic: {
    // 成功状態
    success: "var(--k1s0-color-success, #137333)",
    // 警告状態
    warning: "var(--k1s0-color-warning, #ea8600)",
    // エラー状態
    error: "var(--k1s0-color-error, #c5221f)",
    // 情報状態
    info: "var(--k1s0-color-info, #1a73e8)",
  },
  // 背景カラー
  background: {
    // デフォルト背景
    default: "var(--k1s0-color-bg-default, #ffffff)",
    // サーフェス背景 (カード等)
    surface: "var(--k1s0-color-bg-surface, #f8f9fa)",
    // 強調背景
    elevated: "var(--k1s0-color-bg-elevated, #e8eaed)",
  },
  // テキストカラー
  text: {
    // メインテキスト (コントラスト比 ≥ 7:1)
    primary: "var(--k1s0-color-text-primary, #202124)",
    // サブテキスト (コントラスト比 ≥ 4.5:1)
    secondary: "var(--k1s0-color-text-secondary, #5f6368)",
    // 無効テキスト
    disabled: "var(--k1s0-color-text-disabled, #bdc1c6)",
  },
} as const;

// タイポグラフィトークンを定義する
export const TYPOGRAPHY_TOKENS = {
  // フォントファミリー (日本語 / 英語 対応)
  fontFamily: {
    // サンセリフ (UI テキスト)
    sans: "var(--k1s0-font-sans, 'Noto Sans JP', 'Roboto', sans-serif)",
    // モノスペース (コード表示)
    mono: "var(--k1s0-font-mono, 'Noto Sans Mono', 'Roboto Mono', monospace)",
  },
  // フォントサイズスケール
  fontSize: {
    // 極小
    xs: "var(--k1s0-font-size-xs, 0.75rem)",
    // 小
    sm: "var(--k1s0-font-size-sm, 0.875rem)",
    // 標準
    md: "var(--k1s0-font-size-md, 1rem)",
    // 大
    lg: "var(--k1s0-font-size-lg, 1.125rem)",
    // 特大
    xl: "var(--k1s0-font-size-xl, 1.25rem)",
    // 見出し 1
    h1: "var(--k1s0-font-size-h1, 2rem)",
    // 見出し 2
    h2: "var(--k1s0-font-size-h2, 1.5rem)",
    // 見出し 3
    h3: "var(--k1s0-font-size-h3, 1.25rem)",
  },
} as const;

// スペーシングトークンを定義する (4px 基準グリッド)
export const SPACING_TOKENS = {
  // 1 unit = 4px
  unit: "var(--k1s0-spacing-unit, 4px)",
  // 各サイズ
  xs: "var(--k1s0-spacing-xs, 4px)",
  sm: "var(--k1s0-spacing-sm, 8px)",
  md: "var(--k1s0-spacing-md, 16px)",
  lg: "var(--k1s0-spacing-lg, 24px)",
  xl: "var(--k1s0-spacing-xl, 32px)",
  xxl: "var(--k1s0-spacing-xxl, 48px)",
} as const;

// アニメーションデュレーショントークンを定義する
export const ANIMATION_TOKENS = {
  // 極短 (ホバー等)
  fast: "var(--k1s0-animation-fast, 100ms)",
  // 標準 (フェード等)
  normal: "var(--k1s0-animation-normal, 200ms)",
  // 長め (ページ遷移等)
  slow: "var(--k1s0-animation-slow, 300ms)",
  // イージング関数
  easing: {
    // 標準イージング
    standard: "var(--k1s0-easing-standard, cubic-bezier(0.4, 0, 0.2, 1))",
    // 入力イージング
    enter: "var(--k1s0-easing-enter, cubic-bezier(0, 0, 0.2, 1))",
    // 退出イージング
    exit: "var(--k1s0-easing-exit, cubic-bezier(0.4, 0, 1, 1))",
  },
} as const;
