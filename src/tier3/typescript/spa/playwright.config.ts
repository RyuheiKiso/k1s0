// k1s0 tier3 SPA Playwright テスト設定
// 8 stress scenario の E2E テストを定義する
// msw による tier1 gateway モックと page.evaluate() を組み合わせた stress test を実行する

// @playwright/test の defineConfig をインポートする
import { defineConfig, devices } from '@playwright/test';

// Playwright 設定オブジェクトをエクスポートする
export default defineConfig({
  // テストディレクトリ（bidi stress scenario を格納する）
  testDir: './tests/playwright',
  // テスト実行前に全テストを検出する（テスト依存 globbing）
  testMatch: '**/*.spec.ts',
  // 並列実行: 各ファイル内は直列、ファイル間は並列にする
  fullyParallel: false,
  // CI 環境では再試行を 1 回行う（flaky 検出用）
  retries: process.env.CI ? 1 : 0,
  // ワーカー数（CI では 1 ワーカーに固定して安定させる）
  workers: process.env.CI ? 1 : undefined,
  // レポーター設定（CI では GitHub Actions 形式で出力する）
  reporter: process.env.CI ? 'github' : 'html',
  // 全テスト共通の設定
  use: {
    // msw を起動した後の SPA URL をベース URL にする（devServer との組み合わせを想定する）
    baseURL: 'http://localhost:5173',
    // テスト失敗時にトレースを収集する
    trace: 'on-first-retry',
    // ヘッドレスモードで実行する（CI 環境対応）
    headless: true,
  },
  // プロジェクト設定（Chromium のみでストレステストを実行する）
  projects: [
    {
      // Chromium で全 stress scenario を実行する
      name: 'chromium',
      // Chromium デバイス設定を使用する
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  // dev server の自動起動設定（pnpm dev でサーバーを立ち上げる）
  webServer: {
    // Vite dev server を起動するコマンド
    command: 'pnpm dev',
    // dev server の待機 URL
    url: 'http://localhost:5173',
    // dev server の起動完了を待機する
    reuseExistingServer: !process.env.CI,
    // 起動タイムアウト（30 秒）
    timeout: 30_000,
  },
});
