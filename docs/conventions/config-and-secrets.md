# 設定と秘密情報の規約

本ドキュメントは、k1s0 における設定（config）と秘密情報（secrets）の取り扱い規約を定義する。

## 1. 基本方針

- **環境変数は使用しない**（アプリ実装での `std::env` / `os.Getenv` / `Environment.GetEnvironmentVariable` / `os.environ` / `System.getenv` / `System.getProperty` / `ProcessBuilder` / `dotenv` / `BuildConfig.` / `Platform.environment` / `fromEnvironment` / `flutter_dotenv` 等は禁止）
- framework 自身の設定は **YAML ファイル**で制御する
- feature 固有の動的設定は **DB（`fw_m_setting`）** で管理する
- 秘密情報は **ファイル参照**のみを保持し、値そのものを YAML/DB に置かない

## 2. 設定の優先順位

```
1. CLI 引数（--config / --env / --secrets-dir）
       ↓ override
2. YAML（config/{env}.yaml）
       ↓ override
3. DB（fw_m_setting）
```

## 3. CLI 引数

サービスは起動時に以下の引数を明示的に受け取る：

| 引数 | 説明 | 例 |
|------|------|-----|
| `--env` | 環境名 | `--env dev` |
| `--config` | 設定ファイルパス | `--config /etc/k1s0/config/dev.yaml` |
| `--secrets-dir` | 秘密情報ディレクトリ | `--secrets-dir /var/run/secrets/k1s0/` |

**暗黙の環境選択は禁止**（必ず明示する）

## 4. YAML 設定ファイル

### 4.1 配置先

```
{service}/config/
├── default.yaml  # 共通デフォルト
├── dev.yaml      # 開発環境
├── stg.yaml      # ステージング環境
└── prod.yaml     # 本番環境
```

### 4.2 YAML に書いてよいもの

- 非機密の静的設定（ホスト名、ポート、タイムアウト等）
- 秘密情報への **参照**（`*_file` キー）

### 4.3 YAML に書いてはいけないもの

- パスワード、API キー、トークン等の **秘密情報そのもの**

### 4.4 キー例

```yaml
db:
  host: localhost
  port: 5432
  name: k1s0_dev
  user: k1s0_user
  password_file: /var/run/secrets/k1s0/db_password  # 値ではなく参照

auth:
  jwt_private_key_file: /var/run/secrets/k1s0/jwt_private_key.pem
  jwt_public_key_file: /var/run/secrets/k1s0/jwt_public_key.pem

http:
  timeout_ms: 5000
  max_connections: 100
```

## 5. 秘密情報の配布

### 5.1 Kubernetes 環境

```yaml
# Pod spec（抜粋）
volumes:
  - name: config
    configMap:
      name: {service}-config
  - name: secrets
    secret:
      secretName: {service}-secrets

containers:
  - name: {service}
    volumeMounts:
      - name: config
        mountPath: /etc/k1s0/config/
      - name: secrets
        mountPath: /var/run/secrets/k1s0/
    args:
      - --env
      - $(ENV)
      - --config
      - /etc/k1s0/config/$(ENV).yaml
      - --secrets-dir
      - /var/run/secrets/k1s0/
```

### 5.2 ローカル開発

```
{service}/
└── secrets/            # .gitignore に含める
    └── dev/
        ├── db_password
        └── jwt_private_key.pem
```

起動例：
```bash
cargo run -- --env dev --config ./config/dev.yaml --secrets-dir ./secrets/dev/
```

## 6. DB 設定（fw_m_setting）

### 6.1 用途

- feature 固有の動的設定
- 実行時に変更可能な設定
- サービス再起動なしで反映が必要な設定

### 6.2 テーブル構造

```sql
CREATE TABLE fw_m_setting (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- プレフィックス検索用インデックス
CREATE INDEX idx_fw_m_setting_key_prefix
ON fw_m_setting (key varchar_pattern_ops);
```

マイグレーション SQL は `k1s0-db` クレートから提供される：

```rust
use k1s0_db::{SETTING_MIGRATION_SQL, SETTING_ROLLBACK_SQL};

// テーブル作成
sqlx::query(SETTING_MIGRATION_SQL).execute(&pool).await?;

// ロールバック
sqlx::query(SETTING_ROLLBACK_SQL).execute(&pool).await?;
```

### 6.3 setting_key 命名規則

```
{category}.{name}
```

- 小文字 + 数字 + アンダースコア
- ドット区切り

例：
- `http.timeout_ms`
- `db.pool_size`
- `auth.jwt_ttl_sec`
- `feature.flag_x`

### 6.4 値の形式

- **単純な値**: 数値や文字列をそのまま格納（例: `5000`, `"my-value"`）
- **JSON 形式**: 複雑な構造は JSON として格納

```sql
-- 数値
INSERT INTO fw_m_setting (key, value) VALUES ('http.timeout_ms', '5000');

-- 文字列（JSON 形式）
INSERT INTO fw_m_setting (key, value) VALUES ('app.name', '"my-service"');

-- オブジェクト
INSERT INTO fw_m_setting (key, value) VALUES ('db.config', '{"host":"localhost","port":5432}');
```

### 6.5 Framework 実装

#### クレート構成

| クレート | Tier | 提供する機能 |
|---------|------|-------------|
| `k1s0-config` | 1 | `DbSettingRepository` トレイト、`DbConfigLoader`、`FailureMode` |
| `k1s0-db` | 2 | `PostgresSettingRepository`、`PostgresSettingWriter` 実装 |

Tier 依存ルールにより、k1s0-config（Tier 1）は具体的な DB 実装に依存しない。

#### 有効化方法

```toml
# Cargo.toml
[dependencies]
k1s0-config = { path = "...", features = ["db"] }
k1s0-db = { path = "...", features = ["postgres", "config"] }
```

### 6.6 使用方法

#### 基本的な使用例

```rust
use k1s0_config::{ConfigLoader, ConfigOptions, DbConfigLoader, FailureMode};
use k1s0_db::PostgresSettingRepository;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct AppConfig {
    http: HttpConfig,
    cache: CacheConfig,
}

#[derive(Debug, Deserialize)]
struct HttpConfig {
    timeout_ms: u64,
    max_connections: u32,
}

#[derive(Debug, Deserialize)]
struct CacheConfig {
    enabled: bool,
    ttl_sec: u64,
}

async fn load_config(pool: Arc<PgPool>) -> Result<AppConfig, ConfigError> {
    // 1. YAML 設定ローダーを作成
    let yaml_loader = ConfigLoader::new(
        ConfigOptions::new("dev").with_config_path("config/dev.yaml")
    )?;

    // 2. PostgreSQL 設定リポジトリを作成
    let setting_repo = PostgresSettingRepository::new(pool)
        .with_cache_ttl(std::time::Duration::from_secs(60));

    // 3. DB 設定ローダーを作成
    let loader = DbConfigLoader::new(yaml_loader, Box::new(setting_repo))
        .with_failure_mode(FailureMode::UseCacheOrFail);

    // 4. 設定を読み込み（YAML が優先、DB はフォールバック）
    loader.load().await
}
```

#### 設定のマージ動作

YAML が優先され、DB はフォールバック（補完）として使用される。

```yaml
# config/dev.yaml
http:
  timeout_ms: 10000  # YAML で指定
cache:
  enabled: true
```

```sql
-- fw_m_setting テーブル
INSERT INTO fw_m_setting (key, value) VALUES ('http.timeout_ms', '5000');  -- YAML が優先されるため無視
INSERT INTO fw_m_setting (key, value) VALUES ('http.max_connections', '100');  -- YAML にないため採用
INSERT INTO fw_m_setting (key, value) VALUES ('cache.ttl_sec', '300');  -- YAML にないため採用
```

結果：
```rust
AppConfig {
    http: HttpConfig {
        timeout_ms: 10000,      // YAML から
        max_connections: 100,   // DB から
    },
    cache: CacheConfig {
        enabled: true,          // YAML から
        ttl_sec: 300,           // DB から
    },
}
```

### 6.7 障害時の挙動（FailureMode）

| モード | 説明 | 用途 |
|--------|------|------|
| `UseCacheOrFail` | キャッシュがあれば継続、なければエラー【既定】 | 一般的なサービス |
| `FailOpen` | DB 失敗時も YAML のみで継続 | 可用性優先のサービス |
| `FailClosed` | DB 設定取得が必須、失敗時はエラー | 設定が必須な機能 |

```rust
// 可用性優先（DB 障害時も起動可能）
let loader = DbConfigLoader::new(yaml_loader, Box::new(repo))
    .with_failure_mode(FailureMode::FailOpen);

// 設定必須（DB 障害時は起動不可）
let loader = DbConfigLoader::new(yaml_loader, Box::new(repo))
    .with_failure_mode(FailureMode::FailClosed);
```

### 6.8 キャッシュ機能

`PostgresSettingRepository` はインメモリキャッシュを内蔵：

```rust
// キャッシュ TTL を設定（デフォルト: 60秒）
let repo = PostgresSettingRepository::new(pool)
    .with_cache_ttl(Duration::from_secs(300));  // 5分

// キャッシュを手動で無効化
repo.invalidate_cache().await;
```

`DbConfigLoader` も独自のキャッシュを持ち、DB 障害時のフォールバックに使用：

```rust
// キャッシュをクリア
loader.clear_cache().await;

// キャッシュを更新
loader.refresh_cache().await?;
```

### 6.9 管理操作（設定の書き込み）

通常のアプリケーションでは読み取りのみを行い、書き込みは管理ツールで行う：

```rust
use k1s0_db::PostgresSettingWriter;

let writer = PostgresSettingWriter::new(pool);

// 単一の設定を挿入/更新
writer.upsert("http.timeout_ms", "10000").await?;

// 設定を削除
writer.delete("http.timeout_ms").await?;

// 複数の設定を一括挿入
writer.bulk_upsert(&[
    ("http.timeout_ms", "10000"),
    ("http.max_connections", "200"),
    ("cache.ttl_sec", "600"),
]).await?;

// プレフィックスで削除
writer.delete_by_prefix("feature.").await?;
```

### 6.10 稼働中の動作

- 取得失敗時は直前のキャッシュを使用（`UseCacheOrFail` の場合）
- 一定時間後にリトライ（キャッシュ TTL 経過後）
- 失敗はメトリクス/ログ/トレースで観測可能

### 6.11 テスト用モック

```rust
use k1s0_config::{MockDbSettingRepository, SettingEntry, DbConfigLoader};

// モックリポジトリを作成
let mock_repo = MockDbSettingRepository::with_entries(vec![
    SettingEntry::new("http.timeout_ms", "5000"),
    SettingEntry::new("http.max_connections", "100"),
]);

// 失敗モードを設定
mock_repo.set_should_fail(true);

// テストで使用
let loader = DbConfigLoader::new(yaml_loader, Box::new(mock_repo));
```

## 7. 禁止事項

| 禁止事項 | 理由 |
|----------|------|
| 環境変数での設定注入 | 監査困難、誤設定リスク |
| `envFrom` / `secretKeyRef` での Secret 注入 | 上記同様 |
| ConfigMap への機密値直書き | Git 等への漏洩リスク |
| 暗黙の環境選択（`--env` 省略） | 意図しない環境での動作リスク |

## 8. 検査（lint）

`k1s0 lint` は以下を検査する：

- `config/{env}.yaml` に機密パターン（password/token/secret/key 等）の値が直書きされていないか
- `*_file` 以外の機密キーがないか

## 関連ドキュメント

- [サービス構成規約](service-structure.md)
- [Framework 設計書](../design/framework.md): k1s0-config、k1s0-db の詳細
- [Tier システム](../architecture/tier-system.md): クレートの依存関係
- [構想.md](../../work/構想.md): 全体方針（11. 設定と秘密情報）
