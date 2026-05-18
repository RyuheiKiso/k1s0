// k1s0 tier2 admin C# (.NET 8+) インターフェース定義
// Rust 実装（admin/src/admin_boundary.rs, admin/src/admin_operation.rs）と 4 言語等価強度を持つ C# 版
// すべての管理操作は IAdminBoundaryGuard を通過してから実行する（設計方針 14）

// System.Guid などの基本型
using System;
// System.Threading.Tasks: 非同期処理に使用する
using System.Threading.Tasks;

// k1s0 tier2 admin 名前空間
namespace K1s0.Tier2.Admin;

/// <summary>
/// AdminOperation: 管理境界を通過できる操作の識別子（業界中立語のみ）
/// Rust の AdminOperation enum に対応する
/// </summary>
public enum AdminOperationKind
{
    /// <summary>テナントプロビジョニング（新規テナントのリソース割り当て）</summary>
    TenantProvision,
    /// <summary>テナント一時停止（すべての API アクセスを拒否状態にする）</summary>
    TenantSuspend,
    /// <summary>クォータ上限の一時的な上書き（緊急措置）</summary>
    QuotaOverride,
    /// <summary>ユーザーへの権限付与（デュアル承認を強制する）</summary>
    UserGrant,
    /// <summary>緊急アクセス（インシデント対応時のみ / デュアル承認必須）</summary>
    EmergencyAccess,
}

/// <summary>
/// AdminRequest: 管理操作リクエストのエンベロープ型
/// Rust の AdminRequest 構造体に対応する
/// </summary>
public sealed record AdminRequest
{
    // リクエスト一意識別子（UUID v4 / 監査ログの相関 ID に使用する）
    public required Guid RequestId { get; init; }
    // 呼び出し元識別子（ユーザー ID またはサービスアカウント ID）
    public required Guid CallerId { get; init; }
    // 実行しようとしている管理操作種別
    public required AdminOperationKind OperationKind { get; init; }
    // 操作の正当化理由（監査ログに記録される）
    public required string Justification { get; init; }
    // 操作固有パラメータ（JSON シリアライズ形式で渡す）
    public string? OperationPayloadJson { get; init; }
}

/// <summary>
/// AdminResponse: 管理操作レスポンスのエンベロープ型
/// Rust の AdminResponse 構造体に対応する
/// </summary>
public sealed record AdminResponse
{
    // 対応するリクエスト識別子（相関追跡に使用する）
    public required Guid RequestId { get; init; }
    // 操作が成功したか
    public required bool Success { get; init; }
    // 操作結果メッセージ（成功時は完了詳細 / 失敗時はエラー詳細）
    public required string Message { get; init; }
    // 操作が監査ログに記録されたか（必ず true でなければならない）
    public required bool AuditRecorded { get; init; }
}

/// <summary>
/// AdminBoundaryException: 管理境界違反例外
/// Rust の AdminBoundaryError に対応する
/// </summary>
public sealed class AdminBoundaryException : Exception
{
    // 違反種別（スコープ不足 / デュアル承認未完了 / テナントアクセス拒否）
    public AdminBoundaryErrorKind ErrorKind { get; }

    // AdminBoundaryException を生成する
    public AdminBoundaryException(AdminBoundaryErrorKind kind, string message)
        // 基底クラスのコンストラクタにメッセージを渡す
        : base(message)
    {
        // 違反種別を格納する
        ErrorKind = kind;
    }
}

/// <summary>
/// AdminBoundaryErrorKind: 管理境界違反の種別
/// Rust の AdminBoundaryError バリアントに対応する
/// </summary>
public enum AdminBoundaryErrorKind
{
    /// <summary>呼び出し元トークンが管理スコープを持っていない</summary>
    InsufficientScope,
    /// <summary>デュアル承認が必要な操作で承認が足りない</summary>
    DualApprovalRequired,
    /// <summary>操作対象テナントへのアクセス権がない</summary>
    TenantAccessDenied,
}

/// <summary>
/// IAdminBoundaryGuard: 管理操作の境界チェックを実装する C# インターフェース
/// Rust の AdminBoundaryGuard トレイトに対応する（設計方針 14）
/// Keycloak + Kyverno ポリシーと連動してスコープ検証を行う
/// tenant_id は AuthContext から取得するため API 引数で受け取らない
/// </summary>
public interface IAdminBoundaryGuard
{
    /// <summary>
    /// 管理操作の実行スコープを検証する
    /// token: Bearer トークン（Keycloak JWT）
    /// operationKind: 実行しようとしている管理操作種別
    /// スコープ不足またはデュアル承認未完了の場合は AdminBoundaryException をスローする
    /// </summary>
    // CheckAdminScope メソッド（スコープ検証操作）
    Task CheckAdminScopeAsync(string token, AdminOperationKind operationKind);

    /// <summary>
    /// 管理リクエストを処理する（スコープ検証後に呼び出す）
    /// 操作の実行結果を AdminResponse として返す
    /// tenant_id は AuthContext から取得するため引数で受け取らない
    /// </summary>
    // ProcessRequest メソッド（管理操作実行）
    Task<AdminResponse> ProcessRequestAsync(AdminRequest request);

    /// <summary>
    /// この操作がデュアル承認を必要とするか返す
    /// Rust の AdminOperation.requires_dual_approval() に対応する
    /// </summary>
    // RequiresDualApproval メソッド（デュアル承認チェック）
    bool RequiresDualApproval(AdminOperationKind operationKind);
}
