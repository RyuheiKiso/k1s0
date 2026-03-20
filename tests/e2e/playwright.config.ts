// k1s0 E2E テスト用 Playwright 設定
// Docker Compose で起動したサービス群に対してテストを実行する
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  // テストファイルの配置ディレクトリ
  testDir: "./specs",

  // テストの並列実行設定（CI では並列度を制限）
  fullyParallel: true,
  forbidOnly: !!process.env.CI,

  // リトライ設定: CI では 2 回リトライ、ローカルでは 0 回
  retries: process.env.CI ? 2 : 0,

  // 並列ワーカー数: CI では 1 ワーカーに制限し、リソース消費を抑える
  workers: process.env.CI ? 1 : undefined,

  // テストレポート形式: HTML レポートを生成
  reporter: process.env.CI ? "github" : "html",

  // 全テスト共通の設定
  use: {
    // BFF プロキシ経由でアクセスするベース URL
    baseURL: process.env.BASE_URL || "http://localhost:8082",

    // テスト失敗時にスクリーンショットを取得
    screenshot: "only-on-failure",

    // テスト失敗時にトレースを記録
    trace: "on-first-retry",
  },

  // タイムアウト設定: E2E テストは 30 秒まで許容
  timeout: 30_000,

  // ブラウザ設定
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
