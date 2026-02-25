# k1s0-file-client ライブラリ設計

## 概要

ファイルストレージ抽象化クライアントライブラリ。`FileClient` トレイトにより S3/GCS/Ceph のマルチバックエンドに対して統一インターフェースを提供する。プリサインドURL生成（アップロード・ダウンロード）、マルチパートアップロード（大容量ファイル）、ファイルメタデータ取得・削除・一覧・コピーをサポートする。

file-server が存在する場合は file-server 経由で操作を委譲し、存在しない場合は直接 S3 互換 API を呼び出すデュアルモード設計。バックエンドはプロバイダー設定の切り替えのみで変更でき、アプリケーションコードに影響しない。

**配置先**: `regions/system/library/rust/file-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `FileClient` | トレイト | ファイルストレージ操作の抽象インターフェース |
| `S3FileClient` | 構造体 | AWS S3 / GCS / Ceph 直接実装（aws-sdk-s3 使用） |
| `ServerFileClient` | 構造体 | file-server 経由実装（HTTP クライアント使用） |
| `MockFileClient` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `FileClientConfig` | 構造体 | バックエンド設定・エンドポイント・認証情報 |
| `FileMetadata` | 構造体 | ファイルパス・サイズ・コンテンツタイプ・ETag・更新日時・タグ |
| `PresignedUrl` | 構造体 | プリサインドURL・HTTPメソッド・有効期限・追加ヘッダー |
| `MultipartUpload` | 構造体 | マルチパートアップロードセッション管理 |
| `FileClientError` | enum | 接続エラー・認証エラー・NotFound・クォータ超過等 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-file-client"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]
server-mode = []      # file-server 経由モード（デフォルト有効）
direct-mode = ["aws-sdk-s3"]  # 直接 S3 API モード

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
reqwest = { version = "0.12", features = ["json", "multipart"] }
aws-sdk-s3 = { version = "1", optional = true }
aws-config = { version = "1", optional = true }
chrono = { version = "0.4", features = ["serde"] }
bytes = "1"
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
testcontainers = "0.23"
wiremock = "0.6"
```

**Cargo.toml への追加行**:

```toml
k1s0-file-client = { path = "../../system/library/rust/file-client" }
# モックを有効化する場合:
k1s0-file-client = { path = "../../system/library/rust/file-client", features = ["mock"] }
# 直接 S3 API モードを有効化する場合:
k1s0-file-client = { path = "../../system/library/rust/file-client", features = ["direct-mode"] }
```

**モジュール構成**:

```
file-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # FileClient トレイト・ServerFileClient・MockFileClient
│   ├── s3.rs           # S3FileClient（aws-sdk-s3 使用）
│   ├── config.rs       # FileClientConfig（バックエンド・認証設定）
│   ├── multipart.rs    # MultipartUpload セッション管理
│   ├── model.rs        # FileMetadata・PresignedUrl
│   └── error.rs        # FileClientError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_file_client::{FileClient, FileClientConfig, ServerFileClient};
use std::time::Duration;

// file-server 経由モード
let config = FileClientConfig::server_mode("http://file-server:8080")
    .with_timeout(Duration::from_secs(30));

let client = ServerFileClient::new(config).await.unwrap();

// プリサインドアップロード URL の生成
let upload_url = client
    .generate_upload_url("uploads/image.png", "image/png", Duration::from_secs(3600))
    .await
    .unwrap();
println!("Upload to: {}", upload_url.url);

// プリサインドダウンロード URL の生成
let download_url = client
    .generate_download_url("uploads/image.png", Duration::from_secs(300))
    .await
    .unwrap();

// メタデータ取得
let meta = client.get_metadata("uploads/image.png").await.unwrap();
println!("Size: {} bytes, ETag: {}", meta.size_bytes, meta.etag);

// 一覧取得
let files = client.list("uploads/").await.unwrap();
for f in &files {
    println!("{}: {} bytes", f.path, f.size_bytes);
}

// ファイルコピー
client.copy("uploads/image.png", "archive/image.png").await.unwrap();

// 削除
client.delete("uploads/image.png").await.unwrap();
```

## Go 実装

**配置先**: `regions/system/library/go/file-client/`

```
file-client/
├── file_client.go
├── client.go
├── s3.go
├── multipart.go
├── model.go
├── file_client_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/aws/aws-sdk-go-v2 v1.32.0`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type FileClient interface {
    GenerateUploadURL(ctx context.Context, path, contentType string, expiresIn time.Duration) (*PresignedURL, error)
    GenerateDownloadURL(ctx context.Context, path string, expiresIn time.Duration) (*PresignedURL, error)
    Delete(ctx context.Context, path string) error
    GetMetadata(ctx context.Context, path string) (*FileMetadata, error)
    List(ctx context.Context, prefix string) ([]*FileMetadata, error)
    Copy(ctx context.Context, src, dst string) error
}

type FileMetadata struct {
    Path        string
    SizeBytes   int64
    ContentType string
    ETag        string
    LastModified time.Time
    Tags        map[string]string
}

type PresignedURL struct {
    URL       string
    Method    string
    ExpiresAt time.Time
    Headers   map[string]string
}

func NewServerFileClient(serverURL string, opts ...Option) FileClient
func NewS3FileClient(cfg aws.Config, bucket string, opts ...Option) FileClient
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/file-client/`

```
file-client/
├── package.json        # "@k1s0/file-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # FileClient, ServerFileClient, S3FileClient, FileMetadata, PresignedUrl, FileClientError
└── __tests__/
    └── file-client.test.ts
```

**主要 API**:

```typescript
export interface FileMetadata {
  path: string;
  sizeBytes: number;
  contentType: string;
  etag: string;
  lastModified: Date;
  tags: Record<string, string>;
}

export interface PresignedUrl {
  url: string;
  method: 'PUT' | 'GET';
  expiresAt: Date;
  headers: Record<string, string>;
}

export interface FileClient {
  generateUploadUrl(path: string, contentType: string, expiresInMs: number): Promise<PresignedUrl>;
  generateDownloadUrl(path: string, expiresInMs: number): Promise<PresignedUrl>;
  delete(path: string): Promise<void>;
  getMetadata(path: string): Promise<FileMetadata>;
  list(prefix: string): Promise<FileMetadata[]>;
  copy(src: string, dst: string): Promise<void>;
}

export interface FileClientConfig {
  serverUrl?: string;      // file-server モード
  s3Endpoint?: string;     // 直接 S3 モード
  bucket?: string;
  region?: string;
  accessKeyId?: string;
  secretAccessKey?: string;
  timeoutMs?: number;
}

export class ServerFileClient implements FileClient {
  constructor(config: FileClientConfig);
  generateUploadUrl(path: string, contentType: string, expiresInMs: number): Promise<PresignedUrl>;
  generateDownloadUrl(path: string, expiresInMs: number): Promise<PresignedUrl>;
  delete(path: string): Promise<void>;
  getMetadata(path: string): Promise<FileMetadata>;
  list(prefix: string): Promise<FileMetadata[]>;
  copy(src: string, dst: string): Promise<void>;
}

export class FileClientError extends Error {
  constructor(message: string, public readonly code: string, public readonly cause?: Error);
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/file_client/`

```
file_client/
├── pubspec.yaml        # k1s0_file_client
├── analysis_options.yaml
├── lib/
│   ├── file_client.dart
│   └── src/
│       ├── client.dart       # FileClient abstract, ServerFileClient, S3FileClient
│       ├── config.dart       # FileClientConfig
│       ├── multipart.dart    # MultipartUpload
│       ├── model.dart        # FileMetadata, PresignedUrl
│       └── error.dart        # FileClientError
└── test/
    └── file_client_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  http: ^1.2.0
  aws_s3_api: ^2.0.0
  meta: ^1.14.0
```

**主要インターフェース**:

```dart
abstract class FileClient {
  Future<PresignedUrl> generateUploadUrl(
    String path,
    String contentType,
    Duration expiresIn,
  );
  Future<PresignedUrl> generateDownloadUrl(String path, Duration expiresIn);
  Future<void> delete(String path);
  Future<FileMetadata> getMetadata(String path);
  Future<List<FileMetadata>> list(String prefix);
  Future<void> copy(String src, String dst);
}

class FileMetadata {
  final String path;
  final int sizeBytes;
  final String contentType;
  final String etag;
  final DateTime lastModified;
  final Map<String, String> tags;
}
```

**カバレッジ目標**: 85%以上

## Python 実装

**配置先**: `regions/system/library/python/file_client/`

### パッケージ構造

```
file_client/
├── pyproject.toml
├── src/
│   └── k1s0_file_client/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── client.py         # FileClient ABC・ServerFileClient・S3FileClient
│       ├── config.py         # FileClientConfig dataclass
│       ├── multipart.py      # MultipartUpload
│       ├── model.py          # FileMetadata・PresignedUrl dataclass
│       ├── exceptions.py     # FileClientError
│       └── py.typed
└── tests/
    ├── test_client.py
    └── test_multipart.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `FileClient` | ABC | ファイルストレージ操作抽象基底クラス |
| `ServerFileClient` | class | file-server 経由実装（httpx 使用） |
| `S3FileClient` | class | 直接 S3 API 実装（boto3 使用） |
| `FileClientConfig` | dataclass | バックエンド設定・認証情報 |
| `FileMetadata` | dataclass | パス・サイズ・コンテンツタイプ・ETag・更新日時・タグ |
| `PresignedUrl` | dataclass | URL・メソッド・有効期限・追加ヘッダー |
| `FileClientError` | Exception | ファイルクライアントエラー基底クラス |

### 使用例

```python
import asyncio
from datetime import timedelta
from k1s0_file_client import ServerFileClient, FileClientConfig

config = FileClientConfig(
    server_url="http://file-server:8080",
    timeout=timedelta(seconds=30),
)
client = ServerFileClient(config)

# プリサインドアップロード URL の生成
upload_url = await client.generate_upload_url(
    "uploads/image.png",
    "image/png",
    timedelta(hours=1),
)
print(f"Upload to: {upload_url.url}")

# メタデータ取得
meta = await client.get_metadata("uploads/image.png")
print(f"Size: {meta.size_bytes} bytes")

# 一覧取得
files = await client.list("uploads/")
for f in files:
    print(f"{f.path}: {f.size_bytes} bytes")

# ファイルコピー
await client.copy("uploads/image.png", "archive/image.png")

# 削除
await client.delete("uploads/image.png")
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| httpx | >=0.27 | 非同期 HTTP クライアント（server-mode） |
| boto3 | >=1.35 | AWS S3 互換クライアント（direct-mode） |
| pydantic | >=2.10 | 設定バリデーション |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock / moto（S3 モック）
- カバレッジ目標: 85%以上
- 実行: `pytest` / `ruff check .`

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | URL生成ロジック・メタデータパース・エラーハンドリング | tokio::test |
| モックテスト | `mockall` による FileClient モック・サーバーレスポンスのモック | mockall (feature = "mock") / wiremock |
| 統合テスト（server-mode） | wiremock による file-server レスポンスシミュレーション | wiremock |
| 統合テスト（direct-mode） | MinIO コンテナによる S3 互換 API テスト | testcontainers + MinIO |
| プロパティテスト | 任意パス文字列でのURL生成・メタデータラウンドトリップ検証 | proptest |

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-encryption設計](system-library-encryption設計.md) — ファイル暗号化との組み合わせ
- [system-library-quota-client設計](system-library-quota-client設計.md) — ストレージクォータ管理
- [system-library-audit-client設計](system-library-audit-client設計.md) — ファイル操作の監査ログ
