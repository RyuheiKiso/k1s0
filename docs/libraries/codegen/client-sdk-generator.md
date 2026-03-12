> **ステータス**: 未実装（設計フェーズ）
> 本ライブラリは現在設計段階です。実装は将来のフェーズで予定されています。
> 検収対象外とし、実装開始時に本ドキュメントのステータスを更新してください。

# Client SDK 自動生成

## 概要

`k1s0-codegen` の `client-sdk` Feature で有効化される Client SDK 生成機能。サービス定義から型安全なクライアントライブラリ (trait + gRPC/HTTP/Mock/Resilient 実装) を一括生成する。

## Feature Flags

```toml
[dependencies]
k1s0-codegen = { path = "...", features = ["client-sdk"] }
```

## API

### ClientSdkConfig

SDK 生成の入力パラメータ。

```rust
pub struct ClientSdkConfig {
    pub service_name: String,   // e.g., "UserProfile"
    pub package_name: String,   // e.g., "k1s0-user-profile-client"
    pub methods: Vec<ClientMethod>,
    pub types: Vec<ClientType>,
}

pub struct ClientMethod {
    pub name: String,           // e.g., "get_user"
    pub request_type: String,   // e.g., "GetUserRequest"
    pub response_type: String,  // e.g., "GetUserResponse"
}

pub struct ClientType {
    pub name: String,
    pub fields: Vec<ClientField>,
}

pub struct ClientField {
    pub name: String,
    pub field_type: String,     // Rust 型 (e.g., "String", "i64")
}
```

### generate_client_sdk

```rust
pub fn generate_client_sdk(
    config: &ClientSdkConfig,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, CodegenError>;
```

指定ディレクトリに SDK の全ファイルを生成。既存ファイルはスキップ（冪等）。

## 生成ファイル構造

```
{output_dir}/
  Cargo.toml        -- パッケージ定義 + Feature Flag
  src/
    lib.rs           -- モジュール宣言 + re-export
    client.rs        -- {Service}Client trait (async_trait)
    grpc.rs          -- Grpc{Service}Client (tonic)
    http.rs          -- Http{Service}Client (reqwest)
    mock.rs          -- Mock{Service}Client (mockall)
    resilient.rs     -- Resilient{Service}Client (retry + circuit breaker)
    error.rs         -- ClientError enum
    types.rs         -- リクエスト/レスポンス型 (Serialize + Deserialize)
```

## 各ジェネレーター

### trait_generator -- クライアント trait

`{Service}Client` trait を生成。全実装の共通インターフェース。

```rust
#[async_trait]
pub trait UserProfileClient: Send + Sync {
    async fn get_user(&self, request: GetUserRequest) -> Result<GetUserResponse, ClientError>;
    async fn create_user(&self, request: CreateUserRequest) -> Result<CreateUserResponse, ClientError>;
}
```

### grpc_generator -- gRPC 実装

`tonic::transport::Channel` を使った gRPC クライアント。Feature: `grpc` (default)。

### http_generator -- HTTP 実装

`reqwest::Client` を使った REST クライアント。Feature: `http`。メソッド名をエンドポイントパスとして POST リクエストを送信。

### mock_generator -- Mock 実装

`mockall::mock!` を使ったテスト用モッククライアント。Feature: `mock`。

### resilient_generator -- Resilient 実装

任意の `{Service}Client` 実装をラップし、リトライとサーキットブレーカーを追加。Feature: `resilient`。

```rust
pub struct ResilientUserProfileClient<T: UserProfileClient> {
    inner: T,
    max_retries: u32,               // default: 3
    timeout: Duration,               // default: 5s
    circuit_breaker_threshold: u32,  // default: 5
}

impl<T: UserProfileClient> ResilientUserProfileClient<T> {
    pub fn new(inner: T) -> Self;
    pub fn with_max_retries(self, max_retries: u32) -> Self;
    pub fn with_timeout(self, timeout: Duration) -> Self;
    pub fn with_circuit_breaker_threshold(self, threshold: u32) -> Self;
}
```

### 生成される Feature Flag (Cargo.toml)

```toml
[features]
default = ["grpc"]
grpc = ["dep:tonic"]
http = ["dep:reqwest"]
mock = ["dep:mockall"]
resilient = ["dep:tokio"]
```

### ClientError

```rust
pub enum ClientError {
    Transport(String),
    Request { status: u32, message: String },
    Serialization(String),
    Timeout(Duration),
    CircuitBreakerOpen,
}
```

## 使用例

```rust
use k1s0_codegen::client_sdk::{generate_client_sdk, ClientSdkConfig, ClientMethod, ClientType, ClientField};

let config = ClientSdkConfig {
    service_name: "UserProfile".into(),
    package_name: "k1s0-user-profile-client".into(),
    methods: vec![
        ClientMethod {
            name: "get_user".into(),
            request_type: "GetUserRequest".into(),
            response_type: "GetUserResponse".into(),
        },
    ],
    types: vec![
        ClientType {
            name: "GetUserRequest".into(),
            fields: vec![ClientField { name: "user_id".into(), field_type: "String".into() }],
        },
    ],
};

let created = generate_client_sdk(&config, Path::new("output/"))?;
println!("Generated {} files", created.len());
```

生成後のクライアント利用:

```rust
use k1s0_user_profile_client::{UserProfileClient, GetUserRequest};
use k1s0_user_profile_client::grpc::GrpcUserProfileClient;
use k1s0_user_profile_client::resilient::ResilientUserProfileClient;

// gRPC クライアント + Resilient ラッパー
let channel = tonic::transport::Channel::from_static("http://localhost:50051").connect().await?;
let grpc = GrpcUserProfileClient::new(channel);
let client = ResilientUserProfileClient::new(grpc)
    .with_max_retries(5)
    .with_timeout(Duration::from_secs(10));

let response = client.get_user(GetUserRequest { user_id: "123".into() }).await?;
```

## 設計判断

| 判断 | 理由 |
|------|------|
| trait ベースの抽象化 | 実装の差し替え (gRPC/HTTP/Mock) をコンパイル時に保証 |
| Feature Flag で実装を分離 | 不要なトランスポート依存を排除 |
| Resilient をジェネリックに | 任意の実装をラップ可能、デコレーターパターン |
| ファイル単位の冪等性 | 再実行時に既存コードを上書きしない安全性 |
| サーキットブレーカーを AtomicU32 で実装 | ロック不要で高パフォーマンス |
