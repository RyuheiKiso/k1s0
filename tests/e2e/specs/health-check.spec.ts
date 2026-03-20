// k1s0 ヘルスチェック E2E テスト
// 各サービスの /healthz エンドポイントが正常応答することを検証する
import { test, expect } from "@playwright/test";

// System tier サービスのヘルスチェック対象一覧
// ポート番号は docker-compose.yaml のホストポートマッピングに対応
const SYSTEM_SERVICES = [
  { name: "auth", port: 8083 },
  { name: "config", port: 8084 },
  { name: "saga", port: 8085 },
  { name: "dlq-manager", port: 8086 },
  { name: "featureflag", port: 8087 },
  { name: "ratelimit", port: 8088 },
  { name: "tenant", port: 8089 },
  { name: "vault", port: 8091 },
  { name: "bff-proxy", port: 8082 },
] as const;

test.describe("System サービスヘルスチェック", () => {
  // 各サービスの /healthz が 200 を返すことを検証する
  for (const service of SYSTEM_SERVICES) {
    test(`${service.name} サービスが正常稼働していること`, async ({
      request,
    }) => {
      // ヘルスチェックエンドポイントにリクエストを送信
      const response = await request.get(
        `http://localhost:${service.port}/healthz`
      );

      // HTTP 200 が返ることを検証
      expect(response.status()).toBe(200);
    });
  }
});

test.describe("インフラサービスヘルスチェック", () => {
  test("Keycloak が正常稼働していること", async ({ request }) => {
    // Keycloak の Realm 情報エンドポイントにリクエストを送信
    const response = await request.get(
      "http://localhost:8180/realms/k1s0/.well-known/openid-configuration"
    );

    // HTTP 200 が返ることを検証
    expect(response.status()).toBe(200);
  });
});
