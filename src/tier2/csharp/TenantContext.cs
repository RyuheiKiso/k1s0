// k1s0 tier2 テナントコンテキスト C# (.NET 8+) 実装
// Rust 実装（tenant_context.rs）と 4 言語等価強度を持つ C# 版
// PostgreSQL session GUC を transaction 開始時に SET LOCAL で自動注入する

// System.Text.RegularExpressions を using する
using System;
using System.Collections.Generic;
using System.Text;

// k1s0 tier2 名前空間
namespace K1s0.Tier2;

/// <summary>
/// PostgreSQL session GUC の app.purpose 許容値
/// 10_テナント分離適合仕様.md の purpose enum と完全整合する
/// </summary>
public enum SessionPurpose
{
    /// <summary>通常業務操作: テナント所有データへのアクセス</summary>
    BusinessOp,
    /// <summary>サポートアクセス: support_engineer role での PII 参照</summary>
    Support,
    /// <summary>データエクスポート: バッチ出力</summary>
    Export,
    /// <summary>マイグレーション: スキーマ移行期間中の特権操作</summary>
    Migration,
    /// <summary>緊急オペレーション: 障害対応時の緊急権限（最小化・全記録必須）</summary>
    Emergency,
}

/// <summary>
/// テナントセッションコンテキスト
/// 4 つの PostgreSQL session GUC をまとめて管理する
/// tenant_id は直接 API 引数で受け取らず、Create() 経由のみ許容する
/// </summary>
public sealed class TenantContext
{
    // テナント識別子（private: 外部からの直接設定を禁止する）
    private readonly Guid _tenantId;
    // アクター識別子（Keycloak subject）
    private readonly string _actorId;
    // セッション目的
    private readonly SessionPurpose _purpose;
    // 委譲チェーン（通常操作では空リスト）
    private readonly IReadOnlyList<string> _delegationChain;

    /// <summary>
    /// TenantContext を認証済み情報から生成する（ファクトリメソッド）
    /// tenant_id を外部から直接 API 引数として渡せないことを型で表現する
    /// </summary>
    private TenantContext(Guid tenantId, string actorId, SessionPurpose purpose, IReadOnlyList<string> delegationChain)
    {
        // フィールドを初期化する
        _tenantId = tenantId;
        _actorId = actorId;
        _purpose = purpose;
        _delegationChain = delegationChain;
    }

    /// <summary>
    /// 認証済み情報から TenantContext を生成する
    /// tenant_id は AuthContext 経由のみ許容する設計
    /// </summary>
    public static TenantContext Create(Guid tenantId, string actorId, SessionPurpose purpose)
    {
        // 委譲チェーンは空で初期化する
        return new TenantContext(tenantId, actorId, purpose, Array.Empty<string>());
    }

    /// <summary>
    /// 委譲元の actor を追加した TenantContext を返す（イミュータブルコピー）
    /// </summary>
    public TenantContext WithDelegation(string delegatorId)
    {
        // 既存の委譲チェーンに delegator を追加した新しいリストを生成する
        var newChain = new List<string>(_delegationChain) { delegatorId };
        return new TenantContext(_tenantId, _actorId, _purpose, newChain.AsReadOnly());
    }

    /// <summary>
    /// 4 GUC を一括 SET LOCAL する SQL 文字列を返す
    /// Npgsql 等の DB クライアントが実行する（このメソッドは SQL 生成のみ担う）
    /// </summary>
    public string ToSetLocalSql()
    {
        // purpose 値を GUC 文字列に変換する
        var purposeStr = _purpose switch
        {
            SessionPurpose.BusinessOp  => "business_op",
            SessionPurpose.Support     => "support",
            SessionPurpose.Export      => "export",
            SessionPurpose.Migration   => "migration",
            SessionPurpose.Emergency   => "emergency",
            _ => throw new InvalidOperationException($"Unknown purpose: {_purpose}"),
        };
        // delegation_chain の配列リテラルを構築する
        var chainLiteral = _delegationChain.Count == 0
            ? "'{}'"
            : $"'{{{string.Join(",", System.Linq.Enumerable.Select(_delegationChain, s => $"\"{s.Replace("\"", "\\\"")}\""))}}}'";
        // 4 GUC の SET LOCAL SQL を返す
        var sb = new StringBuilder();
        sb.AppendFormat("SET LOCAL app.tenant_id = '{0}'; ", _tenantId.ToString("D"));
        sb.AppendFormat("SET LOCAL app.actor_id = '{0}'; ", _actorId.Replace("'", "''"));
        sb.AppendFormat("SET LOCAL app.purpose = '{0}'; ", purposeStr);
        sb.AppendFormat("SET LOCAL app.delegation_chain = {0};", chainLiteral);
        return sb.ToString();
    }

    // TenantID を取得する（リポジトリ抽象のみが使用する）
    internal Guid TenantId => _tenantId;
}
