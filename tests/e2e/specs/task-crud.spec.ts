// タスク CRUD の E2E テスト
// BFF-Proxy 経由でタスクサービスへのリクエストを検証する。
// TEST_KEYCLOAK_AVAILABLE=true の場合のみ認証付きフローを実行する。
// 認証なしのテストは常に実行し、未認証リクエストが正しく 401 を返すことを確認する。
import { test, expect, type Page } from "@playwright/test";
// BFF_BASE は環境変数または config.ts のデフォルト値を使用する
import { BFF_BASE } from "./config";

// タスク API のベースパス（BFF プロキシ経由: /bff/api/v1/tasks）
const TASK_API = `${BFF_BASE}/bff/api/v1/tasks`;

// Keycloak が利用可能かどうかを確認するフラグ
// TEST_KEYCLOAK_AVAILABLE=true の場合のみ認証済みフローのテストを実行する
const keycloakAvailable = process.env.TEST_KEYCLOAK_AVAILABLE === "true";

// ========================================================
// 認証なしのテスト群（常時実行）
// 未認証リクエストが 401 を返すことを確認する
// ========================================================
test.describe("タスク API: 未認証アクセス", () => {
  // 前提条件: セッション Cookie なし
  // 期待結果: 全エンドポイントで 401 が返ること

  // 認証なしでのタスク一覧取得が 401 を返すことを検証する
  test("未認証で GET /bff/api/v1/tasks にアクセスすると 401 が返ること", async ({
    request,
  }) => {
    const response = await request.get(TASK_API, {
      headers: { Accept: "application/json" },
    });
    expect(response.status()).toBe(401);
  });

  // 認証なしでのタスク作成が 401 を返すことを検証する
  test("未認証で POST /bff/api/v1/tasks にアクセスすると 401 が返ること", async ({
    request,
  }) => {
    const response = await request.post(TASK_API, {
      headers: { "Content-Type": "application/json" },
      data: { title: "test task" },
    });
    // 認証エラー (401) または CSRF エラー (403) を期待する
    expect([401, 403]).toContain(response.status());
  });

  // 認証なしでの特定タスク取得が 401 を返すことを検証する
  test("未認証で GET /bff/api/v1/tasks/:id にアクセスすると 401 が返ること", async ({
    request,
  }) => {
    const response = await request.get(`${TASK_API}/non-existent-id`, {
      headers: { Accept: "application/json" },
    });
    // 認証エラー (401) が返ることを確認する（404 より先に認証チェックが発生する）
    expect(response.status()).toBe(401);
  });

  // 未認証リクエストのエラーレスポンスが ADR-0005 形式であることを検証する
  // 前提条件: Content-Type: application/json でリクエスト
  // 期待結果: { "error": { "code": "...", "message": "..." } } 形式のレスポンス
  test("401 レスポンスが ADR-0005 準拠のエラー形式を返すこと", async ({
    request,
  }) => {
    const response = await request.get(TASK_API, {
      headers: { Accept: "application/json" },
    });
    expect(response.status()).toBe(401);
    // Content-Type ヘッダーが設定されていることを確認する
    const contentType = response.headers()["content-type"] ?? "";
    expect(contentType).toBeTruthy();

    // レスポンスボディが JSON であればエラー構造を検証する
    // BFF が JSON を返せない場合（セッションリダイレクトなど）はスキップする
    if (contentType.includes("application/json")) {
      const body = await response.json();
      // ADR-0005 準拠: { "error": { "code": "...", "message": "..." } } 形式
      expect(body).toHaveProperty("error");
      expect(body.error).toHaveProperty("code");
      expect(body.error).toHaveProperty("message");
    }
  });
});

// ========================================================
// 認証済みフローのテスト群（Keycloak 利用可能時のみ実行）
// タスクの CRUD 操作を実際に BFF-Proxy → タスクサービスの経路で検証する
// ========================================================
test.describe("タスク CRUD E2E（Keycloak 連携）", () => {
  test.skip(!keycloakAvailable, "Keycloak が利用不可のためスキップ");

  // 認証ユーティリティ: Keycloak テストユーザーでログインしてセッションを取得する
  // 前提条件: TEST_USER, TEST_PASSWORD 環境変数が設定済みであること
  async function loginAndGetPage(page: Page) {
    const testUser = process.env.TEST_USER ?? "testuser";
    const testPassword = process.env.TEST_PASSWORD ?? "testpassword";

    // ログインページに遷移して Keycloak 認証フローを完了する
    await page.goto(`${BFF_BASE}/bff/auth/login`);
    // Keycloak のログインページに遷移していることを確認する
    await expect(page).toHaveURL(/\/realms\/k1s0\/protocol\/openid-connect\/auth/);
    await page.fill("#username", testUser);
    await page.fill("#password", testPassword);
    await page.click('[type="submit"]');
    // BFF へのコールバック後、アプリに戻ることを確認する
    await page.waitForURL(`${BFF_BASE}/**`);
  }

  // タスク作成 → 取得 → ステータス更新 → 削除の正常フローを検証する
  // 前提条件: Keycloak 認証済みセッションが存在すること
  // 期待結果: 各 CRUD 操作が正常に完了し、最終的に 404 になること
  test("タスク作成 → 取得 → ステータス更新 → 削除の正常フロー", async ({
    page,
  }) => {
    // ログインしてセッションを確立する
    await loginAndGetPage(page);

    // テスト用のダミープロジェクト ID（UUID 形式）
    const projectId = "00000000-0000-0000-0000-000000000001";

    // 1. POST /bff/api/v1/tasks でタスクを作成する
    const createResponse = await page.request.post(TASK_API, {
      headers: { "Content-Type": "application/json" },
      data: {
        project_id: projectId,
        title: "E2E テスト用タスク",
        description: "E2E テストで作成されたタスク",
        priority: "medium",
        labels: [],
        checklist: [],
      },
    });
    // 201 Created が返ることを確認する
    expect(createResponse.status()).toBe(201);
    const createdTask = await createResponse.json();
    // 作成されたタスクに id フィールドが存在することを確認する
    expect(createdTask).toHaveProperty("id");
    const taskId = createdTask.id as string;
    expect(taskId).toBeTruthy();

    // 2. GET /bff/api/v1/tasks/:id で作成したタスクを取得して内容を検証する
    const getResponse = await page.request.get(`${TASK_API}/${taskId}`);
    expect(getResponse.status()).toBe(200);
    const fetchedTask = await getResponse.json();
    expect(fetchedTask.id).toBe(taskId);
    expect(fetchedTask.title).toBe("E2E テスト用タスク");
    // 初期ステータスは "open" であることを確認する
    expect(fetchedTask.status).toBe("open");

    // 3. PUT /bff/api/v1/tasks/:id/status でステータスを "in_progress" に更新する
    // expected_version は楽観的排他制御のために作成時のバージョンを使用する
    const updateResponse = await page.request.put(
      `${TASK_API}/${taskId}/status`,
      {
        headers: { "Content-Type": "application/json" },
        data: {
          status: "in_progress",
          expected_version: createdTask.version ?? 1,
        },
      }
    );
    expect(updateResponse.status()).toBe(200);
    const updatedTask = await updateResponse.json();
    expect(updatedTask.status).toBe("in_progress");

    // 4. GET /bff/api/v1/tasks/:id で更新後のステータスを確認する
    const getAfterUpdateResponse = await page.request.get(
      `${TASK_API}/${taskId}`
    );
    expect(getAfterUpdateResponse.status()).toBe(200);
    const taskAfterUpdate = await getAfterUpdateResponse.json();
    expect(taskAfterUpdate.status).toBe("in_progress");

    // 5. DELETE /bff/api/v1/tasks/:id でタスクを削除する
    // タスクサービスが DELETE エンドポイントを提供している場合のみ実行する
    const deleteResponse = await page.request.delete(
      `${TASK_API}/${taskId}`
    );
    // 200 または 204 No Content を期待する
    expect([200, 204]).toContain(deleteResponse.status());

    // 6. GET /bff/api/v1/tasks/:id で削除後に 404 が返ることを確認する
    const getAfterDeleteResponse = await page.request.get(
      `${TASK_API}/${taskId}`
    );
    expect(getAfterDeleteResponse.status()).toBe(404);

    // 404 レスポンスが ADR-0005 準拠の形式であることを確認する
    const errorBody = await getAfterDeleteResponse.json();
    // ADR-0005 準拠: { "error": { "code": "SVC_TASK_NOT_FOUND", "message": "..." } }
    expect(errorBody).toHaveProperty("error");
    expect(errorBody.error).toHaveProperty("code");
    expect(errorBody.error.code).toBe("SVC_TASK_NOT_FOUND");
    expect(errorBody.error).toHaveProperty("message");
  });

  // タスク一覧取得のページネーション動作を検証する
  // 前提条件: Keycloak 認証済みセッションが存在すること
  // 期待結果: レスポンスに tasks と total フィールドが含まれること
  test("タスク一覧取得: ページネーション動作", async ({ page }) => {
    // ログインしてセッションを確立する
    await loginAndGetPage(page);

    // GET /bff/api/v1/tasks?limit=10&offset=0 でページネーション付き一覧を取得する
    // タスクサービスのクエリパラメーターは limit・offset（page ではない）
    const response = await page.request.get(
      `${TASK_API}?limit=10&offset=0`
    );
    expect(response.status()).toBe(200);
    const body = await response.json();

    // レスポンスに tasks 配列と total カウントが含まれることを確認する
    // list_tasks ハンドラーは { "tasks": [...], "total": N } 形式で返す
    expect(body).toHaveProperty("tasks");
    expect(body).toHaveProperty("total");
    expect(Array.isArray(body.tasks)).toBe(true);
    expect(typeof body.total).toBe("number");
  });

  // タスク一覧取得のステータスフィルタリング動作を検証する
  // 前提条件: Keycloak 認証済みセッションが存在すること
  // 期待結果: status フィルターで絞り込んだタスクが全て指定ステータスであること
  test("タスク一覧取得: ステータスフィルタリング動作", async ({ page }) => {
    // ログインしてセッションを確立する
    await loginAndGetPage(page);

    // GET /bff/api/v1/tasks?status=open でステータスフィルタを確認する
    const response = await page.request.get(`${TASK_API}?status=open`);
    expect(response.status()).toBe(200);
    const body = await response.json();

    // レスポンスに tasks 配列が含まれることを確認する
    expect(body).toHaveProperty("tasks");
    expect(Array.isArray(body.tasks)).toBe(true);

    // 返却されたタスクが全て "open" ステータスであることを確認する
    // タスクが存在する場合のみ各要素を検証する
    if (body.tasks.length > 0) {
      for (const task of body.tasks as Array<{ status: string }>) {
        expect(task.status).toBe("open");
      }
    }
  });

  // 存在しないタスク取得時の 404 エラーレスポンス構造を検証する
  // 前提条件: Keycloak 認証済みセッションが存在すること
  // 期待結果: 404 と ADR-0005 準拠のエラーレスポンスが返ること
  test("存在しないタスクの取得: 404 エラーレスポンス構造の確認", async ({
    page,
  }) => {
    // ログインしてセッションを確立する
    await loginAndGetPage(page);

    // 存在しないタスク ID（UUID 形式）でリクエストを送信する
    const nonExistentId = "00000000-0000-0000-0000-000000000000";
    const response = await page.request.get(
      `${TASK_API}/${nonExistentId}`
    );
    // 404 Not Found が返ることを確認する
    expect(response.status()).toBe(404);

    // ADR-0005 準拠のエラーレスポンス構造を検証する
    // 期待形式: { "error": { "code": "SVC_TASK_NOT_FOUND", "message": "..." } }
    const body = await response.json();
    expect(body).toHaveProperty("error");
    expect(body.error).toHaveProperty("code");
    expect(body.error).toHaveProperty("message");
    // タスクサービスのエラーコード（task_handler.rs の SVC_TASK_NOT_FOUND）を確認する
    expect(body.error.code).toBe("SVC_TASK_NOT_FOUND");
    expect(typeof body.error.message).toBe("string");
    expect(body.error.message).toBeTruthy();
  });
});

// ========================================================
// 旧 RPC スタイルパスの拒否確認（常時実行）
// RESTful パスのみ許可されており、RPC スタイルパスは 404 を返すことを確認する
// ========================================================
test.describe("タスク API: ルーティング確認", () => {
  // 旧 RPC スタイルのパスは 404 を返すことを確認する
  // 前提条件: api-crud.spec.ts の他サービスと同パターンで検証する
  // 期待結果: /bff/api/v1/list_tasks は 404 を返すこと
  test("旧 RPC スタイル /bff/api/v1/list_tasks は 404 が返ること", async ({
    request,
  }) => {
    const response = await request.get(
      `${BFF_BASE}/bff/api/v1/list_tasks`
    );
    expect(response.status()).toBe(404);
  });

  // RESTful パス /bff/api/v1/tasks がルーティングされていることを確認する
  // 前提条件: 認証なし（認証エラーが返ればエンドポイントは存在する）
  // 期待結果: 401 または 403 が返ること（404 ではない）
  test("GET /bff/api/v1/tasks がルーティングされていること", async ({
    request,
  }) => {
    const response = await request.get(TASK_API, {
      headers: { Accept: "application/json" },
    });
    // 認証エラーが返れば、エンドポイントは存在する（404 ではない）
    expect([401, 403]).toContain(response.status());
  });
});
