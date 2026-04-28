// 本ファイルは tier3 web pnpm workspace 共通の ESLint flat config。
// 設計正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/03_web配置.md
//   docs/SHIP_STATUS.md（tier3/web/packages 4 種 + apps 3 種、リリース時点 同梱）
//
// 役割:
//   workspace 配下 7 package（packages/{config,api-client,ui,i18n} + apps/{portal,admin,docs-site}）の
//   `pnpm run lint` （内部実行は `eslint src`）から本ファイルが flat config として
//   解決される。最小限の TypeScript + (任意) React 規則を共通適用し、過剰なルールで
//   skeleton 状態の実装が落ちないよう unused-vars / explicit-any はベース推奨より緩めに保つ。

// JavaScript 推奨ルール一式。
import js from "@eslint/js";
// TypeScript 用 ESLint プラグイン（flat config 統合済の typescript-eslint パッケージから提供）。
import tseslint from "typescript-eslint";
// グローバル変数集（browser / node / es2024）を提供。
import globals from "globals";

// flat config 配列を export する。後段の設定が前段を上書きする順序解決ルール。
export default [
  // node_modules / dist / coverage は走査対象外（pnpm workspace で各 package が
  // ローカルに dist/ を持つため、明示的に除外する）。
  {
    ignores: [
      "**/node_modules/**",
      "**/dist/**",
      "**/coverage/**",
      "**/.vite/**",
      "**/build/**",
    ],
  },

  // JavaScript 推奨ルール（var 禁止、unused 警告、no-undef 等の安全網）。
  js.configs.recommended,

  // TypeScript ファイル（.ts / .tsx）には typescript-eslint の推奨を適用。
  // 配列を spread することで複数 config が flat config に展開される。
  ...tseslint.configs.recommended,

  // 共通の言語オプション。Vite + React + ESM 前提のため module / browser globals を有効化。
  {
    files: ["**/*.{ts,tsx,js,jsx,mjs,cjs}"],
    languageOptions: {
      ecmaVersion: 2024,
      sourceType: "module",
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.es2024,
      },
    },
    rules: {
      // skeleton 状態で `_` プレフィックスの引数を許容する一般的緩和。
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
      // skeleton 段階の placeholder 関数で `any` 戻り値を許容（plan 04-13 で順次 typed 化）。
      "@typescript-eslint/no-explicit-any": "warn",
    },
  },
];
