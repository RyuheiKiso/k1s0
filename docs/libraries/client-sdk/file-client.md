# k1s0-file-client ライブラリ設計

## 概要

ファイルストレージ抽象化クライアントライブラリ。`FileClient` トレイトにより S3/GCS/Ceph のマルチバックエンドに対して統一インターフェースを提供する。プリサインドURL生成（アップロード・ダウンロード）、マルチパートアップロード（大容量ファイル）、ファイルメタデータ取得・削除・一覧・コピーをサポートする。

file-server が存在する場合は file-server 経由で操作を委譲し、存在しない場合は直接 S3 互換 API を呼び出すデュアルモード設計。バックエンドはプロバイダー設定の切り替えのみで変更でき、アプリケーションコードに影響しない。

**配置先**: `regions/system/library/rust/file-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `FileClient` | トレイト/インターフェース/abstract class | ファイルストレージ操作の抽象インターフェース（Rust: trait, Go/TypeScript: interface, Dart: abstract class） |
| `S3FileClient` | 構造体/クラス | AWS S3 / GCS / Ceph 直接実装。**Go/TypeScript/Dart に実装あり**。Rust は未実装 |
| `ServerFileClient` | 構造体/クラス | file-server 経由実装（HTTP クライアント使用） |
| `MockFileClient` | 構造体/クラス | テスト用モック実装（Rust: feature = "mock" で有効、Go/TypeScript/Dart: 実装クラスとして提供） |
| `InMemoryFileClient` | 構造体/クラス | テスト用インメモリ実装（現在の主実装） |
| `FileClientConfig` | 構造体/クラス | バックエンド設定・エンドポイント・認証情報 |
| `FileMetadata` | 構造体/クラス | ファイルパス・サイズ・コンテンツタイプ・ETag・更新日時・タグ |
| `PresignedUrl` | 構造体/クラス | プリサインドURL・HTTPメソッド・有効期限・追加ヘッダー（Go のみ `PresignedURL` と命名、Go の頭字語規約による） |
| `MultipartUpload` | 構造体 | マルチパートアップロードセッション管理 |
| `FileClientError` | enum/クラス | 接続エラー・認証エラー・NotFound・クォータ超過等 |

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

**依存追加**: `k1s0-file-client = { path = "../../system/library/rust/file-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

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

**FileClientConfig フィールド**:

```rust
pub struct FileClientConfig {
    pub server_url: Option<String>,
    pub s3_endpoint: Option<String>,
    pub bucket: Option<String>,
    pub region: Option<String>,
    // access_key_id / secret_access_key は Rust では未実装（Go/TypeScript/Dart に実装あり）
    pub timeout: Duration,   // Option ではなくデフォルト値あり（デフォルト 30 秒）
}

impl FileClientConfig {
    /// file-server 経由モード用コンフィグを生成する（コンストラクタ）
    pub fn server_mode(server_url: impl Into<String>) -> Self;
    /// タイムアウトを設定する（`FileClientConfig` のビルダーメソッド）
    pub fn with_timeout(self, timeout: Duration) -> Self;
}
```

> **コンストラクタ言語別対応**:
> - Rust: `FileClientConfig::server_mode(url)` → `ServerFileClient::new(config)`, `InMemoryFileClient::new(config)`
> - Go: `NewServerFileClient(serverURL, ...opts)`, `NewS3FileClient(endpoint, bucket, region, ...opts)`, `NewInMemoryFileClient()`
> - TypeScript: `new ServerFileClient(config)`, `new S3FileClient(config)`, `new InMemoryFileClient()`
> - Dart: `ServerFileClient(config)`, `InMemoryFileClient()`
>
> **`with_timeout` / `WithTimeout` の実装位置**:
> - Rust: `FileClientConfig` のビルダーメソッド（`config.with_timeout(Duration::from_secs(60))`）
> - Go: コンストラクタのオプション関数 `WithTimeout(d)` を `opts` として渡す
> - TypeScript/Dart: `FileClientConfig` の `timeoutMs` / `timeout` フィールドで直接設定（専用メソッドなし）

**使用例（ServerFileClient — 将来実装予定）**:

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

**InMemoryFileClient（テスト用実装 — 現在の主実装）**:

`InMemoryFileClient` はプロセスメモリ上でファイルを管理するテスト用実装。`ServerFileClient` / `S3FileClient` の実装が完了するまでの主実装としても機能する。

```rust
use k1s0_file_client::{FileClient, FileClientConfig, InMemoryFileClient};
use std::time::Duration;

let config = FileClientConfig::server_mode("http://file-server:8080");
let client = InMemoryFileClient::new(config);

// アップロード URL 生成（インメモリ上でメタデータを記録）
let upload_url = client
    .generate_upload_url("uploads/image.png", "image/png", Duration::from_secs(3600))
    .await
    .unwrap();

// テスト補助 API: 格納済みファイル一覧を取得（テストコードのみで使用）
let stored = client.stored_files().await;
assert_eq!(stored.len(), 1);
```

公開メソッド:
- `InMemoryFileClient::new(config: FileClientConfig) -> Self`
- `InMemoryFileClient::stored_files(&self) -> Vec<FileMetadata>` （テスト補助用、`async`）

**MockFileClient（feature = "mock" 有効時）**:

`feature = "mock"` を有効にすると `mockall` クレートの `#[automock]` マクロにより `FileClient` トレイトの全メソッドがモック化された `MockFileClient` が自動生成される。

```toml
# Cargo.toml（テスト依存）
[dev-dependencies]
k1s0-file-client = { path = "...", features = ["mock"] }
```

```rust
use k1s0_file_client::MockFileClient;

let mut mock = MockFileClient::new();
mock.expect_get_metadata()
    .returning(|path| Ok(FileMetadata { path: path.to_string(), ..Default::default() }));
```

## Go 実装

**配置先**: `regions/system/library/go/file-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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
    Path         string
    SizeBytes    int64
    ContentType  string
    ETag         string            // Go 命名規約により大文字 ETag
    LastModified time.Time
    Tags         map[string]string
}

// PresignedURL — Go の頭字語規約により URL を大文字で表記（他言語は PresignedUrl）
type PresignedURL struct {
    URL       string
    Method    string
    ExpiresAt time.Time
    Headers   map[string]string
}

type FileClientConfig struct {
    ServerURL       string
    S3Endpoint      string
    Bucket          string
    Region          string
    AccessKeyID     string        // S3 アクセスキー
    SecretAccessKey string        // S3 シークレットキー
    Timeout         time.Duration
}

type Option func(*FileClientConfig)

// WithTimeout はコンストラクタオプション関数としてタイムアウトを設定する
func WithTimeout(d time.Duration) Option

func NewServerFileClient(serverURL string, opts ...Option) FileClient
func NewS3FileClient(endpoint, bucket, region string, opts ...Option) FileClient
func NewInMemoryFileClient() *InMemoryFileClient
```

**S3FileClient**:

`S3FileClient` は S3 互換ストレージ（AWS S3 / GCS / Ceph）に直接アクセスする `FileClient` 実装。
プリサインドURL生成（`GenerateUploadURL` / `GenerateDownloadURL`）は骨格実装済み。
`Delete`, `GetMetadata`, `List`, `Copy` は aws-sdk-go-v2 統合が必要なため未実装（エラーを返す）。

```go
client := fileclient.NewS3FileClient(
    "https://s3.amazonaws.com",
    "my-bucket",
    "ap-northeast-1",
    fileclient.WithTimeout(60 * time.Second),
)
```

**InMemoryFileClient（テスト用実装 — 現在の主実装）**:

`InMemoryFileClient` はプロセスメモリ上でファイルを管理するテスト用実装。`NewServerFileClient` / `NewS3FileClient` の実装が完了するまでの主実装としても機能する。

```go
// コンストラクタ
func NewInMemoryFileClient() *InMemoryFileClient

// テスト補助 API: 格納済みファイル一覧を取得（テストコードのみで使用）
func (c *InMemoryFileClient) StoredFiles() []*FileMetadata
```

使用例:

```go
client := fileclient.NewInMemoryFileClient()

url, err := client.GenerateUploadURL(ctx, "uploads/image.png", "image/png", time.Hour)
if err != nil {
    log.Fatal(err)
}

// テストアサーション
stored := client.StoredFiles()
assert.Len(t, stored, 1)
```

**MockFileClient（テスト用モック実装）**:

`MockFileClient` は `FileClient` インターフェースを実装した録再生可能なモック。呼び出し履歴の記録・期待値検証・スタブ応答の注入が可能。

```go
mock := fileclient.NewMockFileClient()

// スタブ応答を設定（第2引数以降が戻り値）
mock.On("GetMetadata", &fileclient.FileMetadata{
    Path:        "uploads/image.png",
    ContentType: "image/png",
}, nil)

// テスト対象を実行
meta, err := mock.GetMetadata(ctx, "uploads/image.png")

// 呼び出しを検証
mock.AssertCalled(t, "GetMetadata", "uploads/image.png")
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/file-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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
  serverUrl?: string;       // file-server モード
  s3Endpoint?: string;      // 直接 S3 モード
  bucket?: string;
  region?: string;
  accessKeyId?: string;     // S3 アクセスキー（Go/TypeScript/Dart に実装あり、Rust は未実装）
  secretAccessKey?: string; // S3 シークレットキー（同上）
  /** リクエストタイムアウト（ミリ秒）。省略可能。デフォルト 30_000 ms。
   *  注記: 他言語（Rust/Go/Dart）では Duration 型だが、TypeScript では number（ms）で省略可能 */
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

/** S3 互換ストレージ直接実装。プリサインドURL生成は動作する。
 *  delete/getMetadata/list/copy は NOT_IMPLEMENTED エラーを返す（AWS SDK 統合が必要）。 */
export class S3FileClient implements FileClient {
  constructor(config: FileClientConfig);
  generateUploadUrl(path: string, contentType: string, expiresInMs: number): Promise<PresignedUrl>;
  generateDownloadUrl(path: string, expiresInMs: number): Promise<PresignedUrl>;
  delete(path: string): Promise<void>;          // throws FileClientError(code='NOT_IMPLEMENTED')
  getMetadata(path: string): Promise<FileMetadata>;  // throws FileClientError(code='NOT_IMPLEMENTED')
  list(prefix: string): Promise<FileMetadata[]>;     // throws FileClientError(code='NOT_IMPLEMENTED')
  copy(src: string, dst: string): Promise<void>;     // throws FileClientError(code='NOT_IMPLEMENTED')
}

export class FileClientError extends Error {
  constructor(message: string, public readonly code: string, public readonly cause?: Error);
}

/** FileClientError のエラーコード一覧（TypeScript）:
 *  - NOT_FOUND          : ファイルが存在しない
 *  - UNAUTHORIZED       : 認証エラー（HTTP 401/403）
 *  - INVALID_CONFIG     : 設定エラー（serverUrl/s3Endpoint 未設定等）
 *  - CONNECTION_ERROR   : ネットワーク接続エラー
 *  - INTERNAL           : サーバー内部エラー（HTTP 5xx 等）
 *  - NOT_IMPLEMENTED    : S3FileClient の未実装メソッドで発生（AWS SDK 統合前）
 *  注記: QUOTA_EXCEEDED は設計上必要だが TypeScript 実装では未対応（Rust では QuotaExceeded として実装済み）
 */
```

**InMemoryFileClient（テスト用実装 — 現在の主実装）**:

`InMemoryFileClient` はプロセスメモリ上でファイルを管理するテスト用実装。`ServerFileClient` の実装が完了するまでの主実装としても機能する。

```typescript
export class InMemoryFileClient implements FileClient {
  generateUploadUrl(path: string, contentType: string, expiresInMs: number): Promise<PresignedUrl>;
  generateDownloadUrl(path: string, expiresInMs: number): Promise<PresignedUrl>;
  delete(path: string): Promise<void>;
  getMetadata(path: string): Promise<FileMetadata>;
  list(prefix: string): Promise<FileMetadata[]>;
  copy(src: string, dst: string): Promise<void>;
  /** テスト補助 API: 格納済みファイル一覧を取得（テストコードのみで使用） */
  getStoredFiles(): FileMetadata[];
}
```

使用例:

```typescript
import { InMemoryFileClient } from 'k1s0-file-client';

const client = new InMemoryFileClient();
await client.generateUploadUrl('uploads/image.png', 'image/png', 3600_000);

// テストアサーション
const stored = client.getStoredFiles();
expect(stored).toHaveLength(1);
```

**MockFileClient（テスト用モック実装）**:

`MockFileClient` は `FileClient` インターフェースを実装したモッククラス。jest 等のテストフレームワークと組み合わせて使用する。

```typescript
import { MockFileClient, FileMetadata } from 'k1s0-file-client';

const mock = new MockFileClient();

// スタブ応答を設定（jest.fn() を直接代入）
mock.getMetadata = jest.fn().mockResolvedValue({
  path: 'uploads/image.png',
  sizeBytes: 1024,
  contentType: 'image/png',
  etag: 'abc123',
  lastModified: new Date(),
  tags: {},
} satisfies FileMetadata);

// 呼び出し検証
expect(mock.getMetadata).toHaveBeenCalledWith('uploads/image.png');
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/file_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  http: ^1.2.0
  aws_s3_api: ^2.0.0
  meta: ^1.14.0
```

**主要インターフェース**:

```dart
// FileClient は abstract class（Dart には interface キーワードが存在しないため）
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

class PresignedUrl {
  final String url;
  final String method;   // 'PUT' または 'GET'
  final DateTime expiresAt;
  final Map<String, String> headers;
}

class FileClientConfig {
  final String? serverUrl;
  final String? s3Endpoint;
  final String? bucket;
  final String? region;
  final String? accessKeyId;     // S3 アクセスキー（Go/TypeScript と共通）
  final String? secretAccessKey; // S3 シークレットキー（Go/TypeScript と共通）
  final Duration timeout;        // デフォルト Duration(seconds: 30)
}

class FileClientError implements Exception {
  final String message;
  final String code;
  // TypeScript と異なり cause フィールドは存在しない
  @override
  String toString() => 'FileClientError($code): $message';
}
```

**InMemoryFileClient（テスト用実装 — 現在の主実装）**:

`InMemoryFileClient` はプロセスメモリ上でファイルを管理するテスト用実装。

```dart
class InMemoryFileClient implements FileClient {
  @override
  Future<PresignedUrl> generateUploadUrl(String path, String contentType, Duration expiresIn);
  @override
  Future<PresignedUrl> generateDownloadUrl(String path, Duration expiresIn);
  @override
  Future<void> delete(String path);
  @override
  Future<FileMetadata> getMetadata(String path);
  @override
  Future<List<FileMetadata>> list(String prefix);
  @override
  Future<void> copy(String src, String dst);
  /// テスト補助 API: 格納済みファイル一覧を取得（テストコードのみで使用）
  List<FileMetadata> get storedFiles;
}
```

**注記**: Dart には S3FileClient の実装は存在しない（`aws_s3_api` 依存は定義されているが未実装）。直接 S3 連携が必要な場合は Go / TypeScript 実装を使用すること。

使用例:

```dart
import 'package:file_client/file_client.dart';

final client = InMemoryFileClient();
await client.generateUploadUrl('uploads/image.png', 'image/png', Duration(hours: 1));

// テストアサーション
expect(client.storedFiles, hasLength(1));
```

**MockFileClient（テスト用モック実装）**:

`MockFileClient` は `FileClient` abstract class を実装したモッククラス。各メソッドはコールバックで動作をオーバーライドできる。

```dart
import 'package:file_client/file_client.dart';

final mock = MockFileClient();

// スタブ応答を設定
mock.onGetMetadata = (path) async => FileMetadata(
  path: path,
  sizeBytes: 1024,
  contentType: 'image/png',
  etag: 'abc123',
  lastModified: DateTime.now(),
  tags: {},
);

final meta = await mock.getMetadata('uploads/image.png');
expect(meta.contentType, 'image/png');
// 呼び出し履歴の確認
expect(mock.calls, contains('getMetadata:uploads/image.png'));
```

**カバレッジ目標**: 85%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | URL生成ロジック・メタデータパース・エラーハンドリング | tokio::test |
| モックテスト | `mockall` による FileClient モック・サーバーレスポンスのモック | mockall (feature = "mock") / wiremock |
| 統合テスト（server-mode） | wiremock による file-server レスポンスシミュレーション | wiremock |
| 統合テスト（direct-mode） | MinIO コンテナによる S3 互換 API テスト | testcontainers + MinIO |
| プロパティテスト | 任意パス文字列でのURL生成・メタデータラウンドトリップ検証 | proptest |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-encryption設計](../auth-security/encryption.md) — ファイル暗号化との組み合わせ
- [system-library-quota-client設計](quota-client.md) — ストレージクォータ管理
- [system-library-audit-client設計](../observability/audit-client.md) — ファイル操作の監査ログ
