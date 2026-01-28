# k1s0-config

## 目的

環境変数を使用せず、YAML ファイルと secrets ファイルから設定を読み込むライブラリ。

## 設計方針

- 環境変数は使用しない（CLI 引数で参照先を指定）
- 機密情報は YAML に直接書かず、`*_file` キーでファイルパスを参照
- `--secrets-dir` で secrets ファイルの配置先を指定

## 起動引数

| 引数 | 短縮 | 説明 | デフォルト |
|------|-----|------|-----------|
| `--env` | `-e` | 環境名（必須: dev, stg, prod） | - |
| `--config` | `-c` | 設定ファイルのパス | `{config_dir}/{env}.yaml` |
| `--config-dir` | - | 設定ファイルのディレクトリ | `/etc/k1s0/config` |
| `--secrets-dir` | `-s` | secrets ディレクトリ | `/var/run/secrets/k1s0` |

## 優先順位

1. CLI 引数（参照先指定に限定）
2. YAML（`config/{env}.yaml`。非機密の静的設定）
3. DB（`fw_m_setting`。feature 固有の動的設定）※ `db` feature で有効化

## 主要な型

### ConfigOptions

```rust
pub struct ConfigOptions {
    pub env: String,
    pub config_path: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub secrets_dir: Option<PathBuf>,
}

impl ConfigOptions {
    pub fn new(env: impl Into<String>) -> Self;
    pub fn with_config_path(self, path: impl Into<PathBuf>) -> Self;
    pub fn with_secrets_dir(self, dir: impl Into<PathBuf>) -> Self;
}
```

### ConfigLoader

```rust
pub struct ConfigLoader {
    options: ConfigOptions,
}

impl ConfigLoader {
    pub fn new(options: ConfigOptions) -> Result<Self>;
    pub fn load<T: DeserializeOwned>(&self) -> Result<T>;
    pub fn resolve_secret_file(&self, path: &str) -> Result<String>;
}
```

### ServiceInit

```rust
pub struct ServiceInit {
    env: String,
    config_dir: PathBuf,
    secrets_dir: PathBuf,
}

impl ServiceInit {
    pub fn from_args(args: &ServiceArgs) -> Result<Self>;
    pub fn load_config<T: DeserializeOwned>(&self) -> Result<T>;
    pub fn is_production(&self) -> bool;
    pub fn env(&self) -> &str;
}
```

## 使用例

```rust
use k1s0_config::{ConfigLoader, ConfigOptions};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    db: DbConfig,
}

#[derive(Debug, Deserialize)]
struct DbConfig {
    host: String,
    port: u16,
    password_file: String,
}

let options = ConfigOptions::new("dev")
    .with_config_path("config/dev.yaml")
    .with_secrets_dir("/var/run/secrets/k1s0");

let loader = ConfigLoader::new(options)?;
let config: AppConfig = loader.load()?;

// *_file キーの値をファイルから読み込む
let password = loader.resolve_secret_file(&config.db.password_file)?;
```

## DB 設定機能（`db` feature）

`db` feature を有効化すると、`fw_m_setting` テーブルからの動的設定取得が可能になる。

### 有効化方法

```toml
[dependencies]
k1s0-config = { path = "...", features = ["db"] }
```

### 主要な型

#### DbSettingRepository

```rust
/// DB設定リポジトリ（トレイト）
#[async_trait]
pub trait DbSettingRepository: Send + Sync {
    /// 全ての設定を取得
    async fn get_all(&self) -> Result<Vec<SettingEntry>, DbSettingError>;
    /// キーで設定を取得
    async fn get(&self, key: &str) -> Result<Option<SettingEntry>, DbSettingError>;
    /// プレフィックスで設定を取得
    async fn get_by_prefix(&self, prefix: &str) -> Result<Vec<SettingEntry>, DbSettingError>;
    /// ヘルスチェック
    async fn health_check(&self) -> Result<(), DbSettingError>;
}
```

#### SettingEntry

```rust
/// fw_m_setting テーブルの1行
pub struct SettingEntry {
    pub key: String,           // 設定キー（例: `http.timeout_ms`）
    pub value: String,         // 設定値（JSON または単純な値）
    pub updated_at: DateTime<Utc>,
}
```

#### DbConfigLoader

```rust
/// YAML と DB 設定をマージして読み込むローダー
pub struct DbConfigLoader {
    yaml_loader: ConfigLoader,
    db_repo: Box<dyn DbSettingRepository>,
    failure_mode: FailureMode,
}

impl DbConfigLoader {
    pub fn new(yaml_loader: ConfigLoader, db_repo: Box<dyn DbSettingRepository>) -> Self;
    pub fn with_failure_mode(self, mode: FailureMode) -> Self;
    pub async fn load<T: DeserializeOwned>(&self) -> ConfigResult<T>;
    pub async fn clear_cache(&self);
    pub async fn refresh_cache(&self) -> ConfigResult<()>;
}
```

#### FailureMode

```rust
/// DB 設定取得失敗時の挙動
pub enum FailureMode {
    /// キャッシュがあれば継続（キャッシュなしならエラー）【既定】
    UseCacheOrFail,
    /// フェイルオープン（DB 設定なしでも継続、YAML のみで動作）
    FailOpen,
    /// 起動不可（DB 設定取得が必須）
    FailClosed,
}
```

### 使用例（DB 設定との統合）

```rust
use k1s0_config::{ConfigLoader, ConfigOptions, DbConfigLoader, FailureMode};
use k1s0_db::PostgresSettingRepository;  // k1s0-db が提供する実装
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct AppConfig {
    http: HttpConfig,
    db: DbConfig,
}

#[derive(Debug, Deserialize)]
struct HttpConfig {
    timeout_ms: u64,
    max_connections: u32,
}

#[derive(Debug, Deserialize)]
struct DbConfig {
    pool_size: u32,
}

// YAML 設定ローダーを作成
let yaml_loader = ConfigLoader::new(
    ConfigOptions::new("dev").with_config_path("config/dev.yaml")
)?;

// PostgreSQL プールから設定リポジトリを作成（k1s0-db が提供）
let setting_repo = PostgresSettingRepository::new(Arc::clone(&pool));

// DB 設定ローダーを作成
let loader = DbConfigLoader::new(yaml_loader, Box::new(setting_repo))
    .with_failure_mode(FailureMode::UseCacheOrFail);

// 設定を読み込み（YAML が優先、DB はフォールバック）
let config: AppConfig = loader.load().await?;
```

### Tier 依存関係

- **k1s0-config（Tier 1）**: `DbSettingRepository` トレイトと `DbConfigLoader` を定義
- **k1s0-db（Tier 2）**: `PostgresSettingRepository` 実装を提供（`config` feature で有効化）

この設計により、Tier 1 の k1s0-config は具体的な DB 実装に依存せず、Tier 依存ルールを維持している。
