// k1s0 ヘルスチェック E2E テスト
// 各サービスの /healthz エンドポイントが正常応答することを検証する
import { test, expect } from "@playwright/test";
// URL やポートはすべて config.ts から取得し、ハードコードを排除する
import { serviceUrl, KEYCLOAK_BASE, SERVICE_PORTS } from "./config";

// System tier サービスのヘルスチェック対象一覧（SERVICE_PORTS のキーを使用）
const SYSTEM_SERVICE_NAMES = Object.keys(SERVICE_PORTS) as Array<
  keyof typeof SERVICE_PORTS
>;

test.describe("System サービスヘルスチェック", () => {
  // 各サービスの /healthz が 200 を返すことを検証する
  for (const name of SYSTEM_SERVICE_NAMES) {
    test(`${name} サービスが正常稼働していること`, async ({
      request,
    }) => {
      // ヘルスチェックエンドポイントにリクエストを送信
      const response = await request.get(serviceUrl(name, "/healthz"));

      // HTTP 200 が返ることを検証
      expect(response.status()).toBe(200);
    });
  }
});

test.describe("インフラサービスヘルスチェック", () => {
  test("Keycloak が正常稼働していること", async ({ request }) => {
    // Keycloak の Realm 情報エンドポイントにリクエストを送信
    const response = await request.get(
      `${KEYCLOAK_BASE}/realms/k1s0/.well-known/openid-configuration`
    );

    // HTTP 200 が返ることを検証
    expect(response.status()).toBe(200);
  });
});
