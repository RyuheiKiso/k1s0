# system-app-registry-server 実装設計

system-app-registry-server（アプリケーションレジストリサーバー）の Rust 実装仕様。概要・API 定義は [server.md](server.md) を参照。

---

## Rust 実装 (regions/system/server/rust/app-registry/)

### ディレクトリ構成

```
regions/system/server/rust/app-registry/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── app.rs                   # App エンティティ
│   │   │   ├── version.rs               # AppVersion エンティティ
│   │   │   ├── platform.rs              # Platform enum（Windows/Linux/Macos）
│   │   │   ├── download_stat.rs         # DownloadStat エンティティ
│   │   │   └── claims.rs                # JWT Claims
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── app_repository.rs        # AppRepository トレイト
│   │       ├── version_repository.rs    # VersionRepository トレイト
│   │       └── download_stats_repository.rs  # DownloadStatsRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── list_apps.rs                 # アプリ一覧取得
│   │   ├── get_app.rs                   # アプリ詳細取得
│   │   ├── list_versions.rs             # バージョン一覧取得
│   │   ├── create_version.rs            # バージョン作成
│   │   ├── delete_version.rs            # バージョン削除
│   │   ├── get_latest.rs                # 最新バージョン取得
│   │   └── generate_download_url.rs     # ダウンロードURL生成
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── app_handler.rs           # アプリ関連 REST ハンドラー
│   │   │   ├── version_handler.rs       # バージョン関連 REST ハンドラー
│   │   │   ├── download_handler.rs      # ダウンロード関連 REST ハンドラー
│   │   │   └── error.rs                 # エラーレスポンス
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── rbac.rs                  # RBAC ミドルウェア
│   └── infrastructure/
│       ├── mod.rs
│       ├── database.rs                  # DB 接続プール管理
│       ├── s3_client.rs                 # S3/Ceph RGW クライアント
│       └── repository/
│           ├── mod.rs
│           ├── app_postgres.rs          # AppRepository PostgreSQL 実装
│           ├── version_postgres.rs      # VersionRepository PostgreSQL 実装
│           └── download_stats_postgres.rs  # DownloadStatsRepository PostgreSQL 実装
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── Cargo.toml
├── Cargo.lock
└── Dockerfile
```

### Cargo.toml

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
# S3/Ceph RGW
aws-sdk-s3 = "1"
aws-config = { version = "1", features = ["behavior-version-latest"] }
```

> **注**: 本サーバーは gRPC を提供しないため、`tonic` / `tonic-build` / `prost` は不要。`build.rs` も不要。

---

## ユースケース

### ユースケース一覧

| ユースケース | 説明 | 主要リポジトリ |
| --- | --- | --- |
| `ListAppsUseCase` | カテゴリ・検索条件でアプリ一覧を取得 | `AppRepository` |
| `GetAppUseCase` | ID 指定でアプリ詳細を取得 | `AppRepository` |
| `ListVersionsUseCase` | アプリのバージョン一覧を取得 | `VersionRepository` |
| `CreateVersionUseCase` | 新しいバージョンを登録 | `VersionRepository` |
| `DeleteVersionUseCase` | バージョンを削除 | `VersionRepository` |
| `GetLatestUseCase` | プラットフォーム・アーキテクチャ指定で最新バージョンを取得 | `VersionRepository` |
| `GenerateDownloadUrlUseCase` | S3 署名付きURLを生成し、ダウンロード統計を記録 | `VersionRepository`, `DownloadStatsRepository`, `S3Client` |

### ユースケース構造体

```rust
// src/usecase/generate_download_url.rs
pub struct GenerateDownloadUrlUseCase {
    version_repo: Arc<dyn VersionRepository>,
    download_stats_repo: Arc<dyn DownloadStatsRepository>,
    s3_client: Arc<S3Client>,
}

impl GenerateDownloadUrlUseCase {
    pub async fn execute(
        &self,
        app_id: &str,
        version: &str,
        platform: Option<&str>,
        arch: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<DownloadUrlResponse, AppRegistryError>;
}
```

---

## S3 クライアント設計

### S3Client

Ceph RGW 互換の S3 クライアント。署名付きURL生成を担当する。

```rust
// src/infrastructure/s3_client.rs
pub struct S3Client {
    client: aws_sdk_s3::Client,
    bucket: String,
    url_expiry_secs: u64,
}

impl S3Client {
    pub async fn new(config: &S3Config) -> Self {
        let s3_config = aws_config::from_env()
            .endpoint_url(&config.endpoint)
            .load()
            .await;

        let client = aws_sdk_s3::Client::from_conf(
            aws_sdk_s3::config::Builder::from(&s3_config)
                .force_path_style(true)  // Ceph RGW 必須
                .build(),
        );

        Self {
            client,
            bucket: config.bucket.clone(),
            url_expiry_secs: config.url_expiry_secs,
        }
    }

    pub async fn generate_presigned_url(&self, s3_key: &str) -> Result<String, anyhow::Error>;
}
```

### S3 設定

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `endpoint` | string | S3/Ceph RGW エンドポイント URL |
| `bucket` | string | バケット名 |
| `region` | string | リージョン |
| `url_expiry_secs` | int | 署名付きURL有効期限（秒） |
| `force_path_style` | bool | パススタイルアクセス（Ceph RGW 必須: `true`） |

---

## リポジトリ実装

### リポジトリトレイト

```rust
// src/domain/repository/app_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AppRepository: Send + Sync {
    async fn list(&self, category: Option<&str>, search: Option<&str>) -> anyhow::Result<Vec<App>>;
    async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<App>>;
}

// src/domain/repository/version_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait VersionRepository: Send + Sync {
    async fn list_by_app_id(&self, app_id: &str) -> anyhow::Result<Vec<AppVersion>>;
    async fn create(&self, version: &AppVersion) -> anyhow::Result<()>;
    async fn delete(&self, app_id: &str, version: &str) -> anyhow::Result<bool>;
    async fn get_latest(
        &self,
        app_id: &str,
        platform: Option<&str>,
        arch: Option<&str>,
    ) -> anyhow::Result<Option<AppVersion>>;
    async fn get_by_version(
        &self,
        app_id: &str,
        version: &str,
        platform: Option<&str>,
        arch: Option<&str>,
    ) -> anyhow::Result<Option<AppVersion>>;
}

// src/domain/repository/download_stats_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DownloadStatsRepository: Send + Sync {
    async fn record(&self, stat: &DownloadStat) -> anyhow::Result<()>;
}
```

### ドメインエンティティ

```rust
// src/domain/entity/platform.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Linux,
    Macos,
}
```

---

## config.yaml

アプリレジストリサーバー固有の設定セクション。共通セクション（app/server/database/observability）は [Rust共通実装.md](../_common/Rust共通実装.md#共通configyaml) を参照。

```yaml
s3:
  endpoint: "http://ceph-rgw.storage.svc.cluster.local:8080"
  bucket: "app-registry"
  region: "us-east-1"
  url_expiry_secs: 3600
  force_path_style: true
```

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [database.md](database.md) -- データベース設計
- [deploy.md](deploy.md) -- DB マイグレーション・テスト・Dockerfile・Helm values
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
