// AuthContextCompat.cs — k1s0 tier1 Library .NET Framework 4.6.2+ Companion
// AuthContext の .NET Framework 互換ラッパーを提供する。
// .NET 8 版の K1s0.Tier1.AuthContext と同等の機能を .NET Framework 4.6.2+ で使用可能にする。
// docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md §TLS バージョン注記:
//   .NET Framework 4.6.2+ は TLS 1.2 をデフォルトで有効化する。
//   ServicePointManager.SecurityProtocol に TLS 1.2 を明示的に設定することを推奨する。

// System: Guid / ArgumentNullException / Type に使用する
using System;
// System.Collections.Generic: List に使用する（IReadOnlyList は .NET 4.5+ で利用可能）
using System.Collections.Generic;
// System.Text: StringBuilder に使用する
using System.Text;

// k1s0 tier1 .NET Framework 互換名前空間
namespace K1s0.Tier1.Compat
{
    /// <summary>
    /// AuthClassCompat は 04_認証適合仕様.md §v1 auth_class セット（5 class）を宣言する。
    /// .NET 8 版 K1s0.Tier1.AuthClass と同等の定義を .NET Framework 4.6.2+ で提供する。
    /// </summary>
    // AuthClassCompat 列挙型: 5 class のいずれかを表す（.NET Framework 互換）
    public enum AuthClassCompat
    {
        /// <summary>v1_human_session: 業務担当者 SPA セッション（OIDC + DPoP + rotating refresh）</summary>
        V1HumanSession,
        /// <summary>v1_workload_jwt: K8s ServiceAccount / SPIFFE SVID（短 TTL JWT、自動 renew）</summary>
        V1WorkloadJwt,
        /// <summary>v1_device_attest: 工場端末（device cert、長 TTL、one_shot refresh）</summary>
        V1DeviceAttest,
        /// <summary>v1_federated_exchange: 外部 IdP からの RFC 8693 token exchange</summary>
        V1FederatedExchange,
        /// <summary>v1_emergency_step_up: break-glass（always step_up、TTL<10m、no refresh）</summary>
        V1EmergencyStepUp,
    }

    /// <summary>
    /// AuthContextCompat は tier1 Library が transaction 開始時に PostgreSQL GUC に SET LOCAL する
    /// 認証拡張フィールドを保持する opaque 型の .NET Framework 4.6.2+ 互換ラッパー。
    /// .NET 8 版 K1s0.Tier1.AuthContext と同等の機能を提供する。
    /// 生 access_token / refresh_token はフィールドに含まない。
    /// </summary>
    // AuthContextCompat クラス定義（sealed: 外部からの継承を禁止する）
    public sealed class AuthContextCompat
    {
        // _authClass: v1_human_session 等（5 class のいずれか）
        private readonly AuthClassCompat _authClass;
        // _subjectId: canonical subject（Keycloak sub など）
        private readonly string _subjectId;
        // _subjectKind: human / workload / device / external_subject
        private readonly string _subjectKind;
        // _tokenId: JWT jti（revocation tracking 用）
        private readonly string _tokenId;
        // _sessionId: human / device のセッション識別子（workload は空文字列）
        private readonly string _sessionId;
        // _tenantId: リクエストのテナント識別子
        private readonly string _tenantId;
        // _audience: token aud（v1_federated_exchange で必須）
        private readonly string _audience;
        // _scopes: OAuth scopes（service.api / emergency.break_glass 等）
        private readonly IList<string> _scopes;
        // _dpopJkt: DPoP key thumbprint（dpop_bound_jwt のみ設定される、null は未設定を意味する）
        private readonly string _dpopJkt;
        // _attestationLevel: device attestation level（jwt_attested のみ設定される、null は未設定を意味する）
        private readonly string _attestationLevel;
        // _stepUpProven: 最終 step_up challenge 済みフラグ
        private readonly bool _stepUpProven;
        // _isValid: token 検証が成功したかどうか
        private readonly bool _isValid;

        // コンストラクタは private（ファクトリメソッド経由のみ生成可能）
        private AuthContextCompat(
            AuthClassCompat authClass,
            string subjectId,
            string subjectKind,
            string tokenId,
            string sessionId,
            string tenantId,
            string audience,
            IList<string> scopes,
            string dpopJkt,
            string attestationLevel,
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
        public AuthClassCompat AuthClass { get { return _authClass; } }

        /// <summary>SubjectId: canonical subject（Keycloak sub など）</summary>
        // SubjectId プロパティ
        public string SubjectId { get { return _subjectId; } }

        /// <summary>IsValid: token 検証が成功したかどうか</summary>
        // IsValid プロパティ
        public bool IsValid { get { return _isValid; } }

        /// <summary>
        /// ForHumanSession は v1_human_session AuthContextCompat を構築するファクトリメソッド。
        /// .NET Framework 4.6.2+ 互換: Guid.NewGuid() は .NET Framework 4.0+ で使用可能。
        /// </summary>
        // ForHumanSession ファクトリメソッド
        public static AuthContextCompat ForHumanSession(
            string subjectId,
            string tenantId,
            string tokenId,
            IList<string> scopes,
            string dpopJkt,
            bool stepUpProven)
        {
            // v1_human_session の固定属性を適用する（dimension override 禁止）
            return new AuthContextCompat(
                authClass: AuthClassCompat.V1HumanSession,
                subjectId: subjectId,
                // human session の subject_kind は常に "human"（spec §各 class の不変条件）
                subjectKind: "human",
                tokenId: tokenId,
                // session_id は Guid.NewGuid() で生成する（.NET Framework 4.0+ 互換）
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
        /// ForWorkloadJwt は v1_workload_jwt AuthContextCompat を構築するファクトリメソッド。
        /// K8s ServiceAccount projection / SPIFFE SVID（短 TTL JWT、自動 renew）に対応する。
        /// </summary>
        // ForWorkloadJwt ファクトリメソッド
        public static AuthContextCompat ForWorkloadJwt(
            string subjectId,
            string tenantId,
            string tokenId,
            string audience)
        {
            // v1_workload_jwt の固定属性を適用する（dimension override 禁止）
            return new AuthContextCompat(
                authClass: AuthClassCompat.V1WorkloadJwt,
                subjectId: subjectId,
                // workload JWT の subject_kind は常に "workload"
                subjectKind: "workload",
                tokenId: tokenId,
                // workload は session を持たない（K8s pod lifecycle で管理する）
                sessionId: string.Empty,
                tenantId: tenantId,
                audience: audience,
                // workload のデフォルトスコープ（service.api のみ）
                scopes: new List<string> { "service.api" },
                dpopJkt: null,
                attestationLevel: null,
                // workload は step_up が不要（never ポリシー）
                stepUpProven: false,
                isValid: true);
        }

        /// <summary>
        /// ForEmergencyStepUp は v1_emergency_step_up AuthContextCompat を構築するファクトリメソッド。
        /// break-glass（always step_up、TTL&lt;10m、no refresh、purpose=emergency 強制）に対応する。
        /// </summary>
        // ForEmergencyStepUp ファクトリメソッド
        public static AuthContextCompat ForEmergencyStepUp(
            string subjectId,
            string tenantId,
            string tokenId,
            string dpopJkt)
        {
            // v1_emergency_step_up の固定属性を適用する（always step_up + purpose=emergency 強制）
            return new AuthContextCompat(
                authClass: AuthClassCompat.V1EmergencyStepUp,
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
                scopes: new List<string> { "emergency.break_glass" },
                dpopJkt: dpopJkt,
                attestationLevel: null,
                // v1_emergency_step_up は常に step_up 済みとして発行される（always ポリシー）
                stepUpProven: true,
                isValid: true);
        }

        /// <summary>
        /// ToGucSetters は PostgreSQL GUC の SET LOCAL 文リストを生成する。
        /// IsValid が false の場合は空リストを返す（無効な AuthContextCompat で GUC を設定しない）。
        /// .NET Framework 4.6.2+ 互換: IReadOnlyList ではなく IList を返す。
        /// </summary>
        // ToGucSetters メソッド: GUC SET LOCAL 文のリストを返す
        public IList<string> ToGucSetters()
        {
            // IsValid が false の場合は GUC setter を空にする（spec §AuthContext スキーマ準拠）
            if (!_isValid)
            {
                // 空リストを返す
                return new List<string>();
            }
            // 04_認証適合仕様.md §AuthContext スキーマの全 GUC 対応フィールドを SET LOCAL 文にする
            var setters = new List<string>
            {
                // auth_class GUC を設定する
                "SET LOCAL app.auth_class = '" + AuthClassCompatExtensions.ToSpecString(_authClass) + "';",
                // subject_id GUC を設定する（single quote をエスケープする）
                "SET LOCAL app.subject_id = '" + _subjectId.Replace("'", "''") + "';",
                // subject_kind GUC を設定する
                "SET LOCAL app.subject_kind = '" + _subjectKind + "';",
                // token_id GUC を設定する
                "SET LOCAL app.token_id = '" + _tokenId.Replace("'", "''") + "';",
                // session_id GUC を設定する
                "SET LOCAL app.session_id = '" + _sessionId + "';",
                // audience GUC を設定する
                "SET LOCAL app.audience = '" + _audience.Replace("'", "''") + "';",
                // step_up_proven GUC を設定する
                "SET LOCAL app.step_up_proven = '" + (_stepUpProven ? "true" : "false") + "';",
            };
            // dpop_jkt が null でない場合のみ GUC を設定する（dpop_bound_jwt のみ）
            if (_dpopJkt != null)
            {
                setters.Add("SET LOCAL app.dpop_jkt = '" + _dpopJkt.Replace("'", "''") + "';");
            }
            // attestation_level が null でない場合のみ GUC を設定する（jwt_attested のみ）
            if (_attestationLevel != null)
            {
                setters.Add("SET LOCAL app.attestation_level = '" + _attestationLevel.Replace("'", "''") + "';");
            }
            // 生成した GUC setter リストを返す
            return setters;
        }
    }

    /// <summary>
    /// AuthClassCompatExtensions は AuthClassCompat enum の文字列変換を提供する静的クラス。
    /// .NET Framework 4.6.2+ 互換: 拡張メソッドは .NET Framework 3.5+ で使用可能。
    /// </summary>
    // AuthClassCompatExtensions 静的クラス: enum の文字列変換を提供する
    public static class AuthClassCompatExtensions
    {
        /// <summary>
        /// ToSpecString は AuthClassCompat を spec 定義の snake_case 文字列に変換する。
        /// </summary>
        // ToSpecString 静的メソッド: enum → spec 文字列
        public static string ToSpecString(AuthClassCompat authClass)
        {
            // 各 class を spec 定義の snake_case 文字列にマッピングする
            switch (authClass)
            {
                // v1_human_session を返す
                case AuthClassCompat.V1HumanSession:
                    return "v1_human_session";
                // v1_workload_jwt を返す
                case AuthClassCompat.V1WorkloadJwt:
                    return "v1_workload_jwt";
                // v1_device_attest を返す
                case AuthClassCompat.V1DeviceAttest:
                    return "v1_device_attest";
                // v1_federated_exchange を返す
                case AuthClassCompat.V1FederatedExchange:
                    return "v1_federated_exchange";
                // v1_emergency_step_up を返す
                case AuthClassCompat.V1EmergencyStepUp:
                    return "v1_emergency_step_up";
                // 未知の class は例外を投げる（dead spec 防止）
                default:
                    throw new ArgumentOutOfRangeException("authClass", authClass, "Unknown AuthClassCompat");
            }
        }
    }
}
