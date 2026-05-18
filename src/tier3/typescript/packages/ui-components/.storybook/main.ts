// main.ts — Storybook 設定
// 設計方針 32_開発者体験: Storybook component カタログ

// Storybook config の型をインポートする
import type { StorybookConfig } from '@storybook/react-vite';

// Storybook config を定義する
const config: StorybookConfig = {
  // stories ファイルの検索パターンを設定する
  stories: ['../src/**/*.stories.@(js|jsx|ts|tsx)'],
  // アドオン設定: アクセシビリティ + docs を有効にする
  addons: [
    // 基本アドオン（controls / actions / viewport / backgrounds）
    '@storybook/addon-essentials',
    // a11y 検査アドオン
    '@storybook/addon-a11y',
    // ドキュメント生成アドオン
    '@storybook/addon-docs',
  ],
  // Vite をビルドシステムとして使用する
  framework: {
    name: '@storybook/react-vite',
    options: {},
  },
};

// config をデフォルトエクスポートする
export default config;
