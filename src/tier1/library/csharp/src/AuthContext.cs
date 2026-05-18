// AuthContext.cs — k1s0 tier1 Library C# 実装: AuthClass enum + AuthContext class
// 04_認証適合仕様.md §v1 auth_class セット（5 class）および
// §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
// 生 access_token / refresh_token は公開シグネチャに一切含まれない。

// System: Guid / ArgumentNullException に使用する
using System;
// System.Collections.Generic: IReadOnlyList に使用する
using System.Collections.Generic;
// System.Text: StringBuilder に使用する
using System.Text;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// AuthClass は 04_認証適合仕様.md §v1 auth_class セット（5 class）を宣言する。
/// class 1 値が subject_kind / token_type / lifetime_class / refresh_policy / step_up_required を
/// 一意に導出する（dimension override 禁止）。
/// </summary>
// AuthClass 列挙型: 5 class のいずれかを表す
public enum AuthClass
{
    /// <summary>v1_human_session: 業務担当者 SPA セッション（OIDC + DPoP + rotating refresh）</summary>
    V1HumanSession,
    /// <summary>v1_workload_jwt: K8s ServiceAccount / SPIFFE SVID（短 TTL JWT、自動 renew）</summary>
    V1WorkloadJwt,
    /// <summary>v1_device_attest: 工場端末（device cert、長 TTL、one_shot refresh）</summary>
    V1DeviceAttest,
    /// <summary>v1_federated_exchange: 外部 IdP からの RFC 8693 token exchange</summary>
    V1FederatedExchange,
    /// <summary>v1_emergency_step_up: break-glass（always step_up、TTL&lt;10m、no refresh）</summary>
    V1EmergencyStepUp,
}

/// <summary>
/// AuthContext は tier1 Library が transaction 開始時に PostgreSQL GUC に SET LOCAL する
/// 認証拡張フィールドを保持する opaque 型。
/// 04_認証適合仕様.md §AuthContext スキーマ準拠。
/// 生 access_token / refresh_token はフィールドに含まない。
/// </summary>
// AuthContext クラス定義（sealed: 外部からの継承を禁止する）
public sealed class AuthContext
{
    // AuthClass: v1_human_session 等（5 class のいずれか）
    private readonly AuthClass _authClass;
    // SubjectId: canonical subject（Keycloak sub など）
    private readonly string _subjectId;
    // SubjectKind: human / workload / device / external_subject
    private readonly string _subjectKind;
    // TokenId: JWT jti（revocation tracking 用）
    private readonly string _tokenId;
    // SessionId: human / device のセッション識別子（workload は空文字列）
    private readonly string _sessionId;
    // TenantId: リクエストのテナント識別子
    private readonly string _tenantId;
    // Audience: token aud（v1_federated_exchange で必須）
    private readonly string _audience;
    // Scopes: OAuth scopes（service.api / emergency.break_glass 等）
    private readonly IReadOnlyList<string> _scopes;
    // DpopJkt: DPoP key thumbprint（dpop_bound_jwt のみ設定される）
    private readonly string? _dpopJkt;
    // AttestationLevel: device attestation level（jwt_attested のみ設定される）
    private readonly string? _attestationLevel;
    // StepUpProven: 最終 step_up challenge 済みフラグ
    private readonly bool _stepUpProven;
    // IsValid: token 検証が成功したかどうか
    private readonly bool _isValid;

    // コンストラクタは private（ファクトリメソッド経由のみ生成可能）
    private AuthContext(
        AuthClass authClass,
        string subjectId,
        string subjectKind,
        string tokenId,
        string sessionId,
        string tenantId,
        string audience,
        IReadOnlyList<string> scopes,
        string? dpopJkt,
        string? attestationLevel,
        bool stepUpProven,
        bool isValid)
    {
        // 全フィールドを初期化する
        _authClass = authClass;
        _subjectId = subjectId;
        _subjectKind = subjectKind;
        _tokenId = tokenId;
        _sessionId = sessionId;
        _tenantId = tenantId;
        _audience = audience;
        _scopes = scopes;
        _dpopJkt = dpopJkt;
        _attestationLevel = attestationLevel;
        _stepUpProven = stepUpProven;
        _isValid = isValid;
    }

    /// <summary>AuthClass: v1_human_session 等（5 class のいずれか）</summary>
    // AuthClass プロパティ
    public AuthClass AuthClass => _authClass;

    /// <summary>SubjectId: canonical subject（Keycloak sub など）</summary>
    // SubjectId プロパティ
    public string SubjectId => _subjectId;

    /// <summary>SubjectKind: human / workload / device / external_subject</summary>
    // SubjectKind プロパティ
    public string SubjectKind => _subjectKind;

    /// <summary>TokenId: JWT jti（revocation tracking 用）</summary>
    // TokenId プロパティ
    public string TokenId => _tokenId;

    /// <summary>IsValid: token 検証が成功したかどうか</summary>
    // IsValid プロパティ
    public bool IsValid => _isValid;

    /// <summary>
    /// ForHumanSession は v1_human_session AuthContext を構築するファクトリメソッド。
    /// OIDC code flow + DPoP 鍵束縛（RFC 9449）+ rotating refresh に対応する。
    /// </summary>
    // ForHumanSession ファクトリメソッド
    public static AuthContext ForHumanSession(
        string subjectId,
        string tenantId,
        string tokenId,
        IReadOnlyList<string> scopes,
        string? dpopJkt,
        bool stepUpProven)
    {
        // v1_human_session の固定属性を適用する（dimension override 禁止）
        return new AuthContext(
            authClass: AuthClass.V1HumanSession,
            subjectId: subjectId,
            // human session の subject_kind は常に "human"（spec §各 class の不変条件）
            subjectKind: "human",
            tokenId: tokenId,
            // session_id は Guid.NewGuid() で生成する
            sessionId: Guid.NewGuid().ToString("D"),
            tenantId: tenantId,
            // audience は human session では空文字列
            audience: string.Empty,
            scopes: scopes,
            dpopJkt: dpopJkt,
            attestationLevel: null,
            stepUpProven: stepUpProven,
            isValid: true);
    }

    /// <summary>
    /// ForWorkloadJwt は v1_workload_jwt AuthContext を構築するファクトリメソッド。
    /// K8s ServiceAccount projection / SPIFFE SVID（短 TTL JWT、自動 renew）に対応する。
    /// </summary>
    // ForWorkloadJwt ファクトリメソッド
    public static AuthContext ForWorkloadJwt(
        string subjectId,
        string tenantId,
        string tokenId,
        string audience)
    {
        // v1_workload_jwt の固定属性を適用する（dimension override 禁止）
        return new AuthContext(
            authClass: AuthClass.V1WorkloadJwt,
            subjectId: subjectId,
            // workload JWT の subject_kind は常に "workload"
            subjectKind: "workload",
            tokenId: tokenId,
            // workload は session を持たない（K8s pod lifecycle で管理する）
            sessionId: string.Empty,
            tenantId: tenantId,
            audience: audience,
            // workload のデフォルトスコープ（service.api のみ）
            scopes: new[] { "service.api" },
            dpopJkt: null,
            attestationLevel: null,
            // workload は step_up が不要（never ポリシー）
            stepUpProven: false,
            isValid: true);
    }

    /// <summary>
    /// ForDeviceAttest は v1_device_attest AuthContext を構築するファクトリメソッド。
    /// 工場端末・KIOSK・現場ハンドヘルド（device cert、長 TTL、one_shot refresh）に対応する。
    /// </summary>
    // ForDeviceAttest ファクトリメソッド
    public static AuthContext ForDeviceAttest(
        string subjectId,
        string tenantId,
        string tokenId,
        string attestationLevel,
        bool stepUpProven)
    {
        // v1_device_attest の固定属性を適用する（dimension override 禁止）
        return new AuthContext(
            authClass: AuthClass.V1DeviceAttest,
            subjectId: subjectId,
            // device attest の subject_kind は常に "device"
            subjectKind: "device",
            tokenId: tokenId,
            // device は session を持つ（device registration session ID を設定する）
            sessionId: Guid.NewGuid().ToString("D"),
            tenantId: tenantId,
            // device attest では audience は空文字列
            audience: string.Empty,
            // device のデフォルトスコープ
            scopes: new[] { "device.api" },
            // device cert で proof-of-possession を担保するため DPoP 不要
            dpopJkt: null,
            // attestationLevel: TPM / HSM / WebAuthn platform authenticator の種別
            attestationLevel: attestationLevel,
            stepUpProven: stepUpProven,
            isValid: true);
    }

    /// <summary>
    /// ForFederatedExchange は v1_federated_exchange AuthContext を構築するファクトリメソッド。
    /// 外部 IdP からの RFC 8693 token exchange（audience-restricted JWT）に対応する。
    /// </summary>
    // ForFederatedExchange ファクトリメソッド
    public static AuthContext ForFederatedExchange(
        string subjectId,
        string tenantId,
        string tokenId,
        string audience)
    {
        // v1_federated_exchange の固定属性を適用する（dimension override 禁止）
        return new AuthContext(
            authClass: AuthClass.V1FederatedExchange,
            subjectId: subjectId,
            // federated exchange の subject_kind は "external_subject"
            subjectKind: "external_subject",
            tokenId: tokenId,
            // federated exchange は session を持たない
            sessionId: string.Empty,
            tenantId: tenantId,
            // audience は federated exchange で必須（resource server を audience claim で絞る）
            audience: audience,
            // federated exchange のデフォルトスコープ
            scopes: new[] { "business_op" },
            dpopJkt: null,
            attestationLevel: null,
            // federated exchange は step_up 不要（never ポリシー）
            stepUpProven: false,
            isValid: true);
    }

    /// <summary>
    /// ForEmergencyStepUp は v1_emergency_step_up AuthContext を構築するファクトリメソッド。
    /// break-glass（always step_up、TTL&lt;10m、no refresh、purpose=emergency 強制）に対応する。
    /// </summary>
    // ForEmergencyStepUp ファクトリメソッド
    public static AuthContext ForEmergencyStepUp(
        string subjectId,
        string tenantId,
        string tokenId,
        string? dpopJkt)
    {
        // v1_emergency_step_up の固定属性を適用する（always step_up + purpose=emergency 強制）
        return new AuthContext(
            authClass: AuthClass.V1EmergencyStepUp,
            subjectId: subjectId,
            // emergency の subject_kind は常に "human"（workload による break-glass 禁止）
            subjectKind: "human",
            tokenId: tokenId,
            // emergency は session を持つ（break-glass セッションを追跡する）
            sessionId: Guid.NewGuid().ToString("D"),
            tenantId: tenantId,
            // emergency では audience は空文字列
            audience: string.Empty,
            // emergency のスコープ（break_glass を明示する）
            scopes: new[] { "emergency.break_glass" },
            dpopJkt: dpopJkt,
            attestationLevel: null,
            // v1_emergency_step_up は常に step_up 済みとして発行される（always ポリシー）
            stepUpProven: true,
            isValid: true);
    }

    /// <summary>
    /// ToGucSetters は PostgreSQL GUC の SET LOCAL 文を生成する。
    /// IsValid が false の場合は空リストを返す（無効な AuthContext で GUC を設定しない）。
    /// tier2 TenantContext の 4 GUC と組み合わせて session_context を構成する。
    /// </summary>
    // ToGucSetters メソッド: GUC SET LOCAL 文のリストを返す
    public IReadOnlyList<string> ToGucSetters()
    {
        // IsValid が false の場合は GUC setter を空にする（spec §AuthContext スキーマ準拠）
        if (!_isValid)
        {
            return Array.Empty<string>();
        }
        // 04_認証適合仕様.md §AuthContext スキーマの全 GUC 対応フィールドを SET LOCAL 文にする
        var setters = new List<string>
        {
            // auth_class GUC を設定する
            $"SET LOCAL app.auth_class = '{_authClass.ToSpecString()}';",
            // subject_id GUC を設定する
            $"SET LOCAL app.subject_id = '{_subjectId.Replace("'", "''")}';",
            // subject_kind GUC を設定する
            $"SET LOCAL app.subject_kind = '{_subjectKind}';",
            // token_id GUC を設定する
            $"SET LOCAL app.token_id = '{_tokenId.Replace("'", "''")}';",
            // session_id GUC を設定する
            $"SET LOCAL app.session_id = '{_sessionId}';",
            // audience GUC を設定する
            $"SET LOCAL app.audience = '{_audience.Replace("'", "''")}';",
            // step_up_proven GUC を設定する
            $"SET LOCAL app.step_up_proven = '{(_stepUpProven ? "true" : "false")}';",
        };
        // dpop_jkt が null でない場合のみ GUC を設定する（dpop_bound_jwt のみ）
        if (_dpopJkt is not null)
        {
            setters.Add($"SET LOCAL app.dpop_jkt = '{_dpopJkt.Replace("'", "''")}';");
        }
        // attestation_level が null でない場合のみ GUC を設定する（jwt_attested のみ）
        if (_attestationLevel is not null)
        {
            setters.Add($"SET LOCAL app.attestation_level = '{_attestationLevel.Replace("'", "''")}';");
        }
        // 生成した GUC setter リストを返す
        return setters.AsReadOnly();
    }
}

/// <summary>
/// AuthClassExtensions は AuthClass enum の文字列変換を提供する拡張メソッド。
/// spec の class 名（snake_case）と 1:1 対応する。
/// </summary>
// AuthClassExtensions 静的クラス: enum の文字列変換を提供する
public static class AuthClassExtensions
{
    /// <summary>
    /// ToSpecString は AuthClass を spec 定義の snake_case 文字列に変換する。
    /// </summary>
    // ToSpecString 拡張メソッド: enum → spec 文字列
    public static string ToSpecString(this AuthClass authClass)
    {
        // 各 class を spec 定義の snake_case 文字列にマッピングする
        return authClass switch
        {
            AuthClass.V1HumanSession => "v1_human_session",
            AuthClass.V1WorkloadJwt => "v1_workload_jwt",
            AuthClass.V1DeviceAttest => "v1_device_attest",
            AuthClass.V1FederatedExchange => "v1_federated_exchange",
            AuthClass.V1EmergencyStepUp => "v1_emergency_step_up",
            // 未知の class は例外を投げる（dead spec 防止）
            _ => throw new ArgumentOutOfRangeException(nameof(authClass), authClass, "Unknown AuthClass"),
        };
    }
}
