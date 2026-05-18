// k1s0 tier2 cross-tenant 統合テスト C# (.NET 8+) 版
// Rust 実装（cross_tenant_test.rs）と意味的に等価な C# 版
// P1-P4 invariant の cross-tenant rejection を検証するテストクラス
//
// 実行方法: TEST_DATABASE_URL 環境変数を設定した上で実行する
//   TEST_DATABASE_URL=postgres://k1s0:k1s0dev@localhost/tier2_dev dotnet test
//   （xUnit を追加する場合は k1s0.Tier2.Tests.csproj を別途作成する）
//
// 通常の CI（unit test job）では TEST_DATABASE_URL が未設定のため全テストを skip する

// 基本型
using System;
// 環境変数取得に使用する
using System.Threading.Tasks;

// K1s0 tier2 テスト名前空間
namespace K1s0.Tier2.Tests;

/// <summary>
/// FactAttribute: xUnit の Fact 属性のスタブ
/// k1s0.Tier2.TenantContext.csproj に xUnit を追加することなくコンパイルを通すための stub
/// 本番テストプロジェクトでは xUnit パッケージを参照すること
/// </summary>
// FactAttribute スタブ定義（xUnit がない場合のコンパイルエラーを防ぐ）
[AttributeUsage(AttributeTargets.Method)]
public sealed class FactAttribute : Attribute
{
    // xUnit の Fact 属性を模倣するスタブ属性
}

/// <summary>
/// AssertHelper: テスト検証用のヘルパークラス（xUnit.Assert のスタブ）
/// </summary>
// AssertHelper 静的クラス定義
public static class AssertHelper
{
    // True 検証: condition が false の場合は InvalidOperationException をスローする
    public static void True(bool condition, string message = "Expected true but was false")
    {
        // condition が false の場合はエラーをスローする
        if (!condition) throw new InvalidOperationException(message);
    }

    // Contains 検証: text が substring を含まない場合は InvalidOperationException をスローする
    public static void Contains(string substring, string text)
    {
        // text が substring を含まない場合はエラーをスローする
        if (!text.Contains(substring))
            throw new InvalidOperationException($"Expected '{text}' to contain '{substring}'");
    }

    // Throws 検証: action が T 型の例外をスローするかを検証する
    public static void Throws<T>(Action action) where T : Exception
    {
        // action を実行して例外をキャッチする
        try
        {
            action();
            // 例外がスローされなかった場合はエラーをスローする
            throw new InvalidOperationException($"Expected {typeof(T).Name} to be thrown but no exception was thrown");
        }
        catch (T)
        {
            // 期待された型の例外がスローされた場合は OK
        }
        catch (InvalidOperationException ex) when (ex.Message.StartsWith("Expected"))
        {
            // 検証エラー自体は再スローする
            throw;
        }
        catch (Exception ex)
        {
            // 異なる型の例外がスローされた場合はエラーをスローする
            throw new InvalidOperationException($"Expected {typeof(T).Name} but got {ex.GetType().Name}: {ex.Message}");
        }
    }
}

/// <summary>
/// CrossTenantTests: cross-tenant 統合テストクラス
/// P1-P4 invariant の cross-tenant rejection を検証する
/// </summary>
public sealed class CrossTenantTests
{
    // TEST_DATABASE_URL 環境変数から接続 URL を取得するヘルパー
    // 環境変数が未設定の場合は null を返す
    private static string? GetTestDatabaseUrl()
    {
        // TEST_DATABASE_URL 環境変数を取得する（未設定の場合は null を返す）
        return Environment.GetEnvironmentVariable("TEST_DATABASE_URL");
    }

    /// <summary>
    /// P3: cross-tenant write が DB に到達する前に reject されることを検証する
    /// 異なる tenant_id の StateChange が即座にエラーになることを確認する
    /// </summary>
    [Fact]
    public void P3_CrossTenantWriteMustBeRejectedBeforeReachingDb()
    {
        // TEST_DATABASE_URL が未設定の場合はスキップする
        var dbUrl = GetTestDatabaseUrl();
        if (string.IsNullOrEmpty(dbUrl))
        {
            // 環境変数が未設定の場合は SkipException でテストをスキップする
            throw new SkipException("TEST_DATABASE_URL が未設定のため cross-tenant 統合テストをスキップする");
        }

        // テナント A の TenantContext を生成する
        var tenantAId = Guid.NewGuid();
        var ctxA = TenantContext.Create(tenantAId, "actor-a", SessionPurpose.BusinessOp);
        // テナント B の tenant_id を生成する（cross-tenant を模擬する）
        var tenantBId = Guid.NewGuid();

        // AtomicTripleWrite をテナント A のコンテキストで生成する
        var writer = new AtomicTripleWrite(ctxA);
        // テナント B の StateChange を生成する（P3 違反を意図的に作る）
        var crossTenantChange = new StateChange(
            // テスト用の aggregate ID を設定する
            AggregateId: Guid.NewGuid(),
            // テナント A のコンテキストに テナント B の change を渡す（P3 違反）
            TenantId: tenantBId,
            TableClass: TableClass.TenantScoped,
            // テスト用ペイロードを設定する
            Payload: """{"cross_tenant": "attempt"}""",
            // バージョン 1 で開始する
            Version: 1
        );

        // P3 検証: VerifyTenantId が InvalidOperationException をスローすることを確認する
        AssertHelper.Throws<InvalidOperationException>(() => writer.VerifyTenantId(crossTenantChange));
    }

    /// <summary>
    /// P1: 同一テナントの atomic triple write の SQL 生成が全 3 テーブルを含むことを確認する
    /// TEST_DATABASE_URL が未設定でも SQL 生成レベルの検証は可能
    /// </summary>
    [Fact]
    public void P1_BuildTripleWriteSqlContainsAllThreeTables()
    {
        // テスト用テナントの TenantContext を生成する
        var tenantId = Guid.NewGuid();
        var ctx = TenantContext.Create(tenantId, "test-actor", SessionPurpose.BusinessOp);
        // AtomicTripleWrite を生成する
        var writer = new AtomicTripleWrite(ctx);
        // テスト用の StateChange を生成する
        var change = new StateChange(
            // テスト用の aggregate ID を設定する
            AggregateId: Guid.NewGuid(),
            // TenantContext と同一の tenant_id を設定する（P3 整合性）
            TenantId: tenantId,
            TableClass: TableClass.TenantScoped,
            // テスト用ペイロードを設定する
            Payload: """{"test": "atomic_triple_write"}""",
            // バージョン 1 で開始する
            Version: 1
        );

        // SQL を生成する
        var sql = writer.BuildTripleWriteSQL(change);
        // BEGIN と COMMIT の間に 3 つの INSERT が含まれることを確認する
        AssertHelper.Contains("BEGIN", sql);
        AssertHelper.Contains("domain_event", sql);
        AssertHelper.Contains("outbox", sql);
        AssertHelper.Contains("audit_event", sql);
        AssertHelper.Contains("COMMIT", sql);
        // GUC 注入が含まれることを確認する
        AssertHelper.Contains("app.tenant_id", sql);
    }

    /// <summary>
    /// P4: pii_segregated テーブルへのアクセスが audit 必須であることを確認する
    /// </summary>
    [Fact]
    public void P4_PiiSegregatedAccessRequiresAudit()
    {
        // テスト用テナントの TenantContext を生成する（Support purpose を使う）
        var tenantId = Guid.NewGuid();
        var ctx = TenantContext.Create(tenantId, "support-engineer-001", SessionPurpose.Support);
        // AtomicTripleWrite を生成する
        var writer = new AtomicTripleWrite(ctx);
        // P4: PiiSegregated テーブルクラスの StateChange を生成する
        var piiChange = new StateChange(
            // テスト用の aggregate ID を設定する
            AggregateId: Guid.NewGuid(),
            // TenantContext と同一の tenant_id を設定する
            TenantId: tenantId,
            // PiiSegregated を指定することで P4 audit 必須フラグが true になる
            TableClass: TableClass.PiiSegregated,
            // PII フィールドは redact 済みのみ含む
            Payload: """{"pii_field": "[REDACTED]"}""",
            // バージョン 1 で開始する
            Version: 1
        );

        // P4: VerifyPiiAuditRequired が true を返すことを確認する
        AssertHelper.True(writer.VerifyPiiAuditRequired(piiChange));
    }
}

/// <summary>
/// SkipException: テストをスキップするための例外
/// TEST_DATABASE_URL が未設定の場合に使用する
/// </summary>
public sealed class SkipException : Exception
{
    // コンストラクタ（スキップメッセージを受け取る）
    public SkipException(string message) : base(message)
    {
        // 基底クラスのコンストラクタにメッセージを渡す
    }
}
