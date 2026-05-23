// manufacturing_stress 専用 Playwright 設定ファイル
// 01_Bidi 適合仕様 §158-160 ship blocker: 9 製造業 RPC stress test の E2E 検証を実行する
// tier1 gateway を page.route() でモックし、bidi streaming assertionを ブラウザ内 JS で検証する

// @playwright/test の defineConfig をインポートする
import { defineConfig, devices } from "@playwright/test";

// 製造業 stress test Playwright 設定をエクスポートする
export default defineConfig({
  // テストディレクトリ: 9 spec を格納した各 RPC サブディレクトリを対象にする
  testDir: ".",
  // 各サブディレクトリ内の *.spec.ts を全て対象とする
  testMatch: "*/*.spec.ts",
  // 並列実行を無効化する（tier1 gateway モックとの競合を防ぐ）
  fullyParallel: false,
  // CI 環境では再試行を 1 回行い flaky を検出する
  retries: process.env.CI ? 1 : 0,
  // ワーカー数は CI では 1 に固定して安定させる
  workers: process.env.CI ? 1 : undefined,
  // レポーター: CI では GitHub Actions 形式、ローカルでは HTML
  reporter: process.env.CI ? "github" : "html",
  // 全テスト共通設定
  use: {
    // ベース URL: ローカルモックサーバーを想定する
    baseURL: "http://localhost:4173",
    // 失敗時にトレースを収集する
    trace: "on-first-retry",
    // ヘッドレスモードで実行する
    headless: true,
    // テストタイムアウト: 各 stress scenario は最大 30 秒
    actionTimeout: 30_000,
  },
  // Chromium のみを対象とする（製造業 pack の対象ブラウザ）
  projects: [
    {
      // Chromium プロジェクトで全 9 stress scenario を実行する
      name: "chromium",
      // Desktop Chrome デバイス設定を使用する
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
