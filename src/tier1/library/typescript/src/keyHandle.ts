/**
 * keyHandle.ts — k1s0 tier1 Library TypeScript 実装: KeyClass enum + KeyHandle class (abstract)
 * 05_鍵管理適合仕様.md §v1 key_class セット（5 class）および
 * §5 層 defense-in-depth 層 A「compile: KeyHandle 必須引数化、生 key bytes 不可視」に準拠する。
 * 公開 API シグネチャに生 key bytes を露出しない opaque 型を実装する。
 */

/**
 * KeyClass は 05_鍵管理適合仕様.md §v1 key_class セット（5 class）を宣言する enum。
 * class 1 値が purpose / rotation_cadence / scope / backend / destruction_method を一意に導出する
 *（dimension override 禁止）。
 */
// KeyClass 列挙型: spec の class 名（snake_case）と 1:1 対応する
export const enum KeyClass {
  // v1_data_dek: データ暗号化鍵（DEK）— per-tenant / software_kms_wrapped / crypto_shred
  V1DataDek = "v1_data_dek",
  // v1_data_kek: 鍵暗号化鍵（KEK）— per-tenant / hsm_pkcs11_shamir_distributed / hsm_zeroize_all_shares
  V1DataKek = "v1_data_kek",
  // v1_token_signing: JWT / DPoP 署名鍵 — platform / hsm_pkcs11 / jwks_revoke
  V1TokenSigning = "v1_token_signing",
  // v1_audit_root_signing: audit hash chain root 署名鍵 — per-tenant / hsm_pkcs11 / external_notary_attest
  V1AuditRootSigning = "v1_audit_root_signing",
  // v1_mtls_workload: workload mTLS 鍵 — per_workload / spire / spire_revoke
  V1MtlsWorkload = "v1_mtls_workload",
}

/**
 * KeyHandle は生 key bytes を公開しない opaque 鍵抽象クラス。
 * 05_鍵管理適合仕様.md §KeyHandle / KeyMaterial の言語横断型 に準拠する。
 * Sign / Verify は OpenBao Transit への委譲として実装し、key bytes は tier1 境界を越えない。
 * abstract: 直接 new は禁止（具体実装クラスのみ instantiate 可能）。
 */
// KeyHandle 抽象クラス定義
export abstract class KeyHandle {
  // #keyId: OpenBao Transit のキー版数識別子（private class field: 外部からの直接アクセスを禁止する）
  readonly #keyId: string;
  // #keyClass: 鍵の用途クラス（private class field: 外部からの直接アクセスを禁止する）
  readonly #keyClass: KeyClass;
  // #isValid: OpenBao による有効性確認結果（private class field: 外部からの直接アクセスを禁止する）
  readonly #isValid: boolean;

  // コンストラクタ: keyId / keyClass / isValid のみを受け取る（生 key bytes は受け取らない）
  protected constructor(keyId: string, keyClass: KeyClass, isValid: boolean) {
    // keyId を設定する
    this.#keyId = keyId;
    // keyClass を設定する
    this.#keyClass = keyClass;
    // isValid を設定する
    this.#isValid = isValid;
  }

  /**
   * keyId は OpenBao Transit のキー版数識別子を返す。
   * 生 key bytes は露出しない（identifier のみを公開する）。
   */
  // keyId ゲッター: 識別子のみを返す（key bytes は返さない）
  get keyId(): string {
    // keyId フィールドを返す
    return this.#keyId;
  }

  /**
   * keyClass は 5 class のいずれかを返す（purpose bundle の代表値）。
   */
  // keyClass ゲッター: 用途クラスを返す
  get keyClass(): KeyClass {
    // keyClass フィールドを返す
    return this.#keyClass;
  }

  /**
   * isValid は OpenBao による鍵の有効性確認結果を返す（revoke / rotate 後 false になる）。
   */
  // isValid ゲッター: 有効性を返す
  get isValid(): boolean {
    // isValid フィールドを返す
    return this.#isValid;
  }

  /**
   * sign は payload を鍵で署名し、署名バイト列を返す。
   * 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
   */
  // sign 抽象メソッド: サブクラスで実装する
  abstract sign(payload: Uint8Array): Promise<Uint8Array>;

  /**
   * verify は payload と signature の一致を検証し、真偽値を返す。
   * 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
   */
  // verify 抽象メソッド: サブクラスで実装する
  abstract verify(payload: Uint8Array, signature: Uint8Array): Promise<boolean>;
}

/**
 * StubKeyHandle は OpenBao Transit 呼出なしに動作する stub 実装。
 * テスト・ドライラン用途（production では OpenBao 経由の KeyHandle 実装を使う）。
 */
// StubKeyHandle クラス定義: KeyHandle の stub 実装
export class StubKeyHandle extends KeyHandle {
  // コンストラクタ: keyId / keyClass / isValid を受け取る（生 key bytes は受け取らない）
  constructor(keyId: string, keyClass: KeyClass, isValid: boolean = true) {
    // 親クラスのコンストラクタを呼び出す（生 key bytes を渡さない）
    super(keyId, keyClass, isValid);
  }

  /**
   * sign は stub 実装として空 Uint8Array を返す。
   * production 実装では OpenBao Transit /v1/transit/sign/:name を呼び出す。
   */
  // sign メソッド実装: stub は空 Uint8Array を返す
  override async sign(_payload: Uint8Array): Promise<Uint8Array> {
    // stub: OpenBao Transit への委譲先は bfl/OpenBaoClient を参照する
    return new Uint8Array(0);
  }

  /**
   * verify は stub 実装として常に true を返す。
   * production 実装では OpenBao Transit /v1/transit/verify/:name を呼び出す。
   */
  // verify メソッド実装: stub は常に true を返す
  override async verify(_payload: Uint8Array, _signature: Uint8Array): Promise<boolean> {
    // stub: OpenBao Transit への委譲先は bfl/OpenBaoClient を参照する
    return true;
  }
}
