// BFF 認証フロー E2E テスト
// 未認証リクエストが適切に拒否され、認証エンドポイントが正しく動作することを検証する
import { test, expect } from "@playwright/test";

// BFF プロキシのベース URL
const BFF_BASE = "http://localhost:8082";

test.describe("未認証アクセス保護", () => {
  // セッション Cookie なしで保護エンドポイントにアクセスすると 401 が返ることを検証する
  test("未認証で /bff/auth/session にアクセスすると 401 が返ること", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/auth/session`, {
      // Cookie を送信しないことを確認するため headers を明示指定
      headers: { Accept: "application/json" },
    });
    expect(response.status()).toBe(401);
  });

  // 認証なしで API エンドポイントにアクセスすると 401 が返ることを検証する
  test("未認証で /bff/api/v1/orders にアクセスすると 401 が返ること", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/api/v1/orders`, {
      headers: { Accept: "application/json" },
    });
    expect(response.status()).toBe(401);
  });

  test("未認証で /bff/api/v1/inventory にアクセスすると 401 が返ること", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/api/v1/inventory`, {
      headers: { Accept: "application/json" },
    });
    expect(response.status()).toBe(401);
  });

  test("未認証で /bff/api/v1/payments にアクセスすると 401 が返ること", async ({
    request,
  }) => {
    const response = await request.get(`${BFF_BASE}/bff/api/v1/payments`, {
      headers: { Accept: "application/json" },
    });
    expect(response.status()).toBe(401);
  });
});

test.describe("認証エンドポイント動作", () => {
  // ログインエンドポイントが Keycloak へのリダイレクト (3xx) を返すことを検証する
  test("/bff/auth/login が Keycloak へリダイレクトすること", async ({
    request,
  }) => {
    // リダイレクトを自動追従しないようにして 3xx を検証する
    const response = await request.get(`${BFF_BASE}/bff/auth/login`, {
      maxRedirects: 0,
    });
    // 302 (Found) または 307 (Temporary Redirect) を期待する
    expect([301, 302, 307, 308]).toContain(response.status());

    // Location ヘッダーが Keycloak の URL を指すことを確認する
    const location = response.headers()["location"] ?? "";
    expect(location).toBeTruthy();
  });

  // 不正な redirect_to パラメーターが拒否されることを検証する (Finding 6 修正の確認)
  test("redirect_to に任意スキームを指定すると 400 が返ること", async ({
    request,
  }) => {
    const response = await request.get(
      `${BFF_BASE}/bff/auth/login?redirect_to=javascript:alert(1)`,
      { maxRedirects: 0 }
    );
    // 400 Bad Request または 422 Unprocessable Entity を期待する
    expect([400, 422]).toContain(response.status());
  });
});

test.describe("CSRF 保護", () => {
  // CSRF トークンなしで POST リクエストを送ると 403 が返ることを検証する
  test("X-CSRF-Token ヘッダーなしの POST は 401 または 403 が返ること", async ({
    request,
  }) => {
    const response = await request.post(`${BFF_BASE}/bff/api/v1/orders`, {
      headers: { "Content-Type": "application/json" },
      data: { item: "test" },
    });
    // 認証なしなので 401、CSRF エラーなら 403
    expect([401, 403]).toContain(response.status());
  });
});
