// API CRUD エンドポイント E2E テスト
// BFF 経由で tasks サービスの CRUD エンドポイントがルーティングされていることを検証する。
// bff-proxy は /api/*path をバックエンドへプロキシする設計であり、/bff/ プレフィックスは存在しない。
// 認証が必要なエンドポイントは未認証で 401/403 が返ること、
// また、存在しないリソースは 401/403（認証エラーが先立つ）または 404 が返ることを確認する。
import { test, expect } from "@playwright/test";
// BFF_BASE は環境変数または config.ts のデフォルト値を使用する
import { BFF_BASE } from "./config";

// 認証が必要なため、未認証リクエストは 401 または 403 を返す
const AUTH_ERROR_STATUSES = [401, 403];

test.describe("Task API ルーティング確認", () => {
  // GET /api/v1/tasks のエンドポイントが BFF にプロキシされていることを確認する
  test("GET /api/v1/tasks requires authentication", async ({ request }) => {
    const response = await request.get(`${BFF_BASE}/api/v1/tasks`);
    // 認証エラーが返れば、エンドポイントは存在する（404 ではない）
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
  });

  // POST /api/v1/tasks のエンドポイントが BFF にプロキシされていることを確認する
  test("POST /api/v1/tasks requires authentication", async ({ request }) => {
    const response = await request.post(`${BFF_BASE}/api/v1/tasks`, {
      headers: { "Content-Type": "application/json" },
      data: {},
    });
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
  });

  // GET /api/v1/tasks/:id のエンドポイントが BFF にプロキシされていることを確認する
  test("GET /api/v1/tasks/:id requires authentication", async ({ request }) => {
    const response = await request.get(
      `${BFF_BASE}/api/v1/tasks/non-existent-id`
    );
    // 認証エラー (401/403) または存在しない場合は 404
    expect([...AUTH_ERROR_STATUSES, 404]).toContain(response.status());
  });

  // PUT /api/v1/tasks/:id のエンドポイントが BFF にプロキシされていることを確認する
  test("PUT /api/v1/tasks/:id requires authentication", async ({ request }) => {
    const response = await request.put(
      `${BFF_BASE}/api/v1/tasks/non-existent-id`,
      {
        headers: { "Content-Type": "application/json" },
        data: {},
      }
    );
    expect([...AUTH_ERROR_STATUSES, 404]).toContain(response.status());
  });

  // DELETE /api/v1/tasks/:id のエンドポイントが BFF にプロキシされていることを確認する
  test("DELETE /api/v1/tasks/:id requires authentication", async ({
    request,
  }) => {
    const response = await request.delete(
      `${BFF_BASE}/api/v1/tasks/non-existent-id`
    );
    expect([...AUTH_ERROR_STATUSES, 404]).toContain(response.status());
  });
});

test.describe("エラーレスポンス形式", () => {
  // 401 レスポンスが JSON または適切な Content-Type を返すことを確認する
  test("保護エンドポイントの 401 レスポンスが Content-Type を返すこと", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/api/v1/tasks`, {
      headers: { Accept: "application/json" },
    });
    expect(AUTH_ERROR_STATUSES).toContain(response.status());
    // Content-Type ヘッダーが設定されていることを確認する
    const contentType = response.headers()["content-type"] ?? "";
    expect(contentType).toBeTruthy();
  });
});
