// ESLint v9 フラットコンフィグ: TypeScript + React用リント設定
import js from '@eslint/js';
import tsPlugin from '@typescript-eslint/eslint-plugin';
import tsParser from '@typescript-eslint/parser';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
// ブラウザグローバル変数の定義（window, document等）
import globals from 'globals';

export default [
  // 基本のJavaScript推奨ルール
  js.configs.recommended,
  {
    // TypeScriptファイルに対するルール設定
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: {
      // ブラウザ環境のグローバル変数を許可
      globals: {
        ...globals.browser,
      },
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 2020,
        sourceType: 'module',
        ecmaFeatures: { jsx: true },
      },
    },
    settings: {
      react: {
        // React 17+ JSX Transform（import React不要）
        runtime: 'automatic',
      },
    },
    plugins: {
      '@typescript-eslint': tsPlugin,
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...tsPlugin.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      // 未使用変数の警告（_プレフィックスは許可）
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      // L-17 監査対応: no-explicit-any を warn から error に厳格化する。
      // any 型の使用は TypeScript の型安全性を損なうため、ビルドエラーとして検出する。
      '@typescript-eslint/no-explicit-any': 'error',
      // L-18 監査対応: exhaustive-deps を warn から error に厳格化する。
      // フックの依存配列の不備はバグの温床となるため、ビルドエラーとして検出する。
      'react-hooks/exhaustive-deps': 'error',
      // TypeScriptが未定義チェックを行うため、ESLintのno-undefは無効化
      // React 17+ JSX Transformでは明示的なReact importが不要
      'no-undef': 'off',
    },
  },
  {
    // テストファイルとノード設定ファイルを除外
    ignores: ['dist/', 'node_modules/', '*.config.*'],
  },
];
