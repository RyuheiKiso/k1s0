/**
 * k1s0 tier2 cross-tenant 統合テスト TypeScript 版
 * Rust 実装（cross_tenant_test.rs）と意味的に等価な TypeScript 版
 * P1-P4 invariant の cross-tenant rejection を検証する
 *
 * 実行方法: TEST_DATABASE_URL 環境変数を設定した上で実行する
 *   TEST_DATABASE_URL=postgres://k1s0:k1s0dev@localhost/tier2_dev pnpm test
 *
 * 通常の CI（unit test job）では TEST_DATABASE_URL が未設定のため全テストを skip する
 */

// TenantContext を import する（テナントコンテキスト）
import { TenantContext } from "./tenantContext.js";
// AtomicTripleWrite と関連型を import する（P1-P4 atomic 書込エンジン）
import { AtomicTripleWrite, TenantIdMismatchError } from "./atomicTripleWrite.js";

// TEST_DATABASE_URL が設定されているかを確認するヘルパー関数
// 未設定の場合は false を返す
function hasDatabaseUrl(): boolean {
  // process.env.TEST_DATABASE_URL が設定されているかを確認する
  return typeof process !== "undefined" && !!process.env["TEST_DATABASE_URL"];
}

// cross-tenant 統合テストスイート
describe("tier2 cross-tenant integration tests", () => {
  // P3: cross-tenant isolation の検証
  // 異なる tenant_id の StateChange が reject されることを確認する
  test("P3: cross-tenant write must be rejected before reaching DB", () => {
    // TEST_DATABASE_URL が未設定の場合はスキップする
    if (!hasDatabaseUrl()) {
      // 環境変数が未設定の場合はスキップメッセージを出してテストをスキップする
      console.log(
        "TEST_DATABASE_URL が未設定のため cross-tenant 統合テストをスキップする",
      );
      return;
    }

    // テナント A の TenantContext を生成する
    const tenantAId = "550e8400-e29b-41d4-a716-446655440001";
    const ctxA = TenantContext.fromAuth(tenantAId, "actor-a", "business_op");
    // テナント B の tenant_id を生成する（cross-tenant を模擬する）
    const tenantBId = "550e8400-e29b-41d4-a716-446655440002";

    // AtomicTripleWrite をテナント A のコンテキストで生成する
    const writer = new AtomicTripleWrite(ctxA);
    // テナント B の StateChange を生成する（P3 違反を意図的に作る）
    const crossTenantChange = {
      // テナント A のコンテキストに テナント B の change を渡す（P3 違反）
      aggregateId: "aggregate-001",
      tenantId: tenantBId,
      tableClass: "TenantScoped" as const,
      payload: JSON.stringify({ cross_tenant: "attempt" }),
      version: 1,
    };

    // P3 検証: cross-tenant StateChange が即座に reject されることを確認する
    // verifyTenantId が TenantIdMismatchError をスローすることを期待する
    expect(() => writer.verifyTenantId(crossTenantChange)).toThrow(
      TenantIdMismatchError,
    );
  });

  // P1: 同一テナントの atomic triple write が成功することを確認する（SQL 生成レベル）
  // TEST_DATABASE_URL が設定されていない場合でも SQL 生成は検証できる
  test("P1: buildTripleWriteSql generates SQL with all three tables", () => {
    // テスト用テナントの TenantContext を生成する
    const tenantId = "550e8400-e29b-41d4-a716-446655440010";
    const ctx = TenantContext.fromAuth(tenantId, "test-actor", "business_op");
    // AtomicTripleWrite を生成する
    const writer = new AtomicTripleWrite(ctx);
    // テスト用の StateChange を生成する
    const change = {
      // テスト用の aggregate ID を設定する
      aggregateId: "agg-001",
      // TenantContext と同一の tenant_id を設定する（P3 整合性）
      tenantId,
      // TenantScoped テーブルクラスを指定する
      tableClass: "TenantScoped" as const,
      // テスト用ペイロードを設定する
      payload: JSON.stringify({ test: "atomic_triple_write" }),
      // バージョン 1 で開始する
      version: 1,
    };

    // SQL を生成する
    const sql = writer.buildTripleWriteSql(change);
    // BEGIN と COMMIT の間に 3 つの INSERT が含まれることを確認する
    expect(sql).toContain("BEGIN");
    expect(sql).toContain("domain_event");
    // outbox_message テーブル名が SQL に含まれることを確認する（migration SoT: k1s0.outbox_message）
    expect(sql).toContain("outbox_message");
    expect(sql).toContain("audit_event");
    expect(sql).toContain("COMMIT");
    // GUC 注入が含まれることを確認する
    expect(sql).toContain("app.tenant_id");
  });

  // P4: pii_segregated テーブルへのアクセスが audit 必須であることを確認する
  test("P4: pii_segregated access requires audit", () => {
    // テスト用テナントの TenantContext を生成する（Support purpose を使う）
    const tenantId = "550e8400-e29b-41d4-a716-446655440020";
    const ctx = TenantContext.fromAuth(tenantId, "support-engineer-001", "support");
    // AtomicTripleWrite を生成する
    const writer = new AtomicTripleWrite(ctx);
    // P4: PiiSegregated テーブルクラスの StateChange を生成する
    const piiChange = {
      // テスト用の aggregate ID を設定する
      aggregateId: "agg-pii-001",
      // TenantContext と同一の tenant_id を設定する
      tenantId,
      // PiiSegregated を指定することで P4 audit 必須フラグが true になる
      tableClass: "PiiSegregated" as const,
      // PII フィールドは redact 済みのみ含む
      payload: JSON.stringify({ pii_field: "[REDACTED]" }),
      // バージョン 1 で開始する
      version: 1,
    };

    // P4: pii_required フラグが true を返すことを確認する
    const piiRequired = writer.verifyPiiAuditRequired(piiChange);
    // PiiSegregated では audit が必須であることを確認する
    expect(piiRequired).toBe(true);
  });
});
