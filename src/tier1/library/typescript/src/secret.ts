/**
 * secret.ts — k1s0 tier1 Library TypeScript 実装: Secret Management L3 facade interface
 * 05_鍵管理適合仕様.md §v1 key_class セット（5 class）および §SecretStore 抽象 に準拠する。
 * Rust core/secret.rs の SecretStore trait と 4 言語等価強度を保つ。
 * OpenBao / AWS Secrets Manager 等のバックエンドを実装で切り替えられる interface を宣言する。
 * 公開 API シグネチャに生 key bytes / 生シークレット値を露出しない。
 */

// KeyClass を keyHandle.ts から参照する（4 言語等価のため同一 enum を使う）
import type { KeyClass } from "./keyHandle.js";

/**
 * SecretMetadata はシークレットのメタデータを宣言する型。
 * 生のシークレット値は含まない（SecretStore.withSecret の callback パターンで別管理する）。
 * Rust core/secret.rs の SecretMetadata struct と 1:1 対応する。
 */
// SecretMetadata 型定義: シークレットのメタデータのみを保持する
export interface SecretMetadata {
  // secretId: シークレットの識別子（UUID v7 形式）
  readonly secretId: string;
  // version: シークレットのバージョン番号（ローテーション追跡用）
  readonly version: number;
  // keyClass: このシークレットを保護している鍵のクラス（05_鍵管理適合仕様 §v1 5 class）
  readonly keyClass: KeyClass;
  // tenantId: シークレットが属するテナントの識別子
  readonly tenantId: string;
  // isActive: シークレットが有効かどうか（revoke / rotate 後 false になる）
  readonly isActive: boolean;
}

/**
 * SecretRotationPolicy はシークレットローテーションポリシーを宣言する型。
 * Rust core/secret.rs の SecretRotationPolicy enum と 4 言語等価強度を保つ。
 */
// SecretRotationPolicy 型定義: ローテーション方式を宣言する
export type SecretRotationPolicy =
  // manual: 手動ローテーション（自動ローテーションなし）
  | { readonly kind: "manual" }
  // onDemand: 要求時ローテーション（呼び出し元が rotate を呼ぶ）
  | { readonly kind: "onDemand" }
  // scheduled: スケジュールローテーション（間隔秒数で指定）
  | { readonly kind: "scheduled"; readonly intervalSeconds: number };

/**
 * SecretStore は Secret Management の L3 抽象 interface。
 * OpenBao / AWS Secrets Manager 等の OSS / サービスを実装で切り替えられる。
 * Rust core/secret.rs の SecretStore trait と 4 言語等価強度を保つ。
 * 公開 API に生シークレット値を返さない（withSecret の callback パターンを使う）。
 */
// SecretStore インターフェース定義
export interface SecretStore {
  /**
   * getMetadata はシークレットのメタデータのみを返す（値は返さない）。
   * Rust の get_metadata(&self, secret_id, tenant_id) -> Result<Option<SecretMetadata>> に対応する。
   */
  // getMetadata メソッド: シークレットのメタデータのみを返す（値は返さない）
  getMetadata(
    // secretId: シークレットの識別子（UUID v7 形式）
    secretId: string,
    // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
    tenantId: string
  ): Promise<SecretMetadata | null>;

  /**
   * withSecret はシークレット値を callback に渡して処理させる。
   * callback 外にシークレット値が漏れない設計（値は返さない）。
   * Rust の with_secret(&self, secret_id, tenant_id, f: FnOnce(&SecretValue) -> T) -> Result<T> に対応する。
   */
  // withSecret メソッド: シークレット値を callback に渡す（値は返さない）
  withSecret<T>(
    // secretId: シークレットの識別子（UUID v7 形式）
    secretId: string,
    // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
    tenantId: string,
    // callback: シークレットのバイト列を受け取り T を返す処理（バイト列を漏洩させない）
    callback: (secretBytes: Readonly<Uint8Array>) => T
  ): Promise<T>;

  /**
   * rotate はシークレットを新しいバージョンにローテーションする。
   * 返した SecretMetadata に新しい version が含まれる。
   * Rust の rotate(&self, secret_id, tenant_id) -> Result<SecretMetadata> に対応する。
   */
  // rotate メソッド: シークレットを新しいバージョンにローテーションする
  rotate(
    // secretId: シークレットの識別子（UUID v7 形式）
    secretId: string,
    // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
    tenantId: string
  ): Promise<SecretMetadata>;

  /**
   * revoke はシークレットを無効化する（isActive=false にする）。
   * revoke 後の withSecret 呼び出しは例外を投げる。
   * Rust の revoke(&self, secret_id, tenant_id) -> Result<()> に対応する。
   */
  // revoke メソッド: シークレットを無効化する
  revoke(
    // secretId: シークレットの識別子（UUID v7 形式）
    secretId: string,
    // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
    tenantId: string
  ): Promise<void>;
}
