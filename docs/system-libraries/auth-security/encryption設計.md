# k1s0-encryption ライブラリ設計

AES-GCM 対称暗号・RSA 非対称暗号の暗号化/復号化ユーティリティライブラリ。

## 概要

Vault から取得したキーを使った AES-256-GCM 対称暗号化と RSA-OAEP 非対称暗号化、および Argon2id / PBKDF2 によるパスワードハッシュ化を提供する。PII データの保護や設定値の暗号化保管に利用する。

zeroize クレートによりメモリ上の秘密鍵・平文データを確実にゼロ消去し、メモリダンプ攻撃に対する耐性を高める。k1s0-vault ライブラリと組み合わせて、キー管理と暗号化処理を分離するアーキテクチャを想定している。

**配置先**: `regions/system/library/rust/encryption/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `AesGcmCipher` | 構造体 | AES-256-GCM 暗号化・復号化 |
| `RsaCipher` | 構造体 | RSA-OAEP 暗号化・復号化（公開鍵/秘密鍵）|
| `Hasher` | トレイト | ハッシュ化インターフェース |
| `Argon2Hasher` | 構造体 | Argon2id ハッシュ化（パスワード保管用）|
| `encrypt` | 関数 | AES-GCM で平文を暗号化し Base64 返却 |
| `decrypt` | 関数 | Base64 暗号文を AES-GCM で復号 |
| `hash_password` | 関数 | Argon2id でパスワードハッシュ化 |
| `verify_password` | 関数 | パスワードとハッシュ値を検証 |
| `EncryptionError` | enum | `InvalidKey`・`DecryptionFailed`・`HashingFailed` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-encryption"
version = "0.1.0"
edition = "2021"

[dependencies]
aes-gcm = "0.10"
rsa = { version = "0.9", features = ["pem"] }
argon2 = "0.5"
base64 = "0.22"
rand = "0.8"
thiserror = "2"
zeroize = { version = "1", features = ["derive"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-encryption = { path = "../../system/library/rust/encryption" }
```

**モジュール構成**:

```
encryption/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── aes.rs          # AesGcmCipher
│   ├── rsa.rs          # RsaCipher
│   ├── hash.rs         # Hasher・Argon2Hasher
│   └── error.rs        # EncryptionError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_encryption::{AesGcmCipher, Argon2Hasher, Hasher, encrypt, decrypt, hash_password, verify_password};

// AES-256-GCM 対称暗号化
let key_bytes = [0u8; 32]; // Vault から取得したキー
let cipher = AesGcmCipher::new(&key_bytes).unwrap();

let plaintext = b"sensitive data";
let ciphertext_b64 = cipher.encrypt(plaintext).unwrap();
let decrypted = cipher.decrypt(&ciphertext_b64).unwrap();
assert_eq!(decrypted, plaintext);

// 便利関数による暗号化
let encrypted = encrypt(&key_bytes, "my secret value").unwrap();
let original = decrypt(&key_bytes, &encrypted).unwrap();

// Argon2id パスワードハッシュ化
let hash = hash_password("user-password").unwrap();
let valid = verify_password("user-password", &hash).unwrap();
assert!(valid);
```

## Go 実装

**配置先**: `regions/system/library/go/encryption/`

```
encryption/
├── encryption.go
├── encryption_test.go
├── go.mod
└── go.sum
```

**依存関係**: `golang.org/x/crypto v0.31.0`

**主要関数**:

```go
// 32バイトランダムキー生成
func GenerateKey() ([]byte, error)

// AES-256-GCM 暗号化（Base64 返却）
func Encrypt(key, plaintext []byte) (string, error)

// AES-256-GCM 復号化（Base64 入力）
func Decrypt(key []byte, ciphertext string) ([]byte, error)

// Argon2id パスワードハッシュ化
func HashPassword(password string) (string, error)

// Argon2id パスワード検証（不一致時は error 返却）
func VerifyPassword(password, encodedHash string) error
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/encryption/`

```
encryption/
├── package.json        # "@k1s0/encryption", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # AesGcmCipher, RsaCipher, Hasher, EncryptionError
└── __tests__/
    └── encryption.test.ts
```

**主要 API**:

```typescript
export interface Cipher {
  encrypt(plaintext: Uint8Array | string): Promise<string>;
  decrypt(ciphertext: string): Promise<Uint8Array>;
}

export interface Hasher {
  hash(password: string): Promise<string>;
  verify(password: string, hash: string): Promise<boolean>;
}

export class AesGcmCipher implements Cipher {
  constructor(key: Uint8Array);
  encrypt(plaintext: Uint8Array | string): Promise<string>;
  decrypt(ciphertext: string): Promise<Uint8Array>;
}

export class RsaCipher implements Cipher {
  static fromPem(publicKeyPem: string, privateKeyPem?: string): RsaCipher;
  encrypt(plaintext: Uint8Array | string): Promise<string>;
  decrypt(ciphertext: string): Promise<Uint8Array>;
}

export async function hashPassword(password: string): Promise<string>;
export async function verifyPassword(password: string, hash: string): Promise<boolean>;
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/encryption/`

```
encryption/
├── pubspec.yaml        # k1s0_encryption
├── analysis_options.yaml
├── lib/
│   ├── encryption.dart
│   └── src/
│       ├── aes.dart        # AesGcmCipher
│       ├── rsa.dart        # RsaCipher
│       ├── hash.dart       # Hasher・Argon2Hasher
│       └── error.dart      # EncryptionError
└── test/
    └── encryption_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  pointycastle: ^3.9.1
  convert: ^3.1.2
```

**使用例**:

```dart
import 'package:k1s0_encryption/encryption.dart';

// AES-256-GCM 暗号化
final key = Uint8List(32); // Vault から取得したキー
final cipher = AesGcmCipher(key);

final ciphertext = await cipher.encrypt(utf8.encode('sensitive data'));
final decrypted = await cipher.decrypt(ciphertext);

// パスワードハッシュ化
final hash = await hashPassword('user-password');
final valid = await verifyPassword('user-password', hash);
```

**カバレッジ目標**: 90%以上

## テスト戦略

**ユニットテスト** (`#[cfg(test)]`):
- 既知平文・暗号文ペアによる暗号化・復号化の正確性検証
- 改ざんされた暗号文の復号失敗（`DecryptionFailed` エラー）を確認
- 不正なキー長（31バイト・33バイト）での `InvalidKey` エラーを確認
- `verify_password` で誤ったパスワードが `false` を返すことを確認

**統合テスト**:
- Vault サーバー（wiremock または testcontainers）との連携テスト
- ラウンドトリップ（暗号化→復号化）テスト
- RSA 公開鍵/秘密鍵ペア生成と暗号化・復号化のエンドツーエンドテスト

**プロパティテスト**:
- proptest クレートによる任意バイト列の暗号化・復号化ラウンドトリップ検証

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key = [0u8; 32];
        let cipher = AesGcmCipher::new(&key).unwrap();
        let plaintext = b"hello, encryption";
        let ciphertext = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = [0u8; 32];
        let cipher = AesGcmCipher::new(&key).unwrap();
        let mut ciphertext = cipher.encrypt(b"data").unwrap();
        ciphertext.push('X');
        let result = cipher.decrypt(&ciphertext);
        assert!(matches!(result, Err(EncryptionError::DecryptionFailed)));
    }

    #[test]
    fn test_password_hash_verify() {
        let hash = hash_password("secret").unwrap();
        assert!(verify_password("secret", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }
}
```

## 関連ドキュメント

- [system-library-概要](../overview/概要.md) — ライブラリ一覧・テスト方針
- [system-vault-server設計](../../system-servers/vault/server設計.md) — キー管理サーバー（暗号化キーの供給元）
- [system-library-cache設計](../data/cache設計.md) — k1s0-cache ライブラリ
