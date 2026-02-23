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
├── aes.go
├── rsa.go
├── hash.go
├── encryption_test.go
├── go.mod
└── go.sum
```

**依存関係**: `golang.org/x/crypto v0.31.0`

**主要インターフェース**:

```go
type Cipher interface {
    Encrypt(plaintext []byte) (string, error)
    Decrypt(ciphertext string) ([]byte, error)
}

type Hasher interface {
    Hash(password string) (string, error)
    Verify(password, hash string) (bool, error)
}

// AES-256-GCM 暗号化
func NewAesGcmCipher(key []byte) (Cipher, error)

// Argon2id ハッシュ化
func NewArgon2Hasher() Hasher
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

## C# 実装

**配置先**: `regions/system/library/csharp/encryption/`

```
encryption/
├── src/
│   ├── Encryption.csproj
│   ├── IAesCipher.cs           # AES-GCM 暗号化インターフェース
│   ├── AesGcmCipher.cs         # AES-256-GCM 実装
│   ├── IRsaCipher.cs           # RSA 暗号化インターフェース
│   ├── RsaCipher.cs            # RSA-OAEP 実装
│   ├── IHasher.cs              # ハッシュ化インターフェース
│   ├── Argon2Hasher.cs         # Argon2id 実装
│   ├── EncryptionConfig.cs     # 暗号化設定
│   └── EncryptionException.cs  # 公開例外型
├── tests/
│   ├── Encryption.Tests.csproj
│   ├── Unit/
│   │   ├── AesGcmCipherTests.cs
│   │   ├── RsaCipherTests.cs
│   │   └── Argon2HasherTests.cs
│   └── Integration/
│       └── VaultKeyEncryptionTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Konscious.Security.Cryptography.Argon2 1.3 | Argon2id ハッシュ化 |
| BouncyCastle.Cryptography 2.4 | RSA-OAEP 実装 |

**名前空間**: `K1s0.System.Encryption`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IAesCipher` | interface | AES-GCM 暗号化・復号化インターフェース |
| `AesGcmCipher` | class | AES-256-GCM 実装（.NET AesGcm 使用） |
| `IRsaCipher` | interface | RSA 暗号化・復号化インターフェース |
| `RsaCipher` | class | RSA-OAEP 実装（BouncyCastle 使用） |
| `IHasher` | interface | ハッシュ化インターフェース |
| `Argon2Hasher` | class | Argon2id パスワードハッシュ化 |
| `EncryptionException` | class | 暗号化ライブラリの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.Encryption;

public interface IAesCipher
{
    string Encrypt(ReadOnlySpan<byte> plaintext);
    byte[] Decrypt(string ciphertext);
}

public interface IHasher
{
    Task<string> HashAsync(string password, CancellationToken ct = default);
    Task<bool> VerifyAsync(string password, string hash, CancellationToken ct = default);
}

public sealed class AesGcmCipher : IAesCipher, IDisposable
{
    public AesGcmCipher(ReadOnlySpan<byte> key);
    public string Encrypt(ReadOnlySpan<byte> plaintext);
    public byte[] Decrypt(string ciphertext);
    public void Dispose();
}

public sealed class Argon2Hasher : IHasher
{
    public Task<string> HashAsync(string password, CancellationToken ct = default);
    public Task<bool> VerifyAsync(string password, string hash, CancellationToken ct = default);
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0Encryption`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API
```swift
// 暗号化プロトコル
public protocol Cipher: Sendable {
    func encrypt(_ plaintext: Data) throws -> String
    func decrypt(_ ciphertext: String) throws -> Data
}

// AES-256-GCM 暗号化
public struct AesGcmCipher: Cipher, Sendable {
    public init(key: Data) throws
    public func encrypt(_ plaintext: Data) throws -> String
    public func decrypt(_ ciphertext: String) throws -> Data
}

// ハッシュ化プロトコル
public protocol Hasher: Sendable {
    func hash(password: String) async throws -> String
    func verify(password: String, hash: String) async throws -> Bool
}

// 暗号化設定
public struct EncryptionConfig: Sendable {
    public let keySize: Int
    public init(keySize: Int = 32)
}
```

### エラー型
```swift
public enum EncryptionError: Error, Sendable {
    case invalidKey
    case decryptionFailed(underlying: Error)
    case hashingFailed(underlying: Error)
    case invalidBase64
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/encryption/`

### パッケージ構造

```
encryption/
├── pyproject.toml
├── src/
│   └── k1s0_encryption/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── aes.py            # AesGcmCipher
│       ├── rsa.py            # RsaCipher
│       ├── hash.py           # Hasher ABC・Argon2Hasher
│       ├── exceptions.py     # EncryptionError
│       └── py.typed
└── tests/
    ├── test_aes.py
    └── test_hash.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `Cipher` | ABC | 暗号化・復号化抽象基底クラス（`encrypt`, `decrypt`）|
| `AesGcmCipher` | class | AES-256-GCM 実装（cryptography ライブラリ使用） |
| `RsaCipher` | class | RSA-OAEP 実装（cryptography ライブラリ使用） |
| `Hasher` | ABC | ハッシュ化抽象基底クラス（`hash`, `verify`） |
| `Argon2Hasher` | class | Argon2id パスワードハッシュ化 |
| `EncryptionError` | Exception | 暗号化エラー基底クラス |

### 使用例

```python
from k1s0_encryption import AesGcmCipher, Argon2Hasher, hash_password, verify_password

# AES-256-GCM 暗号化
key = bytes(32)  # Vault から取得したキー
cipher = AesGcmCipher(key)

ciphertext = cipher.encrypt(b"sensitive data")
decrypted = cipher.decrypt(ciphertext)

# パスワードハッシュ化
hashed = hash_password("user-password")
valid = verify_password("user-password", hashed)
assert valid
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| cryptography | >=43.0 | AES-GCM・RSA-OAEP 実装 |
| argon2-cffi | >=23.1 | Argon2id ハッシュ化 |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

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

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-vault-server設計](system-vault-server設計.md) — キー管理サーバー（暗号化キーの供給元）
- [system-library-cache設計](system-library-cache設計.md) — k1s0-cache ライブラリ
