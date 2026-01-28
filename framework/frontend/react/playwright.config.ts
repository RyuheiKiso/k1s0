import { defineConfig, devices } from '@playwright/test';

/**
 * k1s0 React Framework E2E テスト設定
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  // E2E テストディレクトリ
  testDir: './e2e',

  // テストファイルのパターン
  testMatch: '**/*.spec.ts',

  // 完全な並列実行
  fullyParallel: true,

  // CI では失敗したテストの再試行を許可しない
  forbidOnly: !!process.env.CI,

  // CI では失敗時に 2 回再試行
  retries: process.env.CI ? 2 : 0,

  // CI では並列ワーカー数を制限
  workers: process.env.CI ? 1 : undefined,

  // レポーター設定
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['list'],
    ...(process.env.CI ? [['github'] as ['github']] : []),
  ],

  // 共通設定
  use: {
    // ベース URL
    baseURL: process.env.BASE_URL || 'http://localhost:3000',

    // トレースを収集（失敗時のみ）
    trace: 'on-first-retry',

    // スクリーンショット（失敗時のみ）
    screenshot: 'only-on-failure',

    // ビデオ録画（失敗時のみ）
    video: 'on-first-retry',

    // ロケール
    locale: 'ja-JP',

    // タイムゾーン
    timezoneId: 'Asia/Tokyo',
  },

  // タイムアウト設定
  timeout: 30000,
  expect: {
    timeout: 5000,
  },

  // プロジェクト（ブラウザ）設定
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    // モバイルビューポート
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 13'] },
    },
  ],

  // ローカル開発サーバー設定
  webServer: process.env.CI
    ? undefined
    : {
        command: 'pnpm dev',
        url: 'http://localhost:3000',
        reuseExistingServer: !process.env.CI,
        timeout: 120000,
      },

  // 出力ディレクトリ
  outputDir: 'playwright-results',
});
