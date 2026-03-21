// API CRUD エンドポイント E2E テスト
// BFF 経由で各サービスの CRUD エンドポイントがルーティングされていることを検証する。
// 認証が必要なエンドポイントは未認証で 401 が返ること、
// また、存在しないリソースは 401（認証エラーが先立つ）または 404 が返ることを確認する。
import { test, expect } from "@playwright/test";

const BFF_BASE = "http://localhost:8082";

// 認証が必要なため、未認証リクエストは 401 を返す
const AUTH_ERROR_STATUSES = [401, 403];

test.describe("Order API ルーティング確認", () => {
  // GET /orders のエンドポイントが BFF に登録されていることを確認する
  test("GET /bff/api/v1/orders がルーティングされていること (RESTful パス)", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/api/v1/orders`);
    // 認証エラーが返れば、エンドポイントは存在する（404 ではない）
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
  });

  test("POST /bff/api/v1/orders がルーティングされていること", async ({
    request,
  }) => {
    const response = await request.post(`${BFF_BASE}/bff/api/v1/orders`, {
      headers: { "Content-Type": "application/json" },
      data: {},
    });
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
  });

  test("GET /bff/api/v1/orders/:id がルーティングされていること", async ({
    request,
  }) => {
    const response = await request.get(
      `${BFF_BASE}/bff/api/v1/orders/non-existent-id`
    );
    // 認証エラー (401) または存在しない場合は 404
    expect([...AUTH_ERROR_STATUSES, 404]).toContain(response.status());
  });

  // 旧 RPC スタイルのパスは 404 を返すことを確認する (Finding 9 修正の確認)
  test("旧 RPC スタイル /bff/api/v1/list_orders は 404 が返ること", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/api/v1/list_orders`);
    expect(response.status()).toBe(404);
  });
});

test.describe("Inventory API ルーティング確認", () => {
  test("GET /bff/api/v1/inventory がルーティングされていること", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/api/v1/inventory`);
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
  });

  test("POST /bff/api/v1/inventory/reserve がルーティングされていること", async ({
    request,
  }) => {
    const response = await request.post(
      `${BFF_BASE}/bff/api/v1/inventory/reserve`,
      {
        headers: { "Content-Type": "application/json" },
        data: {},
      }
    );
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
  });

  // 旧 RPC スタイルのパスは 404 を返すことを確認する
  test("旧 RPC スタイル /bff/api/v1/list_inventory は 404 が返ること", async ({
    request,
  }) => {
    const response = await request.get(
      `${BFF_BASE}/bff/api/v1/list_inventory`
    );
    expect(response.status()).toBe(404);
  });
});

test.describe("Payment API ルーティング確認", () => {
  test("GET /bff/api/v1/payments がルーティングされていること", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/api/v1/payments`);
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
  });

  test("POST /bff/api/v1/payments がルーティングされていること", async ({
    request,
  }) => {
    const response = await request.post(`${BFF_BASE}/bff/api/v1/payments`, {
      headers: { "Content-Type": "application/json" },
      data: {},
    });
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
  });

  // 旧 RPC スタイルのパスは 404 を返すことを確認する
  test("旧 RPC スタイル /bff/api/v1/list_payments は 404 が返ること", async ({
    request,
  }) => {
    const response = await request.get(
      `${BFF_BASE}/bff/api/v1/list_payments`
    );
    expect(response.status()).toBe(404);
  });
});

test.describe("エラーレスポンス形式", () => {
  // 401 レスポンスが JSON または適切な Content-Type を返すことを確認する
  test("保護エンドポイントの 401 レスポンスが Content-Type を返すこと", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/api/v1/orders`, {
      headers: { Accept: "application/json" },
    });
    expect(response.status()).toBe(401);
    // Content-Type ヘッダーが設定されていることを確認する
    const contentType = response.headers()["content-type"] ?? "";
    expect(contentType).toBeTruthy();
  });
});
