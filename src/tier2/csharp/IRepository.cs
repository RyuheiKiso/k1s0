// k1s0 tier2 Repository abstraction C# (.NET 8+) 実装
// Rust 実装（repository.rs）と 4 言語等価強度を持つ C# 版
// 生 SQL 文字列を受け取る public API を持たない設計で tier2 の DB アクセスを封鎖する

// Guid / Exception 等の基本型
using System;
// コレクション型
using System.Collections.Generic;
// 非同期処理に使用する
using System.Threading;
using System.Threading.Tasks;

// k1s0 tier2 名前空間
namespace K1s0.Tier2;

/// <summary>
/// RlsBypassException: RLS bypass 検出例外
/// SELECT 結果の TenantId が TenantContext と一致しない場合にスローする
/// RLS が物理的に保証するが、アプリ層でも二重検証する設計
/// </summary>
public sealed class RlsBypassException : InvalidOperationException
{
    // エンティティの TenantId
    public Guid EntityTenantId { get; }
    // コンテキストの TenantId
    public Guid ContextTenantId { get; }

    /// <summary>
    /// RlsBypassException を生成する
    /// </summary>
    public RlsBypassException(Guid entityTenantId, Guid contextTenantId)
        // エラーメッセージを構築する
        : base($"RLS bypass detected: entity.TenantId={entityTenantId:D}, context.TenantId={contextTenantId:D}")
    {
        // エンティティの TenantId を格納する
        EntityTenantId = entityTenantId;
        // コンテキストの TenantId を格納する
        ContextTenantId = contextTenantId;
    }
}

/// <summary>
/// ITenantScopedEntity: テナントスコープ内のエンティティを表す基底インターフェース
/// PII 列は含まない（pii_segregated table は別の型で管理する）
/// </summary>
public interface ITenantScopedEntity
{
    // エンティティの主キー
    Guid Id { get; }
    // テナント ID（read-only、RLS が保証する）
    Guid TenantId { get; }
    // エンティティバージョン（楽観的ロックに使用する）
    long Version { get; }
}

/// <summary>
/// IRepository&lt;T&gt;: テナントスコープ内のエンティティ操作インターフェース
/// tenant_id は引数で受け取らず、実装内部で TenantContext から注入する
/// </summary>
public interface IRepository<T> where T : ITenantScopedEntity
{
    /// <summary>
    /// エンティティを主キーで取得する
    /// tenant_id は引数で受け取らず、内部的に TenantContext から注入する
    /// 見つからない場合は null を返す
    /// </summary>
    Task<T?> FindByIdAsync(Guid id, CancellationToken cancellationToken = default);

    /// <summary>
    /// テナント ID に紐づく全エンティティを取得する
    /// tenant_id は AuthContext から取得するため引数で受け取らない
    /// </summary>
    Task<IReadOnlyList<T>> FindByTenantIdAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// エンティティを永続化する（INSERT または UPDATE）
    /// tenant_id は TenantContext から注入するため entity に含めない
    /// </summary>
    Task SaveAsync(T entity, CancellationToken cancellationToken = default);

    /// <summary>
    /// エンティティを削除する
    /// tenant_id は TenantContext から自動注入される（API 引数経由禁止）
    /// </summary>
    Task DeleteAsync(Guid id, CancellationToken cancellationToken = default);
}

/// <summary>
/// RepositoryContext: Repository を実行するコンテキスト
/// TenantContext をラップして DB アクセス時の GUC 注入を担う
/// 生 SQL 文字列は受け取らない設計を型で表現する
/// </summary>
public sealed class RepositoryContext
{
    // テナントコンテキスト（GUC 注入に使用する）
    private readonly TenantContext _tenantContext;

    /// <summary>
    /// RepositoryContext を生成する
    /// tenant_id は API 引数として渡せない（TenantContext 経由のみ）
    /// </summary>
    public RepositoryContext(TenantContext tenantContext)
    {
        // TenantContext が null の場合は ArgumentNullException をスローする
        _tenantContext = tenantContext ?? throw new ArgumentNullException(nameof(tenantContext));
    }

    /// <summary>
    /// テナントコンテキストを参照する（読み取り専用）
    /// </summary>
    public TenantContext TenantContext => _tenantContext;

    /// <summary>
    /// SET LOCAL SQL を取得する
    /// Repository 実装が DB に注入するために使用する（4 GUC を一括 SET LOCAL する）
    /// </summary>
    public string SetLocalSql()
    {
        // TenantContext の ToSetLocalSql() を呼んで SQL を返す
        return _tenantContext.ToSetLocalSql();
    }

    /// <summary>
    /// SELECT 結果の TenantId が TenantContext と一致することを検証する
    /// RLS が物理的に保証するが、アプリ層でも二重検証する設計
    /// 一致しない場合は RlsBypassException をスローする
    /// </summary>
    public void VerifySelectResult(ITenantScopedEntity entity)
    {
        // entity が null の場合は ArgumentNullException をスローする
        ArgumentNullException.ThrowIfNull(entity);
        // TenantContext の TenantId を取得する
        var expectedTenantId = _tenantContext.TenantId;
        // エンティティの TenantId と TenantContext の TenantId を比較する
        if (entity.TenantId != expectedTenantId)
        {
            // RLS bypass が発生した場合は即座に RlsBypassException をスローする
            throw new RlsBypassException(entity.TenantId, expectedTenantId);
        }
    }
}
