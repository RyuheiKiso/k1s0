// T-05 対応: E2E テスト実行前のサービス待機グローバルセットアップ
// playwright.config.ts の globalSetup として登録し、全テスト実行前にサービスの準備完了を確認する。
// Docker Compose で起動したサービス群（BFF, Keycloak 等）のヘルスチェックエンドポイントを
// ポーリングして、準備完了を待機する。
import { FullConfig } from "@playwright/test";
import { request } from "@playwright/test";

// サービスが準備完了するまでポーリングする最大待機時間（ミリ秒）
const MAX_WAIT_MS = 120_000;
// ポーリング間隔（ミリ秒）
const POLL_INTERVAL_MS = 2_000;

/**
 * 指定 URL が 200 OK を返すまでポーリングする。
 * 最大 maxWaitMs ミリ秒待機し、タイムアウト時は Error をスローする。
 */
async function waitForService(url: string, maxWaitMs: number): Promise<void> {
  const startTime = Date.now();
  const context = await request.newContext();
  try {
    while (Date.now() - startTime < maxWaitMs) {
      try {
        const response = await context.get(url, {
          timeout: 5_000,
          failOnStatusCode: false,
        });
        if (response.ok()) {
          console.log(`[global-setup] ${url} is ready`);
          return;
        }
      } catch {
        // 接続拒否や DNS エラーは無視してリトライする
      }
      const elapsed = Math.round((Date.now() - startTime) / 1000);
      console.log(
        `[global-setup] Waiting for ${url}... (${elapsed}s / ${maxWaitMs / 1000}s)`,
      );
      await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL_MS));
    }
    throw new Error(
      `[global-setup] Timeout waiting for ${url} after ${maxWaitMs}ms`,
    );
  } finally {
    await context.dispose();
  }
}

/**
 * 全テスト実行前に呼び出されるグローバルセットアップ関数。
 * BFF プロキシと（利用可能な場合）Keycloak の準備完了を待機する。
 */
export default async function globalSetup(_config: FullConfig): Promise<void> {
  const bffBase = process.env.BASE_URL ?? "http://localhost:8082";
  const keycloakBase = process.env.KEYCLOAK_URL ?? "http://localhost:8080";
  const keycloakAvailable = process.env.TEST_KEYCLOAK_AVAILABLE === "true";

  console.log("[global-setup] Waiting for services to be ready...");

  // BFF プロキシのヘルスチェックを待機する
  await waitForService(`${bffBase}/healthz`, MAX_WAIT_MS);

  // Keycloak が有効な場合はヘルスチェックを待機する
  if (keycloakAvailable) {
    const keycloakHealthUrl = `${keycloakBase}/health/ready`;
    await waitForService(keycloakHealthUrl, MAX_WAIT_MS);
  }

  console.log("[global-setup] All services are ready. Starting tests.");
}
