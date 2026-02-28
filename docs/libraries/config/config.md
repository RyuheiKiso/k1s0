# config ライブラリ設計

> 詳細なスキーマ定義は [config.md](../../cli/config/config設計.md) を参照。

## 公開 API（全言語共通契約）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| Load | `(basePath, envPath?) -> Config` | YAML を読み込み Config を返す |
| Validate | `(config) -> Error?` | 設定値のバリデーション |
| MergeVaultSecrets | `(config, secrets) -> Config` | Vault シークレットで上書き |

## Go 実装

**配置先**: `regions/system/library/go/config/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

// Config は config.md のスキーマに準拠する。
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

## Rust 実装

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

## TypeScript 実装

**配置先**: `regions/system/library/typescript/config/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

## Dart 実装

**配置先**: `regions/system/library/dart/config/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-telemetry設計](../observability/telemetry.md) — telemetry ライブラリ
- [system-library-authlib設計](../auth-security/authlib.md) — authlib ライブラリ
- [system-library-messaging設計](../messaging/messaging.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](../messaging/kafka.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](../messaging/outbox.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](../data/schemaregistry.md) — k1s0-schemaregistry ライブラリ
- [system-library-serviceauth設計](../auth-security/serviceauth.md) — k1s0-serviceauth ライブラリ

---
