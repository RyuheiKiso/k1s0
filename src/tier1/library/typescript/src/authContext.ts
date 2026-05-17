/**
 * authContext.ts — k1s0 tier1 Library TypeScript 実装: AuthClass enum + AuthContext class
 * 04_認証適合仕様.md §v1 auth_class セット（5 class）および
 * §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
 * 生 access_token / refresh_token は公開シグネチャに一切含まれない。
 */

/**
 * AuthClass は 04_認証適合仕様.md §v1 auth_class セット（5 class）を宣言する enum。
 * class 1 値が subject_kind / token_type / lifetime_class / refresh_policy / step_up_required を
 * 一意に導出する（dimension override 禁止）。
 */
// AuthClass 列挙型: spec の class 名（snake_case）と 1:1 対応する
export const enum AuthClass {
  // v1_human_session: 業務担当者 SPA セッション（OIDC + DPoP + rotating refresh）
  V1HumanSession = "v1_human_session",
  // v1_workload_jwt: K8s ServiceAccount / SPIFFE SVID（短 TTL JWT、自動 renew）
  V1WorkloadJwt = "v1_workload_jwt",
  // v1_device_attest: 工場端末（device cert、長 TTL、one_shot refresh）
  V1DeviceAttest = "v1_device_attest",
  // v1_federated_exchange: 外部 IdP からの RFC 8693 token exchange
  V1FederatedExchange = "v1_federated_exchange",
  // v1_emergency_step_up: break-glass（always step_up、TTL<10m、no refresh）
  V1EmergencyStepUp = "v1_emergency_step_up",
}

/**
 * AuthContext は tier1 Library が transaction 開始時に PostgreSQL GUC に SET LOCAL する
 * 認証拡張フィールドを保持する opaque 型。
 * 04_認証適合仕様.md §AuthContext スキーマ準拠。
 * 生 access_token / refresh_token はフィールドに含まない。
 */
// AuthContext クラス定義（生 token 非保持）
export class AuthContext {
  // #authClass: v1_human_session 等（private class field: 外部からの直接アクセスを禁止する）
  readonly #authClass: AuthClass;
  // #subjectId: canonical subject（private class field）
  readonly #subjectId: string;
  // #subjectKind: human / workload / device / external_subject（private class field）
  readonly #subjectKind: string;
  // #tokenId: JWT jti（private class field: revocation tracking 用）
  readonly #tokenId: string;
  // #sessionId: human / device のセッション識別子（private class field）
  readonly #sessionId: string;
  // #tenantId: リクエストのテナント識別子（private class field）
  readonly #tenantId: string;
  // #audience: token aud（private class field: v1_federated_exchange で必須）
  readonly #audience: string;
  // #scopes: OAuth scopes（private class field）
  readonly #scopes: readonly string[];
  // #dpopJkt: DPoP key thumbprint（private class field: dpop_bound_jwt のみ）
  readonly #dpopJkt: string | undefined;
  // #attestationLevel: device attestation level（private class field: jwt_attested のみ）
  readonly #attestationLevel: string | undefined;
  // #stepUpProven: 最終 step_up challenge 済みフラグ（private class field）
  readonly #stepUpProven: boolean;
  // #isValid: token 検証結果（private class field）
  readonly #isValid: boolean;

  // コンストラクタは private（ファクトリメソッド経由のみ生成可能）
  private constructor(params: {
    // authClass: 5 class のいずれか
    readonly authClass: AuthClass;
    // subjectId: canonical subject
    readonly subjectId: string;
    // subjectKind: human / workload / device / external_subject
    readonly subjectKind: string;
    // tokenId: JWT jti
    readonly tokenId: string;
    // sessionId: セッション識別子
    readonly sessionId: string;
    // tenantId: テナント識別子
    readonly tenantId: string;
    // audience: token aud
    readonly audience: string;
    // scopes: OAuth scopes
    readonly scopes: readonly string[];
    // dpopJkt: DPoP key thumbprint（オプション）
    readonly dpopJkt?: string;
    // attestationLevel: device attestation level（オプション）
    readonly attestationLevel?: string;
    // stepUpProven: step_up 済みフラグ
    readonly stepUpProven: boolean;
    // isValid: 検証結果
    readonly isValid: boolean;
  }) {
    // 全フィールドを初期化する
    this.#authClass = params.authClass;
    this.#subjectId = params.subjectId;
    this.#subjectKind = params.subjectKind;
    this.#tokenId = params.tokenId;
    this.#sessionId = params.sessionId;
    this.#tenantId = params.tenantId;
    this.#audience = params.audience;
    this.#scopes = params.scopes;
    this.#dpopJkt = params.dpopJkt;
    this.#attestationLevel = params.attestationLevel;
    this.#stepUpProven = params.stepUpProven;
    this.#isValid = params.isValid;
  }

  /** authClass: v1_human_session 等を返す */
  // authClass ゲッター
  get authClass(): AuthClass {
    // authClass フィールドを返す
    return this.#authClass;
  }

  /** subjectId: canonical subject を返す */
  // subjectId ゲッター
  get subjectId(): string {
    // subjectId フィールドを返す
    return this.#subjectId;
  }

  /** isValid: token 検証結果を返す */
  // isValid ゲッター
  get isValid(): boolean {
    // isValid フィールドを返す
    return this.#isValid;
  }

  /**
   * forHumanSession は v1_human_session AuthContext を構築するファクトリメソッド。
   * OIDC code flow + DPoP 鍵束縛（RFC 9449）+ rotating refresh に対応する。
   */
  // forHumanSession ファクトリメソッド
  static forHumanSession(params: {
    // subjectId: canonical subject
    readonly subjectId: string;
    // tenantId: テナント識別子
    readonly tenantId: string;
    // tokenId: JWT jti
    readonly tokenId: string;
    // sessionId: セッション識別子（UUID v4 等）
    readonly sessionId: string;
    // scopes: OAuth scopes
    readonly scopes: readonly string[];
    // dpopJkt: DPoP key thumbprint（オプション）
    readonly dpopJkt?: string;
    // stepUpProven: step_up 済みフラグ
    readonly stepUpProven: boolean;
  }): AuthContext {
    // v1_human_session の固定属性を適用する（dimension override 禁止）
    return new AuthContext({
      authClass: AuthClass.V1HumanSession,
      subjectId: params.subjectId,
      // human session の subject_kind は常に "human"（spec §各 class の不変条件）
      subjectKind: "human",
      tokenId: params.tokenId,
      sessionId: params.sessionId,
      tenantId: params.tenantId,
      // audience は human session では空文字列
      audience: "",
      scopes: params.scopes,
      dpopJkt: params.dpopJkt,
      // human session には attestationLevel は不要
      attestationLevel: undefined,
      stepUpProven: params.stepUpProven,
      isValid: true,
    });
  }

  /**
   * forWorkloadJwt は v1_workload_jwt AuthContext を構築するファクトリメソッド。
   * K8s ServiceAccount projection / SPIFFE SVID（短 TTL JWT、自動 renew）に対応する。
   */
  // forWorkloadJwt ファクトリメソッド
  static forWorkloadJwt(params: {
    // subjectId: canonical subject
    readonly subjectId: string;
    // tenantId: テナント識別子
    readonly tenantId: string;
    // tokenId: JWT jti
    readonly tokenId: string;
    // audience: token aud（resource server）
    readonly audience: string;
  }): AuthContext {
    // v1_workload_jwt の固定属性を適用する（dimension override 禁止）
    return new AuthContext({
      authClass: AuthClass.V1WorkloadJwt,
      subjectId: params.subjectId,
      // workload JWT の subject_kind は常に "workload"
      subjectKind: "workload",
      tokenId: params.tokenId,
      // workload は session を持たない（K8s pod lifecycle で管理する）
      sessionId: "",
      tenantId: params.tenantId,
      audience: params.audience,
      // workload のデフォルトスコープ（service.api のみ）
      scopes: ["service.api"],
      // workload は DPoP 不要
      dpopJkt: undefined,
      attestationLevel: undefined,
      // workload は step_up が不要（never ポリシー）
      stepUpProven: false,
      isValid: true,
    });
  }

  /**
   * forDeviceAttest は v1_device_attest AuthContext を構築するファクトリメソッド。
   * 工場端末・KIOSK・現場ハンドヘルド（device cert、長 TTL、one_shot refresh）に対応する。
   */
  // forDeviceAttest ファクトリメソッド
  static forDeviceAttest(params: {
    // subjectId: canonical subject
    readonly subjectId: string;
    // tenantId: テナント識別子
    readonly tenantId: string;
    // tokenId: JWT jti
    readonly tokenId: string;
    // sessionId: device registration session ID
    readonly sessionId: string;
    // attestationLevel: TPM / HSM / WebAuthn platform authenticator の種別
    readonly attestationLevel: string;
    // stepUpProven: step_up 済みフラグ（on_first_use ポリシー）
    readonly stepUpProven: boolean;
  }): AuthContext {
    // v1_device_attest の固定属性を適用する（dimension override 禁止）
    return new AuthContext({
      authClass: AuthClass.V1DeviceAttest,
      subjectId: params.subjectId,
      // device attest の subject_kind は常に "device"
      subjectKind: "device",
      tokenId: params.tokenId,
      sessionId: params.sessionId,
      tenantId: params.tenantId,
      // device attest では audience は空文字列
      audience: "",
      // device のデフォルトスコープ
      scopes: ["device.api"],
      // device cert で proof-of-possession を担保するため DPoP 不要
      dpopJkt: undefined,
      attestationLevel: params.attestationLevel,
      stepUpProven: params.stepUpProven,
      isValid: true,
    });
  }

  /**
   * forFederatedExchange は v1_federated_exchange AuthContext を構築するファクトリメソッド。
   * 外部 IdP からの RFC 8693 token exchange（audience-restricted JWT）に対応する。
   */
  // forFederatedExchange ファクトリメソッド
  static forFederatedExchange(params: {
    // subjectId: canonical subject（外部 IdP の subject）
    readonly subjectId: string;
    // tenantId: テナント識別子
    readonly tenantId: string;
    // tokenId: JWT jti
    readonly tokenId: string;
    // audience: resource server を絞る audience claim
    readonly audience: string;
  }): AuthContext {
    // v1_federated_exchange の固定属性を適用する（dimension override 禁止）
    return new AuthContext({
      authClass: AuthClass.V1FederatedExchange,
      subjectId: params.subjectId,
      // federated exchange の subject_kind は "external_subject"
      subjectKind: "external_subject",
      tokenId: params.tokenId,
      // federated exchange は session を持たない
      sessionId: "",
      tenantId: params.tenantId,
      // audience は federated exchange で必須（resource server を audience claim で絞る）
      audience: params.audience,
      // federated exchange のデフォルトスコープ
      scopes: ["business_op"],
      dpopJkt: undefined,
      attestationLevel: undefined,
      // federated exchange は step_up 不要（never ポリシー）
      stepUpProven: false,
      isValid: true,
    });
  }

  /**
   * forEmergencyStepUp は v1_emergency_step_up AuthContext を構築するファクトリメソッド。
   * break-glass（always step_up、TTL<10m、no refresh、purpose=emergency 強制）に対応する。
   */
  // forEmergencyStepUp ファクトリメソッド
  static forEmergencyStepUp(params: {
    // subjectId: canonical subject
    readonly subjectId: string;
    // tenantId: テナント識別子
    readonly tenantId: string;
    // tokenId: JWT jti
    readonly tokenId: string;
    // sessionId: break-glass セッション識別子
    readonly sessionId: string;
    // dpopJkt: DPoP key thumbprint（オプション）
    readonly dpopJkt?: string;
  }): AuthContext {
    // v1_emergency_step_up の固定属性を適用する（always step_up + purpose=emergency 強制）
    return new AuthContext({
      authClass: AuthClass.V1EmergencyStepUp,
      subjectId: params.subjectId,
      // emergency の subject_kind は常に "human"（workload による break-glass 禁止）
      subjectKind: "human",
      tokenId: params.tokenId,
      sessionId: params.sessionId,
      tenantId: params.tenantId,
      // emergency では audience は空文字列
      audience: "",
      // emergency のスコープ（break_glass を明示する）
      scopes: ["emergency.break_glass"],
      dpopJkt: params.dpopJkt,
      attestationLevel: undefined,
      // v1_emergency_step_up は常に step_up 済みとして発行される（always ポリシー）
      stepUpProven: true,
      isValid: true,
    });
  }

  /**
   * toGucSetters は PostgreSQL GUC の SET LOCAL 文の配列を返す。
   * isValid が false の場合は空配列を返す（無効な AuthContext で GUC を設定しない）。
   * tier2 TenantContext の 4 GUC と組み合わせて session_context を構成する。
   */
  // toGucSetters メソッド: GUC SET LOCAL 文の配列を返す
  toGucSetters(): readonly string[] {
    // isValid が false の場合は GUC setter を空にする（spec §AuthContext スキーマ準拠）
    if (!this.#isValid) {
      return [];
    }
    // SQL injection 対策: single quote をエスケープするヘルパー関数
    const esc = (s: string): string => s.replace(/'/g, "''");
    // 04_認証適合仕様.md §AuthContext スキーマの全 GUC 対応フィールドを SET LOCAL 文にする
    const setters: string[] = [
      // auth_class GUC を設定する
      `SET LOCAL app.auth_class = '${this.#authClass}';`,
      // subject_id GUC を設定する（single quote をエスケープする）
      `SET LOCAL app.subject_id = '${esc(this.#subjectId)}';`,
      // subject_kind GUC を設定する
      `SET LOCAL app.subject_kind = '${this.#subjectKind}';`,
      // token_id GUC を設定する
      `SET LOCAL app.token_id = '${esc(this.#tokenId)}';`,
      // session_id GUC を設定する
      `SET LOCAL app.session_id = '${this.#sessionId}';`,
      // audience GUC を設定する
      `SET LOCAL app.audience = '${esc(this.#audience)}';`,
      // step_up_proven GUC を設定する
      `SET LOCAL app.step_up_proven = '${this.#stepUpProven}';`,
    ];
    // dpopJkt が設定されている場合のみ GUC を設定する（dpop_bound_jwt のみ）
    if (this.#dpopJkt !== undefined) {
      setters.push(`SET LOCAL app.dpop_jkt = '${esc(this.#dpopJkt)}';`);
    }
    // attestationLevel が設定されている場合のみ GUC を設定する（jwt_attested のみ）
    if (this.#attestationLevel !== undefined) {
      setters.push(`SET LOCAL app.attestation_level = '${esc(this.#attestationLevel)}';`);
    }
    // 生成した GUC setter 配列を返す
    return setters;
  }
}
