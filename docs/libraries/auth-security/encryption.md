# k1s0-encryption ライブラリ設計

AES-GCM 対称暗号・RSA 非対称暗号の暗号化/復号化ユーティリティライブラリ。

## 概要

Vault から取得したキーを使った AES-256-GCM 対称暗号化と RSA-OAEP 非対称暗号化、および Argon2id / PBKDF2 によるパスワードハッシュ化を提供する。PII データの保護や設定値の暗号化保管に利用する。

zeroize クレートによりメモリ上の秘密鍵・平文データを確実にゼロ消去し、メモリダンプ攻撃に対する耐性を高める。k1s0-vault ライブラリと組み合わせて、キー管理と暗号化処理を分離するアーキテクチャを想定している。

**配置先**: `regions/system/library/rust/encryption/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `generate_aes_key` | 関数 | 32バイトの AES-256 キーをランダム生成 |
| `aes_encrypt` | 関数 | AES-256-GCM で平文を暗号化し Base64 返却（Rust: `aes_encrypt(key: &[u8; 32], plaintext: &[u8])`）|
| `aes_decrypt` | 関数 | Base64 暗号文を AES-256-GCM で復号（Rust: `aes_decrypt(key: &[u8; 32], ciphertext: &str)`）|
| `hash_password` | 関数 | Argon2id でパスワードをハッシュ化 |
| `verify_password` | 関数 | パスワードとハッシュ値を検証 |
| `EncryptionError` | enum | `EncryptFailed(String)`・`DecryptFailed(String)`・`HashFailed(String)`・`RsaKeyGenerationFailed(String)`・`RsaEncryptFailed(String)`・`RsaDecryptFailed(String)` |
| `generate_rsa_key_pair` | 関数 | 2048bit RSA キーペアを PEM 形式で生成（戻り値: `(public_pem, private_pem)`）|
| `rsa_encrypt` | 関数 | RSA-OAEP-SHA256 で平文を暗号化（引数: `public_key_pem, plaintext`）|
| `rsa_decrypt` | 関数 | RSA-OAEP-SHA256 で暗号文を復号（引数: `private_key_pem, ciphertext`）|

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

**依存追加**: `k1s0-encryption = { path = "../../system/library/rust/encryption" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
encryption/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── aes.rs          # generate_aes_key・aes_encrypt・aes_decrypt
│   ├── rsa.rs          # RsaCipher（TODO: 未実装）
│   ├── hash.rs         # hash_password・verify_password（Argon2id）
│   └── error.rs        # EncryptionError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_encryption::{
    aes_encrypt, aes_decrypt, generate_aes_key,
    generate_rsa_key_pair, rsa_encrypt, rsa_decrypt,
    hash_password, verify_password,
};

// AES-256-GCM 対称暗号化
let key: [u8; 32] = generate_aes_key(); // Vault から取得したキーを使用する場合は [u8;32] に変換

let plaintext = b"sensitive data";
let ciphertext_b64 = aes_encrypt(&key, plaintext).unwrap();
let decrypted = aes_decrypt(&key, &ciphertext_b64).unwrap();
assert_eq!(decrypted, plaintext);

// Argon2id パスワードハッシュ化
let hash = hash_password("user-password").unwrap();
let valid = verify_password("user-password", &hash).unwrap();
assert!(valid);

// RSA-OAEP 非対称暗号化（2048bit、PEM形式）
let (pub_pem, priv_pem) = generate_rsa_key_pair().unwrap();
let ciphertext = rsa_encrypt(&pub_pem, b"sensitive data").unwrap();
let decrypted = rsa_decrypt(&priv_pem, &ciphertext).unwrap();
assert_eq!(decrypted, b"sensitive data");
```

## Go 実装

**配置先**: `regions/system/library/go/encryption/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**配置先**: `regions/system/library/typescript/encryption/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
// 関数ベース API
export function generateKey(): Buffer;

// AES-256-GCM 対称暗号化（同期）
export function encrypt(key: Buffer, plaintext: string): string;
export function decrypt(key: Buffer, ciphertext: string): string;

// Argon2id パスワードハッシュ化（非同期）
export async function hashPassword(password: string): Promise<string>;
export async function verifyPassword(password: string, hash: string): Promise<boolean>;

// RSA-OAEP-SHA256 非対称暗号化（2048bit、PEM形式）
export function generateRsaKeyPair(): { publicKey: string; privateKey: string };
export function rsaEncrypt(publicKeyPem: string, plaintext: Buffer): Buffer;
export function rsaDecrypt(privateKeyPem: string, ciphertext: Buffer): Buffer;
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/encryption/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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
final ciphertext = encrypt(key, 'sensitive data');
final decrypted = decrypt(key, ciphertext);

// パスワードハッシュ化
final hash = await hashPassword('user-password');
final valid = await verifyPassword('user-password', hash);
```

**カバレッジ目標**: 90%以上

## テスト戦略

**ユニットテスト** (`#[cfg(test)]`):
- 既知平文・暗号文ペアによる暗号化・復号化の正確性検証
- 改ざんされた暗号文の復号失敗（`DecryptFailed` エラー）を確認
- 不正な入力での `EncryptFailed` エラーを確認
- `verify_password` で誤ったパスワードが `false` を返すことを確認

**統合テスト**:
- Vault サーバー（wiremock または testcontainers）との連携テスト
- ラウンドトリップ（暗号化→復号化）テスト
- （RSA テストは RsaCipher 実装後に追加予定）

**プロパティテスト**:
- proptest クレートによる任意バイト列の暗号化・復号化ラウンドトリップ検証

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key = [0u8; 32];
        let plaintext = b"hello, encryption";
        let ciphertext = aes_encrypt(&key, plaintext).unwrap();
        let decrypted = aes_decrypt(&key, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = [0u8; 32];
        let mut ciphertext = aes_encrypt(&key, b"data").unwrap();
        ciphertext.push('X');
        let result = aes_decrypt(&key, &ciphertext);
        assert!(matches!(result, Err(EncryptionError::DecryptFailed(_))));
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

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-vault-server設計](../../servers/vault/server.md) — キー管理サーバー（暗号化キーの供給元）
- [system-library-cache設計](../data/cache.md) — k1s0-cache ライブラリ
