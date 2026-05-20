/**
 * secretImpl.ts — k1s0 tier1 Library TypeScript 実装: SecretStore の in-memory facade 実装
 * C# SecretStoreImpl.cs / Go secret_impl.go と同等の深度で SecretStore を実装する。
 * OSS 型（vault-client 等）を公開 API シグネチャに一切露出しない。
 * 生シークレット値は callback パターン（withSecret）でのみ外部に渡し、直接返さない。
 */

// 公開 interface をインポートする
import type { SecretMetadata, SecretStore } from "./secret.js";
// KeyClass を keyHandle.ts から参照する
import { KeyClass } from "./keyHandle.js";

// ---- secretEntry: 内部エントリ型 ----

// SecretEntry は in-memory SecretStore に格納するシークレットエントリを表す内部型。
// secretBytes はゼロクリア対象のため、直接公開しない。
interface SecretEntry {
  // metadata: シークレットのメタデータ（値は含まない）
  readonly metadata: SecretMetadata;
  // secretBytes: シークレットの実際のバイト列（外部に直接露出禁止）
  // Uint8Array は mutable なため、コピーして渡す
  secretBytes: Uint8Array;
}

// zeroize はシークレットのバイト列をゼロクリアする内部ヘルパー関数。
// 05_鍵管理適合仕様.md §メモリゼロクリア規律に準拠する（参照が残る間は 0 で上書きする）。
function zeroize(buf: Uint8Array): void {
  // バイト列を 0 で上書きする
  buf.fill(0);
}

// ---- InMemorySecretStore 実装 ----

/**
 * InMemorySecretStore は SecretStore の in-memory stub 実装クラス。
 * テスト / ドライラン用にシークレットを in-memory に保持する。
 * OpenBao に依存せず、単体テストで利用できる実装とする。
 * Rust inMemorySecretStore / Go inMemorySecretStoreImpl / C# SecretStoreImpl と
 * 4 言語等価強度を保つ。
 */
// InMemorySecretStore クラス定義（テスト用の公開クラス）
export class InMemorySecretStore implements SecretStore {
  // #entries: secretId → SecretEntry のマップ（内部管理用）
  readonly #entries: Map<string, SecretEntry> = new Map();

  /**
   * putSecret はテスト用にシークレットを in-memory に追加するヘルパーメソッド。
   * SecretStore interface 外のメソッド（テスト用）。
   */
  // putSecret メソッド: テスト用にシークレットを追加する
  putSecret(
    // secretId: シークレットの識別子（UUID v7 形式）
    secretId: string,
    // tenantId: テナント識別子
    tenantId: string,
    // keyClass: このシークレットを保護している鍵のクラス
    keyClass: KeyClass,
    // secretBytes: シークレットのバイト列（コピーして保存する）
    secretBytes: Uint8Array,
  ): void {
    // secretBytes をコピーして保存する（外部 Uint8Array の変更から保護する）
    const copied = new Uint8Array(secretBytes.length);
    // コピーする
    copied.set(secretBytes);
    // エントリを追加する
    this.#entries.set(secretId, {
      // メタデータを構築する
      metadata: {
        secretId,
        // バージョン 1 から開始する
        version: 1,
        keyClass,
        tenantId,
        // 追加直後は有効状態
        isActive: true,
      },
      // シークレットのバイト列を保存する
      secretBytes: copied,
    });
  }

  /**
   * getMetadata はシークレットのメタデータのみを返す（値は返さない）。
   * 存在しない場合は null を返す。
   */
  // getMetadata メソッド実装: シークレットのメタデータのみを返す
  async getMetadata(secretId: string, tenantId: string): Promise<SecretMetadata | null> {
    // secretId でエントリを検索する
    const entry = this.#entries.get(secretId);
    // エントリが存在しない場合は null を返す
    if (entry === undefined) {
      return null;
    }
    // テナント不一致の場合はエラーを投げる（tenant 分離必須）
    if (entry.metadata.tenantId !== tenantId) {
      // テナント分離違反: 不正アクセスを禁止する
      throw new Error(
        `InMemorySecretStore.getMetadata: tenant mismatch secretId=${secretId} expected=${entry.metadata.tenantId} actual=${tenantId}`
      );
    }
    // メタデータのコピーを返す（内部オブジェクトへの直接参照を避ける）
    return { ...entry.metadata };
  }

  /**
   * withSecret はシークレット値を callback に渡して処理させる。
   * callback 外にシークレット値が漏れない設計（値は返さない）。
   * callback の引数 secretBytes は呼び出し後にゼロクリアする（外部で保持させない）。
   */
  // withSecret メソッド実装: シークレット値を callback に渡す（値は返さない）
  async withSecret<T>(
    // secretId: シークレットの識別子（UUID v7 形式）
    secretId: string,
    // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
    tenantId: string,
    // callback: シークレットのバイト列を受け取り T を返す処理（バイト列を漏洩させない）
    callback: (secretBytes: Readonly<Uint8Array>) => T
  ): Promise<T> {
    // secretId でエントリを検索する
    const entry = this.#entries.get(secretId);
    // エントリが存在しない場合はエラーを投げる
    if (entry === undefined) {
      throw new Error(`InMemorySecretStore.withSecret: secret not found secretId=${secretId}`);
    }
    // テナント不一致の場合はエラーを投げる（tenant 分離必須）
    if (entry.metadata.tenantId !== tenantId) {
      // テナント分離違反: 不正アクセスを禁止する
      throw new Error(
        `InMemorySecretStore.withSecret: tenant mismatch secretId=${secretId} expected=${entry.metadata.tenantId} actual=${tenantId}`
      );
    }
    // isActive が false の場合はエラーを投げる（revoke 後のアクセス禁止）
    if (!entry.metadata.isActive) {
      throw new Error(`InMemorySecretStore.withSecret: secret is revoked secretId=${secretId}`);
    }
    // シークレットのバイト列をコピーして callback に渡す（ゼロクリアのため）
    const tempBytes = new Uint8Array(entry.secretBytes.length);
    // コピーする
    tempBytes.set(entry.secretBytes);
    // callback を呼び出す（try-finally でゼロクリアを保証する）
    try {
      // callback を呼び出して結果を返す
      return callback(Object.freeze(tempBytes));
    } finally {
      // callback 後にゼロクリアする（05_鍵管理適合仕様 §メモリゼロクリア規律）
      zeroize(tempBytes);
    }
  }

  /**
   * rotate はシークレットを新しいバージョンにローテーションする。
   * 返した SecretMetadata に新しい version が含まれる。
   * in-memory 実装ではバージョン番号をインクリメントするのみ。
   */
  // rotate メソッド実装: バージョン番号をインクリメントして返す
  async rotate(secretId: string, tenantId: string): Promise<SecretMetadata> {
    // secretId でエントリを検索する
    const entry = this.#entries.get(secretId);
    // エントリが存在しない場合はエラーを投げる
    if (entry === undefined) {
      throw new Error(`InMemorySecretStore.rotate: secret not found secretId=${secretId}`);
    }
    // テナント不一致の場合はエラーを投げる（tenant 分離必須）
    if (entry.metadata.tenantId !== tenantId) {
      throw new Error(
        `InMemorySecretStore.rotate: tenant mismatch secretId=${secretId} expected=${entry.metadata.tenantId} actual=${tenantId}`
      );
    }
    // バージョンをインクリメントした新しいメタデータを設定する
    const newMetadata: SecretMetadata = {
      ...entry.metadata,
      // バージョンをインクリメントする
      version: entry.metadata.version + 1,
    };
    // エントリのメタデータを更新する
    (entry as { metadata: SecretMetadata }).metadata = newMetadata;
    // 更新したメタデータのコピーを返す
    return { ...newMetadata };
  }

  /**
   * revoke はシークレットを無効化する（isActive=false にする）。
   * revoke 後の withSecret 呼び出しは例外を投げる。
   */
  // revoke メソッド実装: isActive を false にしてシークレットバイト列をゼロクリアする
  async revoke(secretId: string, tenantId: string): Promise<void> {
    // secretId でエントリを検索する
    const entry = this.#entries.get(secretId);
    // エントリが存在しない場合はエラーを投げる
    if (entry === undefined) {
      throw new Error(`InMemorySecretStore.revoke: secret not found secretId=${secretId}`);
    }
    // テナント不一致の場合はエラーを投げる（tenant 分離必須）
    if (entry.metadata.tenantId !== tenantId) {
      throw new Error(
        `InMemorySecretStore.revoke: tenant mismatch secretId=${secretId} expected=${entry.metadata.tenantId} actual=${tenantId}`
      );
    }
    // isActive を false に設定した新しいメタデータを構築する
    const newMetadata: SecretMetadata = {
      ...entry.metadata,
      // 無効化する
      isActive: false,
    };
    // エントリのメタデータを更新する
    (entry as { metadata: SecretMetadata }).metadata = newMetadata;
    // シークレットのバイト列をゼロクリアする（revoke 後は使用不可）
    zeroize(entry.secretBytes);
  }
}
