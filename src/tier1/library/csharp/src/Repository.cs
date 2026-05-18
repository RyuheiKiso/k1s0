// Repository.cs — k1s0 tier1 Library C# 実装: IRepository<T> interface
// 04_認証適合仕様.md §CI 不変条件 整合 5「生 SQL 文字列受付 API 禁止」に準拠する。
// tenant_id を必須引数として受け取る（SQL 文字列受付 API は提供しない）。
// 実装クラスは Npgsql の compile-time 型安全 API のみを使用する。

// System: ArgumentNullException に使用する
using System;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// IRepository&lt;T&gt; は生 SQL 文字列を受け取らない DB アクセス抽象 interface。
/// tenant_id は必須引数として受け取る（RLS と二重で tenant 境界を保証する）。
/// T は DB に永続化されるドメイン型。
/// </summary>
// IRepository インターフェース定義
public interface IRepository<T>
    // T は null 非許容の参照型（nullable enable による型安全性）
    where T : class
{
    /// <summary>
    /// FindByIdAsync は id と tenantId を受け取り、エンティティを返す。
    /// tenantId は必須引数（RLS と二重で tenant 境界を保証する）。
    /// エンティティが見つからない場合は null を返す。
    /// </summary>
    // FindByIdAsync メソッド: id と tenantId でエンティティを取得する
    Task<T?> FindByIdAsync(
        // id: エンティティの主キー（UUID 文字列）
        string id,
        // tenantId: テナント識別子（必須引数）
        string tenantId,
        // cancellationToken: キャンセルトークン
        CancellationToken cancellationToken = default);

    /// <summary>
    /// SaveAsync はエンティティと tenantId を受け取り、DB に保存する。
    /// tenantId は必須引数（RLS と二重で tenant 境界を保証する）。
    /// </summary>
    // SaveAsync メソッド: エンティティを保存する
    Task SaveAsync(
        // entity: 保存するエンティティ
        T entity,
        // tenantId: テナント識別子（必須引数）
        string tenantId,
        // cancellationToken: キャンセルトークン
        CancellationToken cancellationToken = default);
}

/// <summary>
/// RepositoryException は IRepository 操作で発生する例外の基底クラス。
/// RLS bypass 検出時に投げる（アプリ層での二重検証）。
/// </summary>
// RepositoryException クラス定義
public class RepositoryException : Exception
{
    /// <summary>RepositoryException を生成するコンストラクタ。</summary>
    // コンストラクタ: メッセージを受け取る
    public RepositoryException(string message) : base(message)
    {
        // 基底クラスに message を渡す
    }

    /// <summary>RepositoryException を内部例外付きで生成するコンストラクタ。</summary>
    // コンストラクタ: メッセージと内部例外を受け取る
    public RepositoryException(string message, Exception innerException)
        : base(message, innerException)
    {
        // 基底クラスに message と innerException を渡す
    }
}

/// <summary>
/// RlsBypassException は RLS bypass 検出時に投げる例外。
/// RLS が物理的に保証するが、アプリ層でも二重検証する設計。
/// </summary>
// RlsBypassException クラス定義
public sealed class RlsBypassException : RepositoryException
{
    /// <summary>
    /// RlsBypassException を生成するコンストラクタ。
    /// entity.tenantId と context.tenantId の不一致を報告する。
    /// </summary>
    // コンストラクタ: entityTenantId と contextTenantId を受け取る
    public RlsBypassException(string entityTenantId, string contextTenantId)
        : base($"RLS bypass detected: entity.tenant_id={entityTenantId} != context.tenant_id={contextTenantId}")
    {
        // EntityTenantId を設定する
        EntityTenantId = entityTenantId;
        // ContextTenantId を設定する
        ContextTenantId = contextTenantId;
    }

    /// <summary>EntityTenantId: エンティティに記録されたテナント ID</summary>
    // EntityTenantId プロパティ
    public string EntityTenantId { get; }

    /// <summary>ContextTenantId: コンテキストに設定されたテナント ID</summary>
    // ContextTenantId プロパティ
    public string ContextTenantId { get; }
}
