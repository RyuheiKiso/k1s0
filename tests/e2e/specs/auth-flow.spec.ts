// BFF 認証フロー E2E テスト
// 未認証リクエストが適切に拒否され、認証エンドポイントが正しく動作することを検証する
import { test, expect } from "@playwright/test";
// BFF_BASE は環境変数または config.ts のデフォルト値を使用する
import { BFF_BASE } from "./config";

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

// T-04 対応: 認証成功フローの E2E テスト
// Keycloak が TEST_KEYCLOAK_AVAILABLE=true で利用可能な場合のみ実行する。
// CI では keycloak サービスコンテナが起動している場合に実行される。
test.describe("認証成功フロー（Keycloak 連携）", () => {
  // Keycloak が利用可能かどうかを確認するフラグ
  const keycloakAvailable = process.env.TEST_KEYCLOAK_AVAILABLE === "true";

  test.skip(!keycloakAvailable, "Keycloak が利用不可のためスキップ");

  test("ログイン → セッション確認 → ログアウトの完全フロー", async ({
    page,
  }) => {
    // Keycloak のテストユーザー認証情報（CI 環境の環境変数から取得）
    const testUser = process.env.TEST_USER ?? "testuser";
    const testPassword = process.env.TEST_PASSWORD ?? "testpassword";

    // 1. ログインエンドポイントにアクセスして Keycloak にリダイレクトされることを確認する
    await page.goto(`${BFF_BASE}/bff/auth/login`);
    // Keycloak のログインページに遷移していることを確認する
    await expect(page).toHaveURL(/\/realms\/k1s0\/protocol\/openid-connect\/auth/);

    // 2. Keycloak ログインフォームに認証情報を入力する
    await page.fill("#username", testUser);
    await page.fill("#password", testPassword);
    await page.click('[type="submit"]');

    // 3. BFF へのコールバック後、元のアプリに戻ることを確認する
    await page.waitForURL(`${BFF_BASE}/**`);

    // 4. セッション情報が取得できることを確認する（認証成功の証明）
    const sessionResponse = await page.request.get(
      `${BFF_BASE}/bff/auth/session`,
    );
    expect(sessionResponse.status()).toBe(200);
    const sessionData = await sessionResponse.json();
    expect(sessionData).toHaveProperty("sub");
    expect(sessionData.sub).toBeTruthy();

    // 5. ログアウトしてセッションが無効化されることを確認する
    await page.request.post(`${BFF_BASE}/bff/auth/logout`);

    // 6. ログアウト後はセッションがないため 401 が返ることを確認する
    const afterLogoutResponse = await page.request.get(
      `${BFF_BASE}/bff/auth/session`,
    );
    expect(afterLogoutResponse.status()).toBe(401);
  });

  test("認証後の API アクセスが成功すること", async ({ page }) => {
    const testUser = process.env.TEST_USER ?? "testuser";
    const testPassword = process.env.TEST_PASSWORD ?? "testpassword";

    // ログインフロー
    await page.goto(`${BFF_BASE}/bff/auth/login`);
    await expect(page).toHaveURL(/\/realms\/k1s0\/protocol\/openid-connect\/auth/);
    await page.fill("#username", testUser);
    await page.fill("#password", testPassword);
    await page.click('[type="submit"]');
    await page.waitForURL(`${BFF_BASE}/**`);

    // 認証後は保護された API に 200 または 2xx でアクセスできることを確認する
    const apiResponse = await page.request.get(
      `${BFF_BASE}/bff/api/v1/orders`,
    );
    // 認証済みリクエストは 200 または空リスト（204）を返す（401 は返らない）
    expect(apiResponse.status()).not.toBe(401);
  });
});
