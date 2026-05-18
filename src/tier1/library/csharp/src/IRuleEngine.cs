// IRuleEngine.cs — k1s0 tier1 Library C# 実装: Rule Engine の L1+ interface
// 17_ルールエンジン適合仕様.md §IRuleEngineClient（OPA / Drools L1+ 深耕）に準拠する。
// OPA（Open Policy Agent）の full API を Library 独自語彙で表現しつつ、AuthContext 伝播を強制する。
// OSS 型（OPA.NET 等）を公開シグネチャに一切含まない。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// PolicyResult はポリシー評価結果を宣言する型。
/// OPA の decision result を Library 独自語彙で表現する。
/// </summary>
// PolicyResult クラス定義
public sealed class PolicyResult
{
    /// <summary>Allowed: 評価結果（true = 許可 / false = 拒否）</summary>
    // Allowed プロパティ（必須）
    public required bool Allowed { get; init; }

    /// <summary>Reason: 判断理由（"insufficient_scope" / "tenant_mismatch" 等）</summary>
    // Reason プロパティ
    public string? Reason { get; init; }

    /// <summary>Details: 追加詳細情報（監査ログ / デバッグ用）</summary>
    // Details プロパティ
    public IReadOnlyDictionary<string, object?>? Details { get; init; }
}

/// <summary>
/// PolicyBundle はポリシーのバンドル情報を宣言する型。
/// OPA の Bundle（複数ポリシーファイルのアーカイブ）を Library 独自語彙で表現する。
/// </summary>
// PolicyBundle レコード定義
public sealed record PolicyBundle(
    // BundlePath: バンドルの取得元 URL または Object Storage パス
    string BundlePath,
    // Revision: バンドルのリビジョン（git commit hash 等）
    string Revision
);

/// <summary>
/// PolicyValidateOptions はポリシー検証オプションを宣言する型。
/// </summary>
// PolicyValidateOptions クラス定義
public sealed class PolicyValidateOptions
{
    /// <summary>StrictMode: 厳格モード（未定義変数参照等をエラーとして扱う）</summary>
    // StrictMode プロパティ
    public bool StrictMode { get; init; }

    /// <summary>Capabilities: 使用可能な OPA 組み込み関数の制限（null = 全組み込みを許可する）</summary>
    // Capabilities プロパティ
    public IReadOnlyList<string>? Capabilities { get; init; }
}

/// <summary>
/// IRuleEngineClient は Rule Engine の L1+ 抽象 interface を宣言する。
/// OPA（Open Policy Agent）の full API を Library 独自語彙で表現する。
/// OSS 型を引数・戻り値に一切含まない。
/// AuthContext 伝播を強制する（tenant 分離 + 監査に必須）。
/// </summary>
// IRuleEngineClient インターフェース定義
public interface IRuleEngineClient
{
    /// <summary>
    /// EvaluateAsync はポリシーを評価して結果を返す（Unary 評価）。
    /// path は評価するポリシーパス（"data.k1s0.authz.allow" 等）。
    /// input はポリシーへの入力データ（AuthContext のフィールドは実装が自動注入する）。
    /// </summary>
    // EvaluateAsync メソッド: ポリシーを評価する
    Task<PolicyResult> EvaluateAsync(
        string path,
        IReadOnlyDictionary<string, object?> input,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// EvaluateToAnyAsync はポリシーを評価して任意型の結果を返す（ルール結果が bool 以外の場合）。
    /// </summary>
    // EvaluateToAnyAsync メソッド: 任意型の結果を返す評価
    Task<object?> EvaluateToAnyAsync(
        string path,
        IReadOnlyDictionary<string, object?> input,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// BatchEvaluateAsync は複数のポリシーパスを一括評価する。
    /// paths は評価するポリシーパスのリスト、input は全パスに共通の入力。
    /// </summary>
    // BatchEvaluateAsync メソッド: 複数ポリシーを一括評価する
    Task<IReadOnlyList<PolicyResult>> BatchEvaluateAsync(
        IReadOnlyList<string> paths,
        IReadOnlyDictionary<string, object?> input,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// LoadBundleAsync は OPA ポリシーバンドルをロードする（hot reload 対応）。
    /// </summary>
    // LoadBundleAsync メソッド: ポリシーバンドルをロードする
    Task LoadBundleAsync(PolicyBundle bundle, CancellationToken cancellationToken = default);

    /// <summary>
    /// ValidatePolicyAsync はポリシー文字列の構文 / 意味論的正当性を検証する。
    /// policySource は Rego ポリシーソースコード文字列。
    /// </summary>
    // ValidatePolicyAsync メソッド: ポリシーの正当性を検証する
    Task ValidatePolicyAsync(string policySource, PolicyValidateOptions? opts = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// ListPoliciesAsync は現在ロードされているポリシーのパス一覧を返す。
    /// </summary>
    // ListPoliciesAsync メソッド: ポリシーパス一覧を取得する
    Task<IReadOnlyList<string>> ListPoliciesAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// ICachedRuleEngineClient はポリシー評価結果をキャッシュする L1+ 拡張 interface を宣言する。
/// wall-clock TTL 禁止規約に準拠して HLC ベースの TTL のみを受け付ける。
/// </summary>
// ICachedRuleEngineClient インターフェース定義
public interface ICachedRuleEngineClient : IRuleEngineClient
{
    /// <summary>
    /// EvaluateCachedAsync はキャッシュ付きでポリシーを評価する。
    /// cacheKey はキャッシュキー（tenant_id + policy_path + input hash で構成を推奨する）。
    /// ttl は HLC ベースのキャッシュ有効期限（null = キャッシュしない）。
    /// </summary>
    // EvaluateCachedAsync メソッド: キャッシュ付きでポリシーを評価する
    Task<PolicyResult> EvaluateCachedAsync(
        string path,
        IReadOnlyDictionary<string, object?> input,
        string cacheKey,
        CacheTtl? ttl = null,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// InvalidateCacheAsync は指定キーのキャッシュを無効化する（ポリシー更新時に呼び出す）。
    /// </summary>
    // InvalidateCacheAsync メソッド: 指定キーのキャッシュを無効化する
    Task InvalidateCacheAsync(string cacheKey, CancellationToken cancellationToken = default);

    /// <summary>
    /// InvalidateAllCacheAsync は全キャッシュを無効化する（バンドル更新時に使用する）。
    /// </summary>
    // InvalidateAllCacheAsync メソッド: 全キャッシュを無効化する
    Task InvalidateAllCacheAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// PolicyAuditEntry はポリシー評価の監査ログエントリを宣言する型。
/// OPA の decision log を Library 独自語彙で表現する。
/// </summary>
// PolicyAuditEntry クラス定義
public sealed class PolicyAuditEntry
{
    /// <summary>DecisionId: 評価の識別子（UUID v7 形式を推奨する）</summary>
    // DecisionId プロパティ（必須）
    public required string DecisionId { get; init; }

    /// <summary>Path: 評価したポリシーパス</summary>
    // Path プロパティ（必須）
    public required string Path { get; init; }

    /// <summary>Input: 評価に使用した入力データ（PII を除外した safe 版）</summary>
    // Input プロパティ（必須）
    public required IReadOnlyDictionary<string, object?> Input { get; init; }

    /// <summary>Result: 評価結果</summary>
    // Result プロパティ（必須）
    public required PolicyResult Result { get; init; }

    /// <summary>TenantId: 評価を行ったテナント識別子</summary>
    // TenantId プロパティ（必須）
    public required string TenantId { get; init; }

    /// <summary>TimestampTick: 評価時刻（HLC tick 値: wall-clock TTL 禁止規約に準拠する）</summary>
    // TimestampTick プロパティ
    public ulong TimestampTick { get; init; }
}
