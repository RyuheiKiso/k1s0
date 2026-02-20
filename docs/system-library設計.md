# system-library設計

system tier が提供する共通ライブラリの設計を定義する。
全ライブラリは Go / Rust / TypeScript / Dart の 4 言語で平等に実装する。

## 概要

| ライブラリ | 用途 | 利用者 |
|-----------|------|--------|
| config | YAML 設定読み込み・環境別オーバーライド・バリデーション | 全サーバー・クライアント |
| telemetry | OpenTelemetry 初期化・構造化ログ・分散トレース・メトリクス | 全サーバー・クライアント |
| authlib | JWT 検証（サーバー用）/ OAuth2 PKCE トークン管理（クライアント用） | 全サーバー・クライアント |
| k1s0-messaging | Kafka イベント発行・購読の抽象化（EventProducer トレイト・EventEnvelope） | 全サーバー（Kafka イベント発行） |
| k1s0-kafka | Kafka 接続設定・管理・ヘルスチェック（KafkaConfig・TLS 対応） | k1s0-messaging を使うサーバー |
| k1s0-correlation | 分散トレーシング用相関 ID・トレース ID 管理（UUID v4・32 文字 hex） | 全サーバー・クライアント |
| k1s0-outbox | トランザクショナルアウトボックスパターン（指数バックオフリトライ） | Kafka 発行を必要とするサーバー |
| k1s0-schemaregistry | Confluent Schema Registry クライアント（Avro/Json/Protobuf 対応） | Kafka プロデューサー・コンシューマー |
| k1s0-serviceauth | サービス間 OAuth2 Client Credentials 認証（トークンキャッシュ・SPIFFE） | サービス間 gRPC/HTTP 通信を行うサーバー |

---

## config ライブラリ

> 詳細なスキーマ定義は [config設計.md](config設計.md) を参照。

### 公開 API（全言語共通契約）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| Load | `(basePath, envPath?) -> Config` | YAML を読み込み Config を返す |
| Validate | `(config) -> Error?` | 設定値のバリデーション |
| MergeVaultSecrets | `(config, secrets) -> Config` | Vault シークレットで上書き |

### Go 実装

**配置先**: `regions/system/library/go/config/`

```
config/
├── config.go          # Config 構造体・Load・Validate
├── merge.go           # YAML マージロジック
├── vault.go           # Vault シークレットオーバーライド
├── config_test.go     # ユニットテスト
├── go.mod
└── go.sum
```

**依存関係**:

```
gopkg.in/yaml.v3       # YAML パース
github.com/go-playground/validator/v10  # バリデーション
```

**主要コード**:

```go
package config

import (
    "fmt"
    "os"

    "github.com/go-playground/validator/v10"
    "gopkg.in/yaml.v3"
)

// Config は config設計.md のスキーマに準拠する。
type Config struct {
    App           AppConfig            `yaml:"app" validate:"required"`
    Server        ServerConfig         `yaml:"server" validate:"required"`
    GRPC          *GRPCConfig          `yaml:"grpc,omitempty"`
    Database      *DatabaseConfig      `yaml:"database,omitempty"`
    Kafka         *KafkaConfig         `yaml:"kafka,omitempty"`
    Redis         *RedisConfig         `yaml:"redis,omitempty"`
    RedisSession  *RedisConfig         `yaml:"redis_session,omitempty"`
    Observability ObservabilityConfig  `yaml:"observability" validate:"required"`
    Auth          AuthConfig           `yaml:"auth" validate:"required"`
}

type AppConfig struct {
    Name        string `yaml:"name" validate:"required"`
    Version     string `yaml:"version" validate:"required"`
    Tier        string `yaml:"tier" validate:"required,oneof=system business service"`
    Environment string `yaml:"environment" validate:"required,oneof=dev staging prod"`
}

type ServerConfig struct {
    Host            string `yaml:"host" validate:"required"`
    Port            int    `yaml:"port" validate:"required,min=1,max=65535"`
    ReadTimeout     string `yaml:"read_timeout"`
    WriteTimeout    string `yaml:"write_timeout"`
    ShutdownTimeout string `yaml:"shutdown_timeout"`
}

type GRPCConfig struct {
    Port           int `yaml:"port" validate:"required,min=1,max=65535"`
    MaxRecvMsgSize int `yaml:"max_recv_msg_size"`
}

type DatabaseConfig struct {
    Host           string `yaml:"host" validate:"required"`
    Port           int    `yaml:"port" validate:"required,min=1,max=65535"`
    Name           string `yaml:"name" validate:"required"`
    User           string `yaml:"user" validate:"required"`
    Password       string `yaml:"password"`
    SSLMode        string `yaml:"ssl_mode" validate:"oneof=disable require verify-full"`
    MaxOpenConns   int    `yaml:"max_open_conns"`
    MaxIdleConns   int    `yaml:"max_idle_conns"`
    ConnMaxLifetime string `yaml:"conn_max_lifetime"`
}

type KafkaConfig struct {
    Brokers          []string         `yaml:"brokers" validate:"required,min=1"`
    ConsumerGroup    string           `yaml:"consumer_group" validate:"required"`
    SecurityProtocol string           `yaml:"security_protocol" validate:"required,oneof=PLAINTEXT SASL_SSL"`
    SASL             *KafkaSASLConfig `yaml:"sasl,omitempty"`
    TLS              *KafkaTLSConfig  `yaml:"tls,omitempty"`
    Topics           KafkaTopics      `yaml:"topics"`
}

type KafkaSASLConfig struct {
    Mechanism string `yaml:"mechanism" validate:"required,oneof=SCRAM-SHA-512 PLAIN"`
    Username  string `yaml:"username"`
    Password  string `yaml:"password"`
}

type KafkaTLSConfig struct {
    CACertPath string `yaml:"ca_cert_path"`
}

type KafkaTopics struct {
    Publish   []string `yaml:"publish"`
    Subscribe []string `yaml:"subscribe"`
}

type RedisConfig struct {
    Host     string `yaml:"host" validate:"required"`
    Port     int    `yaml:"port" validate:"required,min=1,max=65535"`
    Password string `yaml:"password"`
    DB       int    `yaml:"db"`
    PoolSize int    `yaml:"pool_size"`
}

type ObservabilityConfig struct {
    Log     LogConfig     `yaml:"log"`
    Trace   TraceConfig   `yaml:"trace"`
    Metrics MetricsConfig `yaml:"metrics"`
}

type LogConfig struct {
    Level  string `yaml:"level" validate:"oneof=debug info warn error"`
    Format string `yaml:"format" validate:"oneof=json text"`
}

type TraceConfig struct {
    Enabled    bool    `yaml:"enabled"`
    Endpoint   string  `yaml:"endpoint"`
    SampleRate float64 `yaml:"sample_rate" validate:"min=0,max=1"`
}

type MetricsConfig struct {
    Enabled bool   `yaml:"enabled"`
    Path    string `yaml:"path"`
}

type AuthConfig struct {
    JWT  JWTConfig   `yaml:"jwt" validate:"required"`
    OIDC *OIDCConfig `yaml:"oidc,omitempty"`
}

type JWTConfig struct {
    Issuer        string `yaml:"issuer" validate:"required"`
    Audience      string `yaml:"audience" validate:"required"`
    PublicKeyPath string `yaml:"public_key_path"`
}

type OIDCConfig struct {
    DiscoveryURL string   `yaml:"discovery_url" validate:"required,url"`
    ClientID     string   `yaml:"client_id" validate:"required"`
    ClientSecret string   `yaml:"client_secret"`
    RedirectURI  string   `yaml:"redirect_uri" validate:"required,url"`
    Scopes       []string `yaml:"scopes"`
    JWKSURI      string   `yaml:"jwks_uri" validate:"required,url"`
    JWKSCacheTTL string   `yaml:"jwks_cache_ttl"`
}

// Load は basePath の YAML を読み込み、envPath があればマージする。
func Load(basePath string, envPath ...string) (*Config, error) {
    data, err := os.ReadFile(basePath)
    if err != nil {
        return nil, fmt.Errorf("failed to read config: %w", err)
    }

    var cfg Config
    if err := yaml.Unmarshal(data, &cfg); err != nil {
        return nil, fmt.Errorf("failed to parse config: %w", err)
    }

    if len(envPath) > 0 && envPath[0] != "" {
        if err := mergeFromFile(&cfg, envPath[0]); err != nil {
            return nil, fmt.Errorf("failed to merge env config: %w", err)
        }
    }

    return &cfg, nil
}

// Validate は設定値のバリデーションを実行する。
func (c *Config) Validate() error {
    v := validator.New()
    if err := v.Struct(c); err != nil {
        return fmt.Errorf("config validation failed: %w", err)
    }
    return nil
}

// MergeVaultSecrets は Vault から取得したシークレットで設定値を上書きする。
func (c *Config) MergeVaultSecrets(secrets map[string]string) {
    if v, ok := secrets["database.password"]; ok && c.Database != nil {
        c.Database.Password = v
    }
    if v, ok := secrets["redis.password"]; ok && c.Redis != nil {
        c.Redis.Password = v
    }
    if v, ok := secrets["redis_session.password"]; ok && c.RedisSession != nil {
        c.RedisSession.Password = v
    }
    if v, ok := secrets["kafka.sasl.username"]; ok && c.Kafka != nil && c.Kafka.SASL != nil {
        c.Kafka.SASL.Username = v
    }
    if v, ok := secrets["kafka.sasl.password"]; ok && c.Kafka != nil && c.Kafka.SASL != nil {
        c.Kafka.SASL.Password = v
    }
    if v, ok := secrets["auth.oidc.client_secret"]; ok && c.Auth.OIDC != nil {
        c.Auth.OIDC.ClientSecret = v
    }
}
```

**テスト例**:

```go
package config

import (
    "os"
    "path/filepath"
    "testing"

    "github.com/stretchr/testify/assert"
    "github.com/stretchr/testify/require"
)

func TestLoad(t *testing.T) {
    dir := t.TempDir()
    base := filepath.Join(dir, "config.yaml")
    os.WriteFile(base, []byte(`
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
`), 0644)

    cfg, err := Load(base)
    require.NoError(t, err)
    assert.Equal(t, "test-server", cfg.App.Name)
    assert.Equal(t, 8080, cfg.Server.Port)
}

func TestValidate_MissingRequired(t *testing.T) {
    cfg := &Config{}
    err := cfg.Validate()
    assert.Error(t, err)
}

func TestMergeVaultSecrets(t *testing.T) {
    cfg := &Config{
        Database: &DatabaseConfig{Password: ""},
    }
    cfg.MergeVaultSecrets(map[string]string{
        "database.password": "secret123",
    })
    assert.Equal(t, "secret123", cfg.Database.Password)
}
```

### Rust 実装

**配置先**: `regions/system/library/rust/config/`

```
config/
├── src/
│   ├── lib.rs         # 公開 API
│   ├── types.rs       # Config 構造体
│   ├── loader.rs      # YAML 読み込み・マージ
│   ├── validate.rs    # バリデーション
│   └── vault.rs       # Vault シークレットオーバーライド
├── tests/
│   └── integration/
│       └── config_test.rs
└── Cargo.toml
```

**Cargo.toml**:

```toml
[package]
name = "k1s0-config"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
thiserror = "2"

[dev-dependencies]
tempfile = "3"
```

**主要コード**:

```rust
// src/types.rs
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub grpc: Option<GrpcConfig>,
    pub database: Option<DatabaseConfig>,
    pub kafka: Option<KafkaConfig>,
    pub redis: Option<RedisConfig>,
    pub redis_session: Option<RedisConfig>,
    pub observability: ObservabilityConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub tier: String,
    pub environment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub read_timeout: Option<String>,
    pub write_timeout: Option<String>,
    pub shutdown_timeout: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrpcConfig {
    pub port: u16,
    pub max_recv_msg_size: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub ssl_mode: Option<String>,
    pub max_open_conns: Option<u32>,
    pub max_idle_conns: Option<u32>,
    pub conn_max_lifetime: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub consumer_group: String,
    pub security_protocol: String,
    pub sasl: Option<KafkaSaslConfig>,
    pub tls: Option<KafkaTlsConfig>,
    pub topics: KafkaTopics,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaSaslConfig {
    pub mechanism: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaTlsConfig {
    pub ca_cert_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaTopics {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: Option<u8>,
    pub pool_size: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ObservabilityConfig {
    pub log: LogConfig,
    pub trace: TraceConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TraceConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub oidc: Option<OidcConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub public_key_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OidcConfig {
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub jwks_uri: String,
    pub jwks_cache_ttl: Option<String>,
}
```

```rust
// src/lib.rs
pub mod types;
mod loader;
mod validate;
mod vault;

pub use types::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read file: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("failed to parse YAML: {0}")]
    ParseYaml(#[from] serde_yaml::Error),
    #[error("validation error: {0}")]
    Validation(String),
}

/// YAML を読み込み Config を返す。env_path があればマージする。
pub fn load(base_path: &str, env_path: Option<&str>) -> Result<Config, ConfigError> {
    let base = std::fs::read_to_string(base_path)?;
    let mut config: Config = serde_yaml::from_str(&base)?;

    if let Some(env) = env_path {
        let env_data = std::fs::read_to_string(env)?;
        let env_config: serde_yaml::Value = serde_yaml::from_str(&env_data)?;
        let mut base_value: serde_yaml::Value = serde_yaml::from_str(&base)?;
        merge_yaml(&mut base_value, &env_config);
        config = serde_yaml::from_value(base_value)?;
    }

    Ok(config)
}

/// 設定値のバリデーション。
pub fn validate(config: &Config) -> Result<(), ConfigError> {
    if config.app.name.is_empty() {
        return Err(ConfigError::Validation("app.name is required".into()));
    }
    if !["system", "business", "service"].contains(&config.app.tier.as_str()) {
        return Err(ConfigError::Validation("app.tier must be system, business, or service".into()));
    }
    if !["dev", "staging", "prod"].contains(&config.app.environment.as_str()) {
        return Err(ConfigError::Validation("app.environment must be dev, staging, or prod".into()));
    }
    if config.server.port == 0 {
        return Err(ConfigError::Validation("server.port must be > 0".into()));
    }
    if config.auth.jwt.issuer.is_empty() {
        return Err(ConfigError::Validation("auth.jwt.issuer is required".into()));
    }
    if config.auth.jwt.audience.is_empty() {
        return Err(ConfigError::Validation("auth.jwt.audience is required".into()));
    }
    Ok(())
}

/// Vault シークレットで設定値を上書きする。
pub fn merge_vault_secrets(config: &mut Config, secrets: &std::collections::HashMap<String, String>) {
    if let Some(v) = secrets.get("database.password") {
        if let Some(ref mut db) = config.database {
            db.password = v.clone();
        }
    }
    if let Some(v) = secrets.get("redis.password") {
        if let Some(ref mut redis) = config.redis {
            redis.password = Some(v.clone());
        }
    }
    if let Some(v) = secrets.get("redis_session.password") {
        if let Some(ref mut redis) = config.redis_session {
            redis.password = Some(v.clone());
        }
    }
    if let Some(v) = secrets.get("kafka.sasl.username") {
        if let Some(ref mut kafka) = config.kafka {
            if let Some(ref mut sasl) = kafka.sasl {
                sasl.username = v.clone();
            }
        }
    }
    if let Some(v) = secrets.get("kafka.sasl.password") {
        if let Some(ref mut kafka) = config.kafka {
            if let Some(ref mut sasl) = kafka.sasl {
                sasl.password = v.clone();
            }
        }
    }
    if let Some(v) = secrets.get("auth.oidc.client_secret") {
        if let Some(ref mut oidc) = config.auth.oidc {
            oidc.client_secret = Some(v.clone());
        }
    }
}

fn merge_yaml(base: &mut serde_yaml::Value, overlay: &serde_yaml::Value) {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(base_map), serde_yaml::Value::Mapping(overlay_map)) => {
            for (key, value) in overlay_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_yaml(base_value, value);
                } else {
                    base_map.insert(key.clone(), value.clone());
                }
            }
        }
        (base, overlay) => {
            *base = overlay.clone();
        }
    }
}
```

**テスト例**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, r#"
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
"#).unwrap();
        let cfg = load(f.path().to_str().unwrap(), None).unwrap();
        assert_eq!(cfg.app.name, "test-server");
        assert_eq!(cfg.server.port, 8080);
    }

    #[test]
    fn test_validate_missing_name() {
        let cfg = Config {
            app: AppConfig { name: "".into(), version: "1.0".into(), tier: "system".into(), environment: "dev".into() },
            server: ServerConfig { host: "0.0.0.0".into(), port: 8080, read_timeout: None, write_timeout: None, shutdown_timeout: None },
            observability: ObservabilityConfig {
                log: LogConfig { level: "info".into(), format: "json".into() },
                trace: TraceConfig { enabled: false, endpoint: None, sample_rate: None },
                metrics: MetricsConfig { enabled: false, path: None },
            },
            auth: AuthConfig { jwt: JwtConfig { issuer: "x".into(), audience: "x".into(), public_key_path: None }, oidc: None },
            grpc: None, database: None, kafka: None, redis: None,
        };
        assert!(validate(&cfg).is_err());
    }
}
```

### TypeScript 実装

**配置先**: `regions/system/library/typescript/config/`

```
config/
├── src/
│   ├── index.ts       # 公開 API エクスポート
│   ├── types.ts       # Config 型定義（Zod スキーマ）
│   ├── loader.ts      # YAML 読み込み・マージ
│   └── vault.ts       # Vault シークレットオーバーライド
├── tests/
│   ├── unit/
│   │   └── config.test.ts
│   └── testutil/
│       └── fixtures.ts
├── package.json
└── tsconfig.json
```

**package.json**:

```json
{
  "name": "@k1s0/config",
  "version": "0.1.0",
  "type": "module",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "dependencies": {
    "yaml": "^2.4.0",
    "zod": "^3.23.0",
    "deepmerge": "^4.3.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vitest": "^2.0.0",
    "@types/node": "^22.0.0"
  }
}
```

**主要コード**:

```typescript
// src/types.ts
import { z } from 'zod';

export const AppConfigSchema = z.object({
  name: z.string().min(1),
  version: z.string().min(1),
  tier: z.enum(['system', 'business', 'service']),
  environment: z.enum(['dev', 'staging', 'prod']),
});

export const ServerConfigSchema = z.object({
  host: z.string().min(1),
  port: z.number().int().min(1).max(65535),
  read_timeout: z.string().optional(),
  write_timeout: z.string().optional(),
  shutdown_timeout: z.string().optional(),
});

export const DatabaseConfigSchema = z.object({
  host: z.string().min(1),
  port: z.number().int().min(1).max(65535),
  name: z.string().min(1),
  user: z.string().min(1),
  password: z.string(),
  ssl_mode: z.enum(['disable', 'require', 'verify-full']).optional(),
  max_open_conns: z.number().optional(),
  max_idle_conns: z.number().optional(),
  conn_max_lifetime: z.string().optional(),
}).optional();

export const KafkaConfigSchema = z.object({
  brokers: z.array(z.string()).min(1),
  consumer_group: z.string().min(1),
  security_protocol: z.enum(['PLAINTEXT', 'SASL_SSL']),
  sasl: z.object({
    mechanism: z.enum(['SCRAM-SHA-512', 'PLAIN']),
    username: z.string(),
    password: z.string(),
  }).optional(),
  tls: z.object({
    ca_cert_path: z.string().optional(),
  }).optional(),
  topics: z.object({
    publish: z.array(z.string()),
    subscribe: z.array(z.string()),
  }),
}).optional();

export const RedisConfigSchema = z.object({
  host: z.string().min(1),
  port: z.number().int().min(1).max(65535),
  password: z.string().optional(),
  db: z.number().optional(),
  pool_size: z.number().optional(),
}).optional();

export const ObservabilityConfigSchema = z.object({
  log: z.object({
    level: z.enum(['debug', 'info', 'warn', 'error']),
    format: z.enum(['json', 'text']),
  }),
  trace: z.object({
    enabled: z.boolean(),
    endpoint: z.string().optional(),
    sample_rate: z.number().min(0).max(1).optional(),
  }),
  metrics: z.object({
    enabled: z.boolean(),
    path: z.string().optional(),
  }),
});

export const AuthConfigSchema = z.object({
  jwt: z.object({
    issuer: z.string().min(1),
    audience: z.string().min(1),
    public_key_path: z.string().optional(),
  }),
  oidc: z.object({
    discovery_url: z.string().url(),
    client_id: z.string().min(1),
    client_secret: z.string().optional(),
    redirect_uri: z.string().url(),
    scopes: z.array(z.string()),
    jwks_uri: z.string().url(),
    jwks_cache_ttl: z.string().optional(),
  }).optional(),
});

export const ConfigSchema = z.object({
  app: AppConfigSchema,
  server: ServerConfigSchema,
  grpc: z.object({
    port: z.number().int().min(1).max(65535),
    max_recv_msg_size: z.number().optional(),
  }).optional(),
  database: DatabaseConfigSchema,
  kafka: KafkaConfigSchema,
  redis: RedisConfigSchema,
  redis_session: RedisConfigSchema,
  observability: ObservabilityConfigSchema,
  auth: AuthConfigSchema,
});

export type Config = z.infer<typeof ConfigSchema>;
export type AppConfig = z.infer<typeof AppConfigSchema>;
export type ServerConfig = z.infer<typeof ServerConfigSchema>;
export type DatabaseConfig = z.infer<typeof DatabaseConfigSchema>;
export type AuthConfig = z.infer<typeof AuthConfigSchema>;
```

```typescript
// src/loader.ts
import { readFileSync } from 'node:fs';
import { parse } from 'yaml';
import deepmerge from 'deepmerge';
import { ConfigSchema, type Config } from './types';

export function load(basePath: string, envPath?: string): Config {
  const baseContent = readFileSync(basePath, 'utf-8');
  let config = parse(baseContent);

  if (envPath) {
    const envContent = readFileSync(envPath, 'utf-8');
    const envConfig = parse(envContent);
    config = deepmerge(config, envConfig);
  }

  return config as Config;
}

export function validate(config: Config): void {
  ConfigSchema.parse(config);
}

export function mergeVaultSecrets(
  config: Config,
  secrets: Record<string, string>,
): Config {
  const merged = structuredClone(config);
  if (secrets['database.password'] && merged.database) {
    merged.database.password = secrets['database.password'];
  }
  if (secrets['redis.password'] && merged.redis) {
    merged.redis.password = secrets['redis.password'];
  }
  if (secrets['redis_session.password'] && merged.redis_session) {
    merged.redis_session.password = secrets['redis_session.password'];
  }
  if (secrets['kafka.sasl.username'] && merged.kafka?.sasl) {
    merged.kafka.sasl.username = secrets['kafka.sasl.username'];
  }
  if (secrets['kafka.sasl.password'] && merged.kafka?.sasl) {
    merged.kafka.sasl.password = secrets['kafka.sasl.password'];
  }
  if (secrets['auth.oidc.client_secret'] && merged.auth.oidc) {
    merged.auth.oidc.client_secret = secrets['auth.oidc.client_secret'];
  }
  return merged;
}
```

```typescript
// src/index.ts
export { load, validate, mergeVaultSecrets } from './loader';
export { ConfigSchema, type Config, type AppConfig, type ServerConfig, type DatabaseConfig, type AuthConfig } from './types';
```

**テスト例**:

```typescript
// tests/unit/config.test.ts
import { describe, it, expect } from 'vitest';
import { writeFileSync, mkdtempSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { load, validate, mergeVaultSecrets } from '../../src';

describe('config', () => {
  it('should load a valid config', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = join(dir, 'config.yaml');
    writeFileSync(path, `
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
`);
    const cfg = load(path);
    expect(cfg.app.name).toBe('test-server');
    expect(cfg.server.port).toBe(8080);
  });

  it('should reject invalid config', () => {
    const cfg = { app: { name: '' } } as any;
    expect(() => validate(cfg)).toThrow();
  });

  it('should merge vault secrets', () => {
    const cfg = load(/* ... */);
    const merged = mergeVaultSecrets(cfg, { 'database.password': 'secret' });
    expect(merged.database?.password).toBe('secret');
  });
});
```

### Dart 実装

**配置先**: `regions/system/library/dart/config/`

```
config/
├── lib/
│   ├── config.dart          # エントリーポイント
│   └── src/
│       ├── types.dart       # Config クラス定義
│       ├── loader.dart      # YAML 読み込み・マージ
│       └── vault.dart       # Vault シークレットオーバーライド
├── test/
│   ├── unit/
│   │   └── config_test.dart
│   └── testutil/
│       └── fixtures.dart
├── pubspec.yaml
└── analysis_options.yaml
```

**pubspec.yaml**:

```yaml
name: k1s0_config
version: 0.1.0
environment:
  sdk: ">=3.4.0 <4.0.0"
dependencies:
  yaml: ^3.1.0
  json_annotation: ^4.9.0
dev_dependencies:
  test: ^1.25.0
  json_serializable: ^6.8.0
  build_runner: ^2.4.0
```

**主要コード**:

```dart
// lib/src/types.dart
class Config {
  final AppConfig app;
  final ServerConfig server;
  final GrpcConfig? grpc;
  final DatabaseConfig? database;
  final KafkaConfig? kafka;
  final RedisConfig? redis;
  final RedisConfig? redisSession;
  final ObservabilityConfig observability;
  final AuthConfig auth;

  Config({
    required this.app,
    required this.server,
    this.grpc,
    this.database,
    this.kafka,
    this.redis,
    this.redisSession,
    required this.observability,
    required this.auth,
  });

  factory Config.fromYaml(Map<String, dynamic> yaml) {
    return Config(
      app: AppConfig.fromYaml(yaml['app'] as Map<String, dynamic>),
      server: ServerConfig.fromYaml(yaml['server'] as Map<String, dynamic>),
      grpc: yaml['grpc'] != null ? GrpcConfig.fromYaml(yaml['grpc'] as Map<String, dynamic>) : null,
      database: yaml['database'] != null ? DatabaseConfig.fromYaml(yaml['database'] as Map<String, dynamic>) : null,
      kafka: yaml['kafka'] != null ? KafkaConfig.fromYaml(yaml['kafka'] as Map<String, dynamic>) : null,
      redis: yaml['redis'] != null ? RedisConfig.fromYaml(yaml['redis'] as Map<String, dynamic>) : null,
      redisSession: yaml['redis_session'] != null ? RedisConfig.fromYaml(yaml['redis_session'] as Map<String, dynamic>) : null,
      observability: ObservabilityConfig.fromYaml(yaml['observability'] as Map<String, dynamic>),
      auth: AuthConfig.fromYaml(yaml['auth'] as Map<String, dynamic>),
    );
  }
}

class AppConfig {
  final String name;
  final String version;
  final String tier;
  final String environment;

  AppConfig({required this.name, required this.version, required this.tier, required this.environment});

  factory AppConfig.fromYaml(Map<String, dynamic> yaml) {
    return AppConfig(
      name: yaml['name'] as String,
      version: yaml['version'] as String,
      tier: yaml['tier'] as String,
      environment: yaml['environment'] as String,
    );
  }
}

class ServerConfig {
  final String host;
  final int port;
  final String? readTimeout;
  final String? writeTimeout;
  final String? shutdownTimeout;

  ServerConfig({required this.host, required this.port, this.readTimeout, this.writeTimeout, this.shutdownTimeout});

  factory ServerConfig.fromYaml(Map<String, dynamic> yaml) {
    return ServerConfig(
      host: yaml['host'] as String,
      port: yaml['port'] as int,
      readTimeout: yaml['read_timeout'] as String?,
      writeTimeout: yaml['write_timeout'] as String?,
      shutdownTimeout: yaml['shutdown_timeout'] as String?,
    );
  }
}

class DatabaseConfig {
  final String host;
  final int port;
  final String name;
  final String user;
  String password;
  final String? sslMode;

  DatabaseConfig({required this.host, required this.port, required this.name, required this.user, required this.password, this.sslMode});

  factory DatabaseConfig.fromYaml(Map<String, dynamic> yaml) {
    return DatabaseConfig(
      host: yaml['host'] as String,
      port: yaml['port'] as int,
      name: yaml['name'] as String,
      user: yaml['user'] as String,
      password: yaml['password'] as String? ?? '',
      sslMode: yaml['ssl_mode'] as String?,
    );
  }
}

class GrpcConfig {
  final int port;
  final int? maxRecvMsgSize;

  GrpcConfig({required this.port, this.maxRecvMsgSize});

  factory GrpcConfig.fromYaml(Map<String, dynamic> yaml) {
    return GrpcConfig(port: yaml['port'] as int, maxRecvMsgSize: yaml['max_recv_msg_size'] as int?);
  }
}

class KafkaConfig {
  final List<String> brokers;
  final String consumerGroup;
  final String securityProtocol;

  KafkaConfig({required this.brokers, required this.consumerGroup, required this.securityProtocol});

  factory KafkaConfig.fromYaml(Map<String, dynamic> yaml) {
    return KafkaConfig(
      brokers: (yaml['brokers'] as List).cast<String>(),
      consumerGroup: yaml['consumer_group'] as String,
      securityProtocol: yaml['security_protocol'] as String,
    );
  }
}

class RedisConfig {
  final String host;
  final int port;
  String? password;

  RedisConfig({required this.host, required this.port, this.password});

  factory RedisConfig.fromYaml(Map<String, dynamic> yaml) {
    return RedisConfig(host: yaml['host'] as String, port: yaml['port'] as int, password: yaml['password'] as String?);
  }
}

class ObservabilityConfig {
  final LogConfig log;
  final TraceConfig trace;
  final MetricsConfig metrics;

  ObservabilityConfig({required this.log, required this.trace, required this.metrics});

  factory ObservabilityConfig.fromYaml(Map<String, dynamic> yaml) {
    return ObservabilityConfig(
      log: LogConfig.fromYaml(yaml['log'] as Map<String, dynamic>),
      trace: TraceConfig.fromYaml(yaml['trace'] as Map<String, dynamic>),
      metrics: MetricsConfig.fromYaml(yaml['metrics'] as Map<String, dynamic>),
    );
  }
}

class LogConfig {
  final String level;
  final String format;

  LogConfig({required this.level, required this.format});

  factory LogConfig.fromYaml(Map<String, dynamic> yaml) {
    return LogConfig(level: yaml['level'] as String, format: yaml['format'] as String);
  }
}

class TraceConfig {
  final bool enabled;
  final String? endpoint;
  final double? sampleRate;

  TraceConfig({required this.enabled, this.endpoint, this.sampleRate});

  factory TraceConfig.fromYaml(Map<String, dynamic> yaml) {
    return TraceConfig(
      enabled: yaml['enabled'] as bool,
      endpoint: yaml['endpoint'] as String?,
      sampleRate: (yaml['sample_rate'] as num?)?.toDouble(),
    );
  }
}

class MetricsConfig {
  final bool enabled;
  final String? path;

  MetricsConfig({required this.enabled, this.path});

  factory MetricsConfig.fromYaml(Map<String, dynamic> yaml) {
    return MetricsConfig(enabled: yaml['enabled'] as bool, path: yaml['path'] as String?);
  }
}

class AuthConfig {
  final JwtConfig jwt;
  final OidcConfig? oidc;

  AuthConfig({required this.jwt, this.oidc});

  factory AuthConfig.fromYaml(Map<String, dynamic> yaml) {
    return AuthConfig(
      jwt: JwtConfig.fromYaml(yaml['jwt'] as Map<String, dynamic>),
      oidc: yaml['oidc'] != null ? OidcConfig.fromYaml(yaml['oidc'] as Map<String, dynamic>) : null,
    );
  }
}

class JwtConfig {
  final String issuer;
  final String audience;
  final String? publicKeyPath;

  JwtConfig({required this.issuer, required this.audience, this.publicKeyPath});

  factory JwtConfig.fromYaml(Map<String, dynamic> yaml) {
    return JwtConfig(issuer: yaml['issuer'] as String, audience: yaml['audience'] as String, publicKeyPath: yaml['public_key_path'] as String?);
  }
}

class OidcConfig {
  final String discoveryUrl;
  final String clientId;
  String? clientSecret;
  final String redirectUri;
  final List<String> scopes;
  final String jwksUri;

  OidcConfig({required this.discoveryUrl, required this.clientId, this.clientSecret, required this.redirectUri, required this.scopes, required this.jwksUri});

  factory OidcConfig.fromYaml(Map<String, dynamic> yaml) {
    return OidcConfig(
      discoveryUrl: yaml['discovery_url'] as String,
      clientId: yaml['client_id'] as String,
      clientSecret: yaml['client_secret'] as String?,
      redirectUri: yaml['redirect_uri'] as String,
      scopes: (yaml['scopes'] as List).cast<String>(),
      jwksUri: yaml['jwks_uri'] as String,
    );
  }
}
```

```dart
// lib/src/loader.dart
import 'dart:io';
import 'package:yaml/yaml.dart';
import 'types.dart';

Config loadConfig(String basePath, [String? envPath]) {
  final baseContent = File(basePath).readAsStringSync();
  var yamlMap = _yamlToMap(loadYaml(baseContent));

  if (envPath != null) {
    final envContent = File(envPath).readAsStringSync();
    final envMap = _yamlToMap(loadYaml(envContent));
    yamlMap = _deepMerge(yamlMap, envMap);
  }

  return Config.fromYaml(yamlMap);
}

void validateConfig(Config config) {
  if (config.app.name.isEmpty) throw ConfigValidationError('app.name is required');
  if (!['system', 'business', 'service'].contains(config.app.tier)) {
    throw ConfigValidationError('app.tier must be system, business, or service');
  }
  if (!['dev', 'staging', 'prod'].contains(config.app.environment)) {
    throw ConfigValidationError('app.environment must be dev, staging, or prod');
  }
  if (config.server.port <= 0 || config.server.port > 65535) {
    throw ConfigValidationError('server.port must be between 1 and 65535');
  }
  if (config.auth.jwt.issuer.isEmpty) throw ConfigValidationError('auth.jwt.issuer is required');
  if (config.auth.jwt.audience.isEmpty) throw ConfigValidationError('auth.jwt.audience is required');
}

Config mergeVaultSecrets(Config config, Map<String, String> secrets) {
  if (secrets.containsKey('database.password') && config.database != null) {
    config.database!.password = secrets['database.password']!;
  }
  if (secrets.containsKey('redis.password') && config.redis != null) {
    config.redis!.password = secrets['redis.password'];
  }
  if (secrets.containsKey('redis_session.password') && config.redisSession != null) {
    config.redisSession!.password = secrets['redis_session.password'];
  }
  if (secrets.containsKey('kafka.sasl.username') && config.kafka?.sasl != null) {
    config.kafka!.sasl!.username = secrets['kafka.sasl.username']!;
  }
  if (secrets.containsKey('kafka.sasl.password') && config.kafka?.sasl != null) {
    config.kafka!.sasl!.password = secrets['kafka.sasl.password']!;
  }
  if (secrets.containsKey('auth.oidc.client_secret') && config.auth.oidc != null) {
    config.auth.oidc!.clientSecret = secrets['auth.oidc.client_secret'];
  }
  return config;
}

class ConfigValidationError implements Exception {
  final String message;
  ConfigValidationError(this.message);
  @override
  String toString() => 'ConfigValidationError: $message';
}

Map<String, dynamic> _yamlToMap(dynamic yaml) {
  if (yaml is YamlMap) {
    return yaml.map((k, v) => MapEntry(k.toString(), _yamlToMap(v)));
  }
  if (yaml is YamlList) return yaml.map(_yamlToMap).toList() as dynamic;
  return yaml;
}

Map<String, dynamic> _deepMerge(Map<String, dynamic> base, Map<String, dynamic> overlay) {
  final result = Map<String, dynamic>.from(base);
  for (final key in overlay.keys) {
    if (result.containsKey(key) && result[key] is Map && overlay[key] is Map) {
      result[key] = _deepMerge(result[key] as Map<String, dynamic>, overlay[key] as Map<String, dynamic>);
    } else {
      result[key] = overlay[key];
    }
  }
  return result;
}
```

**テスト例**:

```dart
// test/unit/config_test.dart
import 'dart:io';
import 'package:test/test.dart';
import 'package:k1s0_config/config.dart';

void main() {
  group('Config', () {
    test('should load a valid config', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final file = File('${dir.path}/config.yaml');
      file.writeAsStringSync('''
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
''');
      final cfg = loadConfig(file.path);
      expect(cfg.app.name, 'test-server');
      expect(cfg.server.port, 8080);
    });

    test('should reject empty app name', () {
      final cfg = Config(
        app: AppConfig(name: '', version: '1.0', tier: 'system', environment: 'dev'),
        server: ServerConfig(host: '0.0.0.0', port: 8080),
        observability: ObservabilityConfig(
          log: LogConfig(level: 'info', format: 'json'),
          trace: TraceConfig(enabled: false),
          metrics: MetricsConfig(enabled: false),
        ),
        auth: AuthConfig(jwt: JwtConfig(issuer: 'x', audience: 'x')),
      );
      expect(() => validateConfig(cfg), throwsA(isA<ConfigValidationError>()));
    });
  });
}
```

---

## telemetry ライブラリ

> 詳細な設計方針は [可観測性設計.md](可観測性設計.md) を参照。

### 公開 API（全言語共通契約）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| InitTelemetry | `(config) -> Provider` | OpenTelemetry 初期化（トレース + メトリクス） |
| Shutdown | `() -> void` | プロバイダーのシャットダウン |
| NewLogger | `(config) -> Logger` | 構造化ログのロガー生成 |

### Go 実装

**配置先**: `regions/system/library/go/telemetry/`

```
telemetry/
├── telemetry.go       # InitTelemetry, Shutdown
├── logger.go          # NewLogger, LogWithTrace
├── metrics.go         # Prometheus メトリクス（RED メソッド: request_total, request_duration, request_errors, request_in_flight）
├── middleware.go      # gin HTTP middleware + gRPC interceptor（リクエストログ・duration計測・メトリクス記録）
├── telemetry_test.go
├── go.mod
└── go.sum
```

**依存関係**:

```
go.opentelemetry.io/otel
go.opentelemetry.io/otel/sdk
go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc
go.opentelemetry.io/otel/exporters/prometheus
```

**主要コード**:

```go
package telemetry

import (
    "context"
    "log/slog"
    "os"

    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
    "go.opentelemetry.io/otel/sdk/resource"
    sdktrace "go.opentelemetry.io/otel/sdk/trace"
    semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
    "go.opentelemetry.io/otel/trace"
)

type TelemetryConfig struct {
    ServiceName string
    Version     string
    Tier        string
    Environment string
    TraceEndpoint string
    SampleRate    float64
    LogLevel      string
    LogFormat     string
}

type Provider struct {
    tracerProvider *sdktrace.TracerProvider
    logger         *slog.Logger
}

func InitTelemetry(ctx context.Context, cfg TelemetryConfig) (*Provider, error) {
    var tp *sdktrace.TracerProvider

    if cfg.TraceEndpoint != "" {
        exporter, err := otlptracegrpc.New(ctx,
            otlptracegrpc.WithEndpoint(cfg.TraceEndpoint),
            otlptracegrpc.WithInsecure(),
        )
        if err != nil {
            return nil, err
        }
        tp = sdktrace.NewTracerProvider(
            sdktrace.WithBatcher(exporter),
            sdktrace.WithSampler(sdktrace.TraceIDRatioBased(cfg.SampleRate)),
            sdktrace.WithResource(resource.NewWithAttributes(
                semconv.SchemaURL,
                semconv.ServiceNameKey.String(cfg.ServiceName),
                semconv.ServiceVersionKey.String(cfg.Version),
            )),
        )
        otel.SetTracerProvider(tp)
    }

    logger := NewLogger(cfg)

    return &Provider{tracerProvider: tp, logger: logger}, nil
}

func (p *Provider) Shutdown(ctx context.Context) error {
    if p.tracerProvider != nil {
        return p.tracerProvider.Shutdown(ctx)
    }
    return nil
}

func (p *Provider) Logger() *slog.Logger {
    return p.logger
}

func NewLogger(cfg TelemetryConfig) *slog.Logger {
    level := slog.LevelWarn
    switch cfg.LogLevel {
    case "debug":
        level = slog.LevelDebug
    case "info":
        level = slog.LevelInfo
    case "warn":
        level = slog.LevelWarn
    case "error":
        level = slog.LevelError
    }

    handler := slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: level})
    return slog.New(handler).With(
        slog.String("service", cfg.ServiceName),
        slog.String("version", cfg.Version),
        slog.String("tier", cfg.Tier),
        slog.String("environment", cfg.Environment),
    )
}

func LogWithTrace(ctx context.Context, logger *slog.Logger) *slog.Logger {
    spanCtx := trace.SpanContextFromContext(ctx)
    if spanCtx.HasTraceID() {
        return logger.With(
            slog.String("trace_id", spanCtx.TraceID().String()),
            slog.String("span_id", spanCtx.SpanID().String()),
        )
    }
    return logger
}
```

### Rust 実装

**配置先**: `regions/system/library/rust/telemetry/`

```
telemetry/
├── src/
│   ├── lib.rs           # 公開 API（init_telemetry, shutdown）
│   ├── metrics.rs       # Prometheus メトリクス（prometheus クレート使用、Go の RED メソッドと同等の4メトリクス）
│   └── middleware.rs    # axum HTTP middleware + tonic gRPC interceptor
├── tests/
│   └── integration/
│       └── telemetry_test.rs
└── Cargo.toml
```

**Cargo.toml**:

```toml
[package]
name = "k1s0-telemetry"
version = "0.1.0"
edition = "2021"

[dependencies]
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.28"
```

**主要コード**:

```rust
use opentelemetry::global;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct TelemetryConfig {
    pub service_name: String,
    pub version: String,
    pub tier: String,
    pub environment: String,
    pub trace_endpoint: Option<String>,
    pub sample_rate: f64,
    pub log_level: String,
}

pub fn init_telemetry(cfg: &TelemetryConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref endpoint) = cfg.trace_endpoint {
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()?;
        let provider = sdktrace::TracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_sampler(sdktrace::Sampler::TraceIdRatioBased(cfg.sample_rate))
            .with_resource(Resource::builder()
                .with_service_name(&cfg.service_name)
                .build())
            .build();
        global::set_tracer_provider(provider);
    }

    let filter = EnvFilter::new(&cfg.log_level);
    let fmt_layer = fmt::layer().json().with_target(true);
    let telemetry_layer = cfg.trace_endpoint.as_ref().map(|_| {
        tracing_opentelemetry::layer().with_tracer(global::tracer("k1s0"))
    });

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer);

    if let Some(tl) = telemetry_layer {
        subscriber.with(tl).init();
    } else {
        subscriber.init();
    }

    Ok(())
}

pub fn shutdown() {
    global::shutdown_tracer_provider();
}
```

### TypeScript 実装

**配置先**: `regions/system/library/typescript/telemetry/`

```
telemetry/
├── src/
│   ├── index.ts             # 公開 API エクスポート
│   ├── telemetry.ts         # initTelemetry, shutdown, createLogger
│   ├── metrics.ts           # prom-client ベースのメトリクス収集
│   └── grpcInterceptor.ts   # gRPC interceptor（リクエストログ・duration計測）
├── tests/
│   └── unit/
│       └── telemetry.test.ts
├── package.json
└── tsconfig.json
```

**package.json**:

```json
{
  "name": "@k1s0/telemetry",
  "version": "0.1.0",
  "dependencies": {
    "@opentelemetry/api": "^1.9.0",
    "@opentelemetry/sdk-node": "^0.56.0",
    "@opentelemetry/exporter-trace-otlp-grpc": "^0.56.0",
    "pino": "^9.5.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vitest": "^2.0.0"
  }
}
```

**主要コード**:

```typescript
import { NodeSDK } from '@opentelemetry/sdk-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
import pino from 'pino';

export interface TelemetryConfig {
  serviceName: string;
  version: string;
  tier: string;
  environment: string;
  traceEndpoint?: string;
  sampleRate?: number;
  logLevel: string;
}

let sdk: NodeSDK | undefined;

export function initTelemetry(cfg: TelemetryConfig): void {
  if (cfg.traceEndpoint) {
    const exporter = new OTLPTraceExporter({ url: cfg.traceEndpoint });
    sdk = new NodeSDK({
      traceExporter: exporter,
      serviceName: cfg.serviceName,
    });
    sdk.start();
  }
}

export function shutdown(): Promise<void> {
  return sdk?.shutdown() ?? Promise.resolve();
}

export function createLogger(cfg: TelemetryConfig): pino.Logger {
  return pino({
    level: cfg.logLevel,
    base: {
      service: cfg.serviceName,
      version: cfg.version,
      tier: cfg.tier,
      environment: cfg.environment,
    },
  });
}
```

### Dart 実装

**配置先**: `regions/system/library/dart/telemetry/`

```
telemetry/
├── lib/
│   ├── telemetry.dart       # エントリーポイント
│   └── src/
│       ├── telemetry.dart   # initTelemetry, createLogger
│       ├── metrics.dart     # メトリクス収集
│       └── middleware.dart  # HTTP middleware（リクエストログ・duration計測）
├── test/
│   └── unit/
│       └── telemetry_test.dart
├── pubspec.yaml
└── analysis_options.yaml
```

**pubspec.yaml**:

```yaml
name: k1s0_telemetry
version: 0.1.0
environment:
  sdk: ">=3.4.0 <4.0.0"
dependencies:
  logging: ^1.2.0
  http: ^1.2.0
dev_dependencies:
  test: ^1.25.0
```

**主要コード**:

```dart
import 'dart:convert';
import 'package:logging/logging.dart';

class TelemetryConfig {
  final String serviceName;
  final String version;
  final String tier;
  final String environment;
  final String? traceEndpoint;
  final double sampleRate;
  final String logLevel;

  TelemetryConfig({
    required this.serviceName,
    required this.version,
    required this.tier,
    required this.environment,
    this.traceEndpoint,
    this.sampleRate = 1.0,
    this.logLevel = 'info',
  });
}

void initTelemetry(TelemetryConfig cfg) {
  Logger.root.level = _parseLevel(cfg.logLevel);
  Logger.root.onRecord.listen((record) {
    final entry = {
      'timestamp': record.time.toUtc().toIso8601String(),
      'level': record.level.name.toLowerCase(),
      'message': record.message,
      'service': cfg.serviceName,
      'version': cfg.version,
      'tier': cfg.tier,
      'environment': cfg.environment,
      'logger': record.loggerName,
    };
    if (record.error != null) entry['error'] = record.error.toString();
    print(jsonEncode(entry));
  });
}

Logger createLogger(String name) => Logger(name);

Level _parseLevel(String level) {
  switch (level) {
    case 'debug': return Level.FINE;
    case 'info': return Level.INFO;
    case 'warn': return Level.WARNING;
    case 'error': return Level.SEVERE;
    default: return Level.INFO;
  }
}
```

---

## authlib ライブラリ

> 詳細な認証設計は [認証認可設計.md](認証認可設計.md) を参照。

### サーバー用 API（Go / Rust）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| NewJWKSVerifier | `(jwksURL, cacheTTL) -> Verifier` | JWKS 検証器を生成 |
| VerifyToken | `(tokenString) -> Claims, Error` | JWT トークンを検証 |
| CheckPermission | `(claims, resource, action) -> bool` | RBAC 権限チェック |
| AuthMiddleware | `(verifier) -> Middleware` | HTTP/gRPC 認証ミドルウェア |

### クライアント用 API（TypeScript / Dart）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| createAuthClient | `(config) -> AuthClient` | OAuth2 PKCE クライアント生成 |
| login | `() -> TokenSet` | 認証フロー開始 |
| logout | `() -> void` | ログアウト |
| getAccessToken | `() -> string` | アクセストークン取得（自動リフレッシュ） |
| isAuthenticated | `() -> bool` | 認証状態確認 |

### Go 実装

**配置先**: `regions/system/library/go/auth/`

```
auth/
├── jwks.go            # JWKS 検証器
├── claims.go          # Claims 型定義
├── middleware.go       # gin ミドルウェア
├── grpc_interceptor.go # gRPC インターセプター
├── rbac.go            # RBAC 権限チェック
├── jwks_test.go
├── middleware_test.go
├── rbac_test.go
├── go.mod
└── go.sum
```

**依存関係**:

```
github.com/lestrrat-go/jwx/v2
github.com/gin-gonic/gin
google.golang.org/grpc
```

**主要コード**:

```go
package authlib

import (
    "context"
    "fmt"
    "sync"
    "time"

    "github.com/lestrrat-go/jwx/v2/jwk"
    "github.com/lestrrat-go/jwx/v2/jwt"
)

type Claims struct {
    Sub            string              `json:"sub"`
    Issuer         string              `json:"iss"`
    Audience       []string            `json:"aud"`
    ExpiresAt      time.Time           `json:"exp"`
    IssuedAt       time.Time           `json:"iat"`
    Jti            string              `json:"jti"`
    Typ            string              `json:"typ"`
    Azp            string              `json:"azp"`
    Scope          string              `json:"scope"`
    Username       string              `json:"preferred_username"`
    Email          string              `json:"email"`
    RealmAccess    RealmAccess         `json:"realm_access"`
    ResourceAccess map[string]RoleSet  `json:"resource_access"`
    TierAccess     []string            `json:"tier_access"`
}

type RealmAccess struct {
    Roles []string `json:"roles"`
}

type RoleSet struct {
    Roles []string `json:"roles"`
}

type JWKSVerifier struct {
    jwksURL   string
    cacheTTL  time.Duration
    issuer    string
    audience  string
    mu        sync.RWMutex
    keySet    jwk.Set
    lastFetch time.Time
}

func NewJWKSVerifier(jwksURL, issuer, audience string, cacheTTL time.Duration) *JWKSVerifier {
    return &JWKSVerifier{
        jwksURL:  jwksURL,
        issuer:   issuer,
        audience: audience,
        cacheTTL: cacheTTL,
    }
}

func (v *JWKSVerifier) VerifyToken(ctx context.Context, tokenString string) (*Claims, error) {
    keySet, err := v.getKeySet(ctx)
    if err != nil {
        return nil, fmt.Errorf("failed to get JWKS: %w", err)
    }

    token, err := jwt.Parse([]byte(tokenString),
        jwt.WithKeySet(keySet),
        jwt.WithIssuer(v.issuer),
        jwt.WithAudience(v.audience),
        jwt.WithValidate(true),
    )
    if err != nil {
        return nil, fmt.Errorf("token validation failed: %w", err)
    }

    return extractClaims(token)
}

func (v *JWKSVerifier) getKeySet(ctx context.Context) (jwk.Set, error) {
    v.mu.RLock()
    if v.keySet != nil && time.Since(v.lastFetch) < v.cacheTTL {
        defer v.mu.RUnlock()
        return v.keySet, nil
    }
    v.mu.RUnlock()

    v.mu.Lock()
    defer v.mu.Unlock()

    keySet, err := jwk.Fetch(ctx, v.jwksURL)
    if err != nil {
        return nil, err
    }
    v.keySet = keySet
    v.lastFetch = time.Now()
    return keySet, nil
}

func CheckPermission(claims *Claims, resource, action string) bool {
    for _, access := range claims.ResourceAccess {
        for _, role := range access.Roles {
            if role == action || role == "admin" {
                return true
            }
        }
    }
    for _, role := range claims.RealmAccess.Roles {
        if role == "admin" {
            return true
        }
    }
    return false
}
```

### Rust 実装

**配置先**: `regions/system/library/rust/auth/`

**Cargo.toml**:

```toml
[package]
name = "k1s0-auth"
version = "0.1.0"
edition = "2021"

[dependencies]
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["sync", "time"] }
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
wiremock = "0.6"
```

**主要コード**:

```rust
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub exp: u64,
    pub iat: u64,
    pub jti: Option<String>,
    pub typ: Option<String>,
    pub azp: Option<String>,
    pub scope: Option<String>,
    pub preferred_username: Option<String>,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<HashMap<String, RoleSet>>,
    pub tier_access: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleSet {
    pub roles: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("token expired")]
    TokenExpired,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),
    #[error("permission denied")]
    PermissionDenied,
}

pub struct JwksVerifier {
    jwks_url: String,
    issuer: String,
    audience: String,
    cache_ttl: std::time::Duration,
    keys: Arc<RwLock<Option<(Vec<Jwk>, std::time::Instant)>>>,
}

#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

impl JwksVerifier {
    pub fn new(jwks_url: &str, issuer: &str, audience: &str, cache_ttl: std::time::Duration) -> Self {
        Self {
            jwks_url: jwks_url.to_string(),
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            cache_ttl,
            keys: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let keys = self.get_keys().await?;
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let kid = header.kid.ok_or_else(|| AuthError::InvalidToken("missing kid".into()))?;

        let jwk = keys.iter().find(|k| k.kid == kid)
            .ok_or_else(|| AuthError::InvalidToken("key not found".into()))?;

        let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let data = decode::<Claims>(token, &key, &validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(data.claims)
    }

    async fn get_keys(&self) -> Result<Vec<Jwk>, AuthError> {
        let cache = self.keys.read().await;
        if let Some((ref keys, ref fetched_at)) = *cache {
            if fetched_at.elapsed() < self.cache_ttl {
                return Ok(keys.clone());
            }
        }
        drop(cache);

        let resp: JwksResponse = reqwest::get(&self.jwks_url).await
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?
            .json().await
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?;

        let mut cache = self.keys.write().await;
        *cache = Some((resp.keys.clone(), std::time::Instant::now()));
        Ok(resp.keys)
    }
}

pub fn check_permission(claims: &Claims, _resource: &str, action: &str) -> bool {
    if let Some(ref realm) = claims.realm_access {
        if realm.roles.contains(&"admin".to_string()) {
            return true;
        }
    }
    if let Some(ref resources) = claims.resource_access {
        for roles in resources.values() {
            if roles.roles.contains(&action.to_string()) || roles.roles.contains(&"admin".to_string()) {
                return true;
            }
        }
    }
    false
}
```

### TypeScript 実装

**配置先**: `regions/system/library/typescript/auth/`

**package.json**:

```json
{
  "name": "@k1s0/auth",
  "version": "0.1.0",
  "dependencies": {
    "axios": "^1.7.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vitest": "^2.0.0"
  }
}
```

**主要コード**:

```typescript
// src/auth-client.ts
export interface AuthConfig {
  discoveryUrl: string;
  clientId: string;
  redirectUri: string;
  scopes: string[];
}

export interface TokenSet {
  accessToken: string;
  refreshToken: string;
  idToken: string;
  expiresAt: number;
}

export type AuthStateCallback = (authenticated: boolean) => void;

export class AuthClient {
  private config: AuthConfig;
  private tokenSet: TokenSet | null = null;
  private listeners: AuthStateCallback[] = [];
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(config: AuthConfig) {
    this.config = config;
  }

  async login(): Promise<void> {
    const { codeVerifier, codeChallenge } = await this.generatePKCE();
    const state = crypto.randomUUID();

    sessionStorage.setItem('pkce_verifier', codeVerifier);
    sessionStorage.setItem('oauth_state', state);

    const params = new URLSearchParams({
      response_type: 'code',
      client_id: this.config.clientId,
      redirect_uri: this.config.redirectUri,
      scope: this.config.scopes.join(' '),
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
      state,
    });

    window.location.href = `${await this.getAuthorizationEndpoint()}?${params}`;
  }

  async handleCallback(code: string, state: string): Promise<TokenSet> {
    const savedState = sessionStorage.getItem('oauth_state');
    if (state !== savedState) throw new Error('State mismatch');

    const codeVerifier = sessionStorage.getItem('pkce_verifier');
    if (!codeVerifier) throw new Error('Missing PKCE verifier');

    const tokenEndpoint = await this.getTokenEndpoint();
    const resp = await fetch(tokenEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: this.config.clientId,
        code,
        redirect_uri: this.config.redirectUri,
        code_verifier: codeVerifier,
      }),
    });

    const data = await resp.json();
    this.tokenSet = {
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      idToken: data.id_token,
      expiresAt: Date.now() + data.expires_in * 1000,
    };

    this.scheduleRefresh();
    this.notifyListeners(true);
    return this.tokenSet;
  }

  async getAccessToken(): Promise<string> {
    if (!this.tokenSet) throw new Error('Not authenticated');
    if (Date.now() >= this.tokenSet.expiresAt - 60000) {
      await this.refreshToken();
    }
    return this.tokenSet.accessToken;
  }

  isAuthenticated(): boolean {
    return this.tokenSet !== null && Date.now() < this.tokenSet.expiresAt;
  }

  async logout(): Promise<void> {
    this.tokenSet = null;
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    this.notifyListeners(false);
  }

  onAuthStateChange(callback: AuthStateCallback): () => void {
    this.listeners.push(callback);
    return () => { this.listeners = this.listeners.filter(l => l !== callback); };
  }

  private async refreshToken(): Promise<void> {
    if (!this.tokenSet?.refreshToken) throw new Error('No refresh token');
    const tokenEndpoint = await this.getTokenEndpoint();
    const resp = await fetch(tokenEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        client_id: this.config.clientId,
        refresh_token: this.tokenSet.refreshToken,
      }),
    });
    const data = await resp.json();
    this.tokenSet = {
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      idToken: data.id_token,
      expiresAt: Date.now() + data.expires_in * 1000,
    };
    this.scheduleRefresh();
  }

  private scheduleRefresh(): void {
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    if (!this.tokenSet) return;
    const delay = this.tokenSet.expiresAt - Date.now() - 60000;
    if (delay > 0) {
      this.refreshTimer = setTimeout(() => this.refreshToken(), delay);
    }
  }

  private notifyListeners(authenticated: boolean): void {
    this.listeners.forEach(cb => cb(authenticated));
  }

  private async getAuthorizationEndpoint(): Promise<string> {
    const discovery = await this.fetchDiscovery();
    return discovery.authorization_endpoint;
  }

  private async getTokenEndpoint(): Promise<string> {
    const discovery = await this.fetchDiscovery();
    return discovery.token_endpoint;
  }

  private discoveryCache: any = null;
  private async fetchDiscovery(): Promise<any> {
    if (!this.discoveryCache) {
      const resp = await fetch(this.config.discoveryUrl);
      this.discoveryCache = await resp.json();
    }
    return this.discoveryCache;
  }

  private async generatePKCE(): Promise<{ codeVerifier: string; codeChallenge: string }> {
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    const codeVerifier = btoa(String.fromCharCode(...array))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(codeVerifier));
    const codeChallenge = btoa(String.fromCharCode(...new Uint8Array(digest)))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    return { codeVerifier, codeChallenge };
  }
}
```

### Dart 実装

**配置先**: `regions/system/library/dart/auth/`

**pubspec.yaml**:

```yaml
name: k1s0_auth
version: 0.1.0
environment:
  sdk: ">=3.4.0 <4.0.0"
dependencies:
  dio: ^5.7.0
  flutter_secure_storage: ^9.2.0
  crypto: ^3.0.0
dev_dependencies:
  test: ^1.25.0
  mocktail: ^1.0.0
```

**主要コード**:

```dart
import 'dart:convert';
import 'dart:math';
import 'package:crypto/crypto.dart';
import 'package:dio/dio.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class AuthConfig {
  final String discoveryUrl;
  final String clientId;
  final String redirectUri;
  final List<String> scopes;

  AuthConfig({required this.discoveryUrl, required this.clientId, required this.redirectUri, required this.scopes});
}

class TokenSet {
  final String accessToken;
  final String refreshToken;
  final String idToken;
  final DateTime expiresAt;

  TokenSet({required this.accessToken, required this.refreshToken, required this.idToken, required this.expiresAt});
}

typedef AuthStateCallback = void Function(bool authenticated);

class AuthClient {
  final AuthConfig config;
  final FlutterSecureStorage _storage;
  final Dio _dio;
  TokenSet? _tokenSet;
  final List<AuthStateCallback> _listeners = [];
  Map<String, dynamic>? _discoveryCache;

  AuthClient(this.config)
      : _storage = const FlutterSecureStorage(),
        _dio = Dio();

  Future<String> getAuthorizationUrl() async {
    final discovery = await _fetchDiscovery();
    final codeVerifier = _generateCodeVerifier();
    final codeChallenge = _generateCodeChallenge(codeVerifier);
    final state = _generateRandomString(32);

    await _storage.write(key: 'pkce_verifier', value: codeVerifier);
    await _storage.write(key: 'oauth_state', value: state);

    final params = {
      'response_type': 'code',
      'client_id': config.clientId,
      'redirect_uri': config.redirectUri,
      'scope': config.scopes.join(' '),
      'code_challenge': codeChallenge,
      'code_challenge_method': 'S256',
      'state': state,
    };

    return '${discovery['authorization_endpoint']}?${Uri(queryParameters: params).query}';
  }

  Future<TokenSet> handleCallback(String code, String state) async {
    final savedState = await _storage.read(key: 'oauth_state');
    if (state != savedState) throw AuthError('State mismatch');

    final codeVerifier = await _storage.read(key: 'pkce_verifier');
    if (codeVerifier == null) throw AuthError('Missing PKCE verifier');

    final discovery = await _fetchDiscovery();
    final resp = await _dio.post(
      discovery['token_endpoint'] as String,
      data: {
        'grant_type': 'authorization_code',
        'client_id': config.clientId,
        'code': code,
        'redirect_uri': config.redirectUri,
        'code_verifier': codeVerifier,
      },
      options: Options(contentType: Headers.formUrlEncodedContentType),
    );

    _tokenSet = TokenSet(
      accessToken: resp.data['access_token'],
      refreshToken: resp.data['refresh_token'],
      idToken: resp.data['id_token'],
      expiresAt: DateTime.now().add(Duration(seconds: resp.data['expires_in'])),
    );

    await _persistTokens();
    _notifyListeners(true);
    return _tokenSet!;
  }

  Future<String> getAccessToken() async {
    if (_tokenSet == null) throw AuthError('Not authenticated');
    if (DateTime.now().isAfter(_tokenSet!.expiresAt.subtract(const Duration(minutes: 1)))) {
      await refreshToken();
    }
    return _tokenSet!.accessToken;
  }

  bool get isAuthenticated => _tokenSet != null && DateTime.now().isBefore(_tokenSet!.expiresAt);

  Future<void> refreshToken() async {
    if (_tokenSet?.refreshToken == null) throw AuthError('No refresh token');
    final discovery = await _fetchDiscovery();
    final resp = await _dio.post(
      discovery['token_endpoint'] as String,
      data: {
        'grant_type': 'refresh_token',
        'client_id': config.clientId,
        'refresh_token': _tokenSet!.refreshToken,
      },
      options: Options(contentType: Headers.formUrlEncodedContentType),
    );
    _tokenSet = TokenSet(
      accessToken: resp.data['access_token'],
      refreshToken: resp.data['refresh_token'],
      idToken: resp.data['id_token'],
      expiresAt: DateTime.now().add(Duration(seconds: resp.data['expires_in'])),
    );
    await _persistTokens();
  }

  Future<void> logout() async {
    _tokenSet = null;
    await _storage.deleteAll();
    _notifyListeners(false);
  }

  void Function() onAuthStateChange(AuthStateCallback callback) {
    _listeners.add(callback);
    return () => _listeners.remove(callback);
  }

  Future<void> _persistTokens() async {
    if (_tokenSet == null) return;
    await _storage.write(key: 'access_token', value: _tokenSet!.accessToken);
    await _storage.write(key: 'refresh_token', value: _tokenSet!.refreshToken);
  }

  void _notifyListeners(bool authenticated) {
    for (final cb in _listeners) { cb(authenticated); }
  }

  Future<Map<String, dynamic>> _fetchDiscovery() async {
    if (_discoveryCache != null) return _discoveryCache!;
    final resp = await _dio.get(config.discoveryUrl);
    _discoveryCache = resp.data as Map<String, dynamic>;
    return _discoveryCache!;
  }

  String _generateCodeVerifier() => _generateRandomString(43);

  String _generateCodeChallenge(String verifier) {
    final bytes = utf8.encode(verifier);
    final digest = sha256.convert(bytes);
    return base64Url.encode(digest.bytes).replaceAll('=', '');
  }

  String _generateRandomString(int length) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
    final random = Random.secure();
    return List.generate(length, (_) => chars[random.nextInt(chars.length)]).join();
  }
}

class AuthError implements Exception {
  final String message;
  AuthError(this.message);
  @override
  String toString() => 'AuthError: $message';
}
```

---

## k1s0-messaging ライブラリ

### 概要

Kafka イベント発行・購読の抽象化ライブラリ。`EventProducer` トレイトと `NoOpEventProducer`（テスト用）実装、`EventMetadata`、`EventEnvelope` を提供する。具体的な Kafka クライアント実装は依存せず、トレイト境界でモック差し替えが可能。

Rust 実装のみ（他言語は別途対応予定）

**配置先**: `regions/system/library/rust/messaging/`

### 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `EventProducer` | トレイト | イベント発行の抽象インターフェース（`async fn publish`） |
| `MockEventProducer` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `EventEnvelope` | 構造体 | 発行イベントのラッパー（ペイロード + メタデータ） |
| `EventMetadata` | 構造体 | イベントID・相関ID・タイムスタンプ・ソースサービス名 |
| `MessagingConfig` | 構造体 | ブローカー・トピック・コンシューマーグループ設定 |
| `ConsumerConfig` | 構造体 | コンシューマー固有設定 |
| `EventConsumer` | トレイト | イベント購読の抽象インターフェース（`async fn subscribe`） |
| `MessagingError` | enum | 発行・購読エラー型 |

### Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-messaging"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**Cargo.toml への追加行**:

```toml
k1s0-messaging = { path = "../../system/library/rust/messaging" }
# テスト時にモックを有効化する場合:
k1s0-messaging = { path = "../../system/library/rust/messaging", features = ["mock"] }
```

**モジュール構成**:

```
messaging/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── config.rs       # MessagingConfig・ConsumerConfig
│   ├── consumer.rs     # EventConsumer トレイト
│   ├── error.rs        # MessagingError
│   ├── event.rs        # EventEnvelope・EventMetadata
│   └── producer.rs     # EventProducer トレイト・MockEventProducer
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_messaging::{EventEnvelope, EventMetadata, EventProducer};

// プロデューサーへのイベント発行
async fn publish_user_created<P: EventProducer>(
    producer: &P,
    user_id: &str,
) -> Result<(), k1s0_messaging::MessagingError> {
    let metadata = EventMetadata::new("auth-service");
    let payload = serde_json::json!({ "user_id": user_id });
    let envelope = EventEnvelope::new("k1s0.system.auth.user-created.v1", payload, metadata);
    producer.publish(envelope).await
}
```

---

## k1s0-kafka ライブラリ

### 概要

Kafka 接続設定・管理・ヘルスチェックライブラリ。`KafkaConfig`（TLS・SASL 対応）、`KafkaHealthChecker`、`TopicConfig`（命名規則検証）を提供する。k1s0-messaging の具体的な Kafka 実装の基盤となる。

Rust 実装のみ（他言語は別途対応予定）

**配置先**: `regions/system/library/rust/kafka/`

### 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `KafkaConfig` | 構造体 | ブローカーアドレス・TLS・SASL・コンシューマーグループ設定 |
| `KafkaHealthChecker` | 構造体 | Kafka クラスター疎通確認・ヘルスチェック |
| `TopicConfig` | 構造体 | トピック名・パーティション数・レプリカ数の設定 |
| `TopicPartitionInfo` | 構造体 | トピックのパーティション情報（オフセット等） |
| `KafkaError` | enum | 接続・設定・ヘルスチェックエラー型 |

### Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-kafka"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"

[dev-dependencies]
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-kafka = { path = "../../system/library/rust/kafka" }
```

**モジュール構成**:

```
kafka/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── config.rs       # KafkaConfig（TLS・SASL 設定を含む）
│   ├── error.rs        # KafkaError
│   ├── health.rs       # KafkaHealthChecker
│   └── topic.rs        # TopicConfig・TopicPartitionInfo・命名規則検証
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_kafka::{KafkaConfig, KafkaHealthChecker, TopicConfig};

// 設定例（SASL_SSL）
let config = KafkaConfig::builder()
    .brokers(vec!["kafka:9092".to_string()])
    .consumer_group("auth-service-group")
    .security_protocol("SASL_SSL")
    .build()?;

// ヘルスチェック
let checker = KafkaHealthChecker::new(config);
checker.check().await?;

// トピック命名規則検証（k1s0.<tier>.<service>.<event>.<version>）
let topic = TopicConfig::new("k1s0.system.auth.user-created.v1")?;
```

---

## k1s0-correlation ライブラリ

### 概要

分散トレーシング用相関 ID・トレース ID 管理ライブラリ。`CorrelationId`（UUID v4）、`TraceId`（32 文字 hex）、`CorrelationContext`、HTTP ヘッダー定数を提供する。サービス間リクエストの追跡に使用し、全サーバー・クライアントで統一的に利用する。

Rust 実装のみ（他言語は別途対応予定）

**配置先**: `regions/system/library/rust/correlation/`

### 公開 API

| 型・定数 | 種別 | 説明 |
|---------|------|------|
| `CorrelationId` | 構造体 | UUID v4 ベースの相関 ID（新規生成・文字列変換対応） |
| `TraceId` | 構造体 | 32 文字 hex のトレース ID（OpenTelemetry 互換） |
| `CorrelationContext` | 構造体 | 相関 ID + トレース ID をまとめたコンテキスト |
| `CorrelationHeaders` | 構造体 | HTTP ヘッダー定数（`X-Correlation-Id`・`X-Trace-Id` 等） |

### Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-correlation"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

**Cargo.toml への追加行**:

```toml
k1s0-correlation = { path = "../../system/library/rust/correlation" }
```

**モジュール構成**:

```
correlation/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── context.rs      # CorrelationContext・CorrelationHeaders（HTTP ヘッダー定数）
│   └── id.rs           # CorrelationId（UUID v4）・TraceId（32 文字 hex）
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_correlation::{CorrelationContext, CorrelationHeaders, CorrelationId, TraceId};

// 新規コンテキスト生成（リクエスト受信時）
let ctx = CorrelationContext::new(CorrelationId::new(), TraceId::new());

// HTTP ヘッダーへの設定
let headers = [
    (CorrelationHeaders::CORRELATION_ID, ctx.correlation_id().to_string()),
    (CorrelationHeaders::TRACE_ID, ctx.trace_id().to_string()),
];

// 下流リクエストへの伝播
let child_ctx = ctx.propagate(); // 相関 ID 継承・新規スパン ID 生成
```

---

## k1s0-outbox ライブラリ

### 概要

トランザクショナルアウトボックスパターンライブラリ。データベーストランザクションと Kafka メッセージ発行の原子性を保証する。`OutboxMessage`（指数バックオフリトライ）、`OutboxStore` トレイト、`OutboxPublisher` トレイト、`OutboxProcessor` を提供する。

Rust 実装のみ（他言語は別途対応予定）

**配置先**: `regions/system/library/rust/outbox/`

### 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `OutboxMessage` | 構造体 | アウトボックスに保存するメッセージ（トピック・ペイロード・ステータス・リトライ回数） |
| `OutboxStatus` | enum | メッセージのステータス（`Pending`・`Published`・`Failed`） |
| `OutboxStore` | トレイト | アウトボックスメッセージの永続化抽象（`save`・`fetch_pending`・`mark_published`） |
| `OutboxProcessor` | 構造体 | `OutboxStore` から未発行メッセージを取得し発行するポーリングプロセッサ |
| `OutboxError` | enum | 保存・取得・発行エラー型 |

### Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-outbox"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**Cargo.toml への追加行**:

```toml
k1s0-outbox = { path = "../../system/library/rust/outbox" }
```

**モジュール構成**:

```
outbox/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── error.rs        # OutboxError
│   ├── message.rs      # OutboxMessage・OutboxStatus（指数バックオフ計算含む）
│   ├── processor.rs    # OutboxProcessor（ポーリングループ・リトライ制御）
│   └── store.rs        # OutboxStore トレイト・OutboxPublisher トレイト
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_outbox::{OutboxMessage, OutboxProcessor, OutboxStore};

// ドメインイベント保存とメッセージ発行を同一トランザクションで実行
async fn create_user_with_event<S: OutboxStore>(
    store: &S,
    user_id: &str,
) -> Result<(), k1s0_outbox::OutboxError> {
    let payload = serde_json::json!({ "user_id": user_id });
    let msg = OutboxMessage::new("k1s0.system.auth.user-created.v1", payload);
    // DB トランザクション内で保存（Saga の一部として）
    store.save(&msg).await
}

// バックグラウンドで未発行メッセージをポーリング発行
let processor = OutboxProcessor::new(store, publisher, /* poll_interval */ Duration::from_secs(5));
processor.run().await;
```

---

## k1s0-schemaregistry ライブラリ

### 概要

Confluent Schema Registry クライアントライブラリ。`SchemaRegistryClient` トレイト（HTTP 実装: `HttpSchemaRegistryClient`）、`SchemaRegistryConfig`、`RegisteredSchema`、`SchemaType`（Avro/Json/Protobuf）を提供する。Kafka トピックのスキーマ登録・取得・互換性検証に使用する。

Rust 実装のみ（他言語は別途対応予定）

**配置先**: `regions/system/library/rust/schemaregistry/`

### 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SchemaRegistryClient` | トレイト | スキーマ登録・取得・互換性確認の抽象インターフェース |
| `HttpSchemaRegistryClient` | 構造体 | HTTP ベースの Schema Registry クライアント実装 |
| `MockSchemaRegistryClient` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `SchemaRegistryConfig` | 構造体 | Registry URL・認証情報・互換性モード設定 |
| `CompatibilityMode` | enum | スキーマ互換性モード（`Backward`・`Forward`・`Full`・`None`） |
| `RegisteredSchema` | 構造体 | 登録済みスキーマ（ID・バージョン・スキーマ文字列） |
| `SchemaType` | enum | スキーマ形式（`Avro`・`Json`・`Protobuf`） |
| `SchemaRegistryError` | enum | 登録・取得・互換性エラー型 |

### Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-schemaregistry"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
reqwest = { version = "0.12", features = ["json"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-schemaregistry = { path = "../../system/library/rust/schemaregistry" }
# テスト時にモックを有効化する場合:
k1s0-schemaregistry = { path = "../../system/library/rust/schemaregistry", features = ["mock"] }
```

**モジュール構成**:

```
schemaregistry/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # SchemaRegistryClient トレイト・HttpSchemaRegistryClient・MockSchemaRegistryClient
│   ├── config.rs       # SchemaRegistryConfig・CompatibilityMode・subject_name ヘルパー
│   ├── error.rs        # SchemaRegistryError
│   └── schema.rs       # RegisteredSchema・SchemaType
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_schemaregistry::{
    HttpSchemaRegistryClient, SchemaRegistryClient, SchemaRegistryConfig, SchemaType,
};

let config = SchemaRegistryConfig::new("http://schema-registry:8081");
let client = HttpSchemaRegistryClient::new(config)?;

let topic = "k1s0.system.auth.user-created.v1";
let subject = SchemaRegistryConfig::subject_name(topic); // "<topic>-value"

// Protobuf スキーマを登録
let schema_id = client
    .register_schema(
        &subject,
        r#"syntax = "proto3"; message UserCreated { string user_id = 1; }"#,
        SchemaType::Protobuf,
    )
    .await?;

// 既存スキーマを ID で取得
let registered = client.get_schema_by_id(schema_id).await?;
```

---

## k1s0-serviceauth ライブラリ

### 概要

サービス間 OAuth2 Client Credentials 認証ライブラリ。`ServiceAuthClient` トレイト（HTTP 実装: `HttpServiceAuthClient`）、`ServiceToken`（キャッシュ・自動更新）、`SpiffeId`（SPIFFE URI 検証）を提供する。Istio mTLS 環境でのワークロードアイデンティティ検証もサポートする。

Rust 実装のみ（他言語は別途対応予定）

**配置先**: `regions/system/library/rust/serviceauth/`

### 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `ServiceAuthClient` | トレイト | トークン取得・検証の抽象インターフェース |
| `HttpServiceAuthClient` | 構造体 | OAuth2 Client Credentials フローの HTTP 実装 |
| `MockServiceAuthClient` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `ServiceClaims` | 構造体 | サービストークンのクレーム（`sub`・`iss`・`scope` 等） |
| `ServiceAuthConfig` | 構造体 | トークンエンドポイント・クライアント ID/シークレット・JWKS URI |
| `ServiceToken` | 構造体 | アクセストークン + 有効期限（キャッシュ・自動更新対応） |
| `SpiffeId` | 構造体 | SPIFFE URI のパース・検証（`spiffe://<trust-domain>/ns/<ns>/sa/<sa>`） |
| `ServiceAuthError` | enum | トークン取得・検証・SPIFFE エラー型 |

### Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-serviceauth"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
reqwest = { version = "0.12", features = ["json"] }
jsonwebtoken = "9"
chrono = { version = "0.4", features = ["serde"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-serviceauth = { path = "../../system/library/rust/serviceauth" }
# テスト時にモックを有効化する場合:
k1s0-serviceauth = { path = "../../system/library/rust/serviceauth", features = ["mock"] }
```

**モジュール構成**:

```
serviceauth/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # ServiceAuthClient トレイト・HttpServiceAuthClient・ServiceClaims・MockServiceAuthClient
│   ├── config.rs       # ServiceAuthConfig（トークンエンドポイント・JWKS URI 等）
│   ├── error.rs        # ServiceAuthError
│   └── token.rs        # ServiceToken（有効期限管理）・SpiffeId（URI 検証）
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_serviceauth::{HttpServiceAuthClient, ServiceAuthClient, ServiceAuthConfig};

let config = ServiceAuthConfig::new(
    "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/token",
    "my-service",
    "my-secret",
)
.with_jwks_uri("https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs");

let client = HttpServiceAuthClient::new(config).unwrap();

// キャッシュ付きトークン取得（有効期限前に自動リフレッシュ）
let bearer = client.get_cached_token().await.unwrap();

// gRPC 発信時のヘッダー設定
let mut request = tonic::Request::new(payload);
request.metadata_mut().insert(
    "authorization",
    format!("Bearer {}", bearer.access_token).parse().unwrap(),
);

// SPIFFE ID 検証（Istio mTLS 環境）
let spiffe = client
    .validate_spiffe_id("spiffe://k1s0.internal/ns/system/sa/auth-service", "system")
    .unwrap();
```

---

## テスト方針

全ライブラリで TDD を適用する。

| 言語 | ユニットテスト | モック | 統合テスト |
|------|---------------|--------|-----------|
| Go | testify + assert/require | gomock | testcontainers-go |
| Rust | #[cfg(test)] + assert | mockall | wiremock |
| TypeScript | vitest + expect | MSW | vitest |
| Dart | test + expect | mocktail | test |

### テストカバレッジ目標

| 対象 | カバレッジ |
|------|-----------|
| config ライブラリ | 90% 以上 |
| telemetry ライブラリ | 80% 以上 |
| authlib ライブラリ | 90% 以上 |
| k1s0-messaging | 85% 以上 |
| k1s0-kafka | 80% 以上 |
| k1s0-correlation | 90% 以上 |
| k1s0-outbox | 85% 以上 |
| k1s0-schemaregistry | 85% 以上 |
| k1s0-serviceauth | 90% 以上 |

---

## 関連ドキュメント

- [config設計](config設計.md) — config.yaml スキーマ・環境別管理
- [可観測性設計](可観測性設計.md) — OpenTelemetry・構造化ログ・メトリクス
- [認証認可設計](認証認可設計.md) — OAuth2.0・JWT・RBAC
- [tier-architecture](tier-architecture.md) — Tier 間の依存関係
- [コーディング規約](コーディング規約.md) — 命名規則・Linter 設定
- [ディレクトリ構成図](ディレクトリ構成図.md) — ライブラリのディレクトリ構成
- [system-server設計](system-server設計.md) — auth サーバーの詳細設計
- [system-database設計](system-database設計.md) — auth-db スキーマ設計
