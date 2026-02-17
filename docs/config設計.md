# config.yaml 設計

k1s0 では環境変数の直接参照を禁止し、`config/config.yaml` で設定を一元管理する。
本ドキュメントでは config.yaml のスキーマと運用ルールを定義する。

## 基本方針

- アプリケーションコード内で `os.Getenv` / `std::env::var` 等による環境変数の直接参照を禁止する
  - **例外**: config ローダー（`config/config.yaml` の読み込み処理）内では環境変数参照を許可する
  - **例外**: テスト用のフィクスチャやヘルパーでは環境変数参照を許可する
  - **例外**: Kubernetes の ConfigMap / Secret による環境変数の注入は許可する
  - **例外**: CI/CD パイプラインでの環境変数設定は許可する
- すべての設定値は `config/config.yaml` に定義し、起動時に構造体へバインドする
- 環境別の差分は Kubernetes ConfigMap / Secret から YAML ファイルとしてマウントする
- シークレット（DB パスワード、API キー等）は HashiCorp Vault から注入する

## スキーマ

```yaml
# config/config.yaml

app:
  name: "order-server"           # サービス名
  version: "1.0.0"               # アプリケーションバージョン
  tier: "service"                # system | business | service
  environment: "dev"             # dev | staging | prod

server:
  host: "0.0.0.0"
  port: 8080                     # Kubernetes Service ポート（80）への変換は Kubernetes / Helm で設定
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"

grpc:                            # gRPC 有効時のみ
  port: 50051
  max_recv_msg_size: 4194304     # 4MB

database:                        # DB 有効時のみ
  host: "postgres.k1s0-service.svc.cluster.local"  # Tier に応じて変更: k1s0-system / k1s0-business / k1s0-service
  port: 5432
  name: "order_db"
  user: "app"
  password: ""                   # Vault パス: secret/data/k1s0/{tier}/{service}/database キー: password
  ssl_mode: "disable"            # disable | require | verify-full
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:                           # Kafka 有効時のみ
  brokers:                       # dev: 9092（PLAINTEXT）、prod: 9093（SASL_SSL リスナー）
    - "kafka-0.messaging.svc.cluster.local:9092"
    - "kafka-1.messaging.svc.cluster.local:9092"
  consumer_group: "order-server.default"  # 命名規則: {service-name}.{purpose}（メッセージング設計.md 参照）。サービスごとに変更すること
  security_protocol: "PLAINTEXT"   # PLAINTEXT（dev） | SASL_SSL（staging/prod）
  sasl:                            # security_protocol が SASL_SSL の場合のみ有効
    mechanism: "SCRAM-SHA-512"     # SCRAM-SHA-512 | PLAIN
    username: ""                   # Vault パス: secret/data/k1s0/system/kafka/sasl キー: username
    password: ""                   # Vault パス: secret/data/k1s0/system/kafka/sasl キー: password
  tls:                             # security_protocol が SASL_SSL の場合のみ有効
    ca_cert_path: ""               # Strimzi が発行する CA 証明書のパス
  topics:
    publish:
      - "k1s0.service.order.created.v1"
      - "k1s0.service.order.updated.v1"
    subscribe:
      - "k1s0.service.payment.completed.v1"
      - "k1s0.service.inventory.reserved.v1"

redis:                           # Redis 有効時のみ
  host: "redis.k1s0-system.svc.cluster.local"
  port: 6379
  password: ""                   # Vault パス: secret/data/k1s0/{tier}/{service}/redis キー: password
  db: 0
  pool_size: 10

redis_session:                   # BFF Proxy 用セッションストア（BFF セッション管理で使用）
  host: "redis-session.k1s0-system.svc.cluster.local"  # prod 環境では Redis Sentinel 構成（Master 1 + Replica 2 + Sentinel 3）。詳細は認証認可設計.md の「BFF セッション管理」を参照
  port: 6380
  password: ""                   # Vault パス: secret/data/k1s0/system/bff/redis キー: password（認証認可設計.md 参照）

observability:
  log:
    level: "info"                # debug | info | warn | error — staging 環境向け。dev は debug、prod は warn に環境別 config で上書き
    format: "json"               # json | text
  trace:
    enabled: true
    endpoint: "jaeger.observability.svc.cluster.local:4317"  # OTLP gRPC エンドポイント
    sample_rate: 1.0             # 0.0 〜 1.0
  metrics:
    enabled: true
    path: "/metrics"                                              # Prometheus ServiceMonitor がスクレイプするパス（可観測性設計.md 参照）

auth:
  jwt:
    issuer: "https://auth.k1s0.internal.example.com/realms/k1s0"
    audience: "k1s0-api"
    public_key_path: ""                      # 非推奨: JWKS（oidc.jwks_uri）による動的取得を優先。オフライン検証が必要な場合のみ PEM ファイルパスを指定
  oidc:                                      # BFF または OIDC 連携サービスで使用
    discovery_url: "https://auth.k1s0.internal.example.com/realms/k1s0/.well-known/openid-configuration"
    client_id: "k1s0-bff"
    client_secret: ""                        # Vault パス: secret/data/k1s0/system/bff/oidc キー: client_secret
    redirect_uri: "https://app.k1s0.internal.example.com/callback"
    scopes: ["openid", "profile", "email"]
    jwks_uri: "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs"
    jwks_cache_ttl: "10m"
```

## config.yaml のマウントパス

| 環境               | マウントパス                  | 説明                                                         |
| ------------------ | ----------------------------- | ------------------------------------------------------------ |
| ローカル開発       | `config/config.yaml`          | サービスディレクトリ内の `config/` から読み込み              |
| Kubernetes         | `/etc/app/config.yaml`        | ConfigMap を `/etc/app` にマウント（[helm設計.md](helm設計.md) 参照） |

config ローダーはファイルパスを引数で受け取り、環境に応じて切り替える。ローカル開発時は `config/config.yaml`、Kubernetes 環境では `/etc/app/config.yaml` を読み込む。Dockerfile の `CMD` や Helm values の `args` でパスを指定する。

## 環境別オーバーライド

環境ごとの差分は ConfigMap として Kubernetes にデプロイし、Pod にマウントする。

```
config/
├── config.yaml           # デフォルト値（リポジトリにコミット）
├── config.dev.yaml       # dev 環境の差分（参考用・リポジトリにコミット）
├── config.staging.yaml   # staging 環境の差分（参考用・リポジトリにコミット）
└── config.prod.yaml      # prod 環境の差分（参考用・リポジトリにコミット）
```

### マージ順序と優先順位（D-079）

設定値は以下の順序でマージされ、**後から読み込まれた値が優先** される。

1. `config.yaml`（デフォルト値）をベースとして読み込む — **最低優先**
2. `config.{environment}.yaml` の値で上書きする
3. Vault から注入されたシークレットで上書きする — **最高優先**

#### ConfigMap と Vault で同一キーが存在する場合

**Vault が常に優先** される。これにより以下を保証する。

- シークレットは必ず Vault の値が使用され、ConfigMap に誤って平文が残っていても無視される
- Vault のローテーションが即座に反映される

#### 設計上の制約

- **ConfigMap にシークレットを定義してはならない**（config.yaml には空文字またはダミー値を記載）
- **Vault に非シークレットを格納してはならない**（設定値の出所が曖昧になるため）
- 両者の責務を明確に分離し、キーの重複が発生しないことを原則とする

| 設定値の種別           | 格納先       | 例                                |
| ---------------------- | ------------ | --------------------------------- |
| 非シークレット設定     | ConfigMap    | ポート番号、タイムアウト値、ログレベル |
| シークレット           | Vault        | DB パスワード、API キー、証明書   |
| シークレットのプレースホルダー | config.yaml | `password: ""` （空文字で定義）   |

#### 競合時の動作

万が一 ConfigMap と Vault の両方に同一キーの値が存在した場合:

1. Vault の値を採用する
2. 警告ログを出力する（`WARN: config key "database.password" found in both ConfigMap and Vault, using Vault value`）
3. アプリケーションは正常に起動する（エラーにはしない）

### ネストされた YAML のマージアルゴリズム（D-080）

環境別 YAML ファイルおよび Vault シークレットのマージは、以下のアルゴリズムに従う。

#### マージルール

| データ型 | マージ動作 | 例 |
| --- | --- | --- |
| スカラー値（string, number, bool） | 上位ソースの値で完全置換 | `port: 8080` → `port: 9090` |
| マップ（object） | キー単位で再帰的にディープマージ | `database.host` のみ上書き、他キーは保持 |
| 配列（array） | 上位ソースの配列で完全置換（マージしない） | `brokers: [a]` → `brokers: [b, c]` で `[b, c]` に置換 |
| null 値 | キーの削除として扱う | `redis: null` でベースの redis セクションを削除 |

#### ディープマージの具体例

**config.yaml（ベース）**

```yaml
database:
  host: "localhost"
  port: 5432
  name: "mydb"
  ssl_mode: "disable"
  max_open_conns: 25
```

**config.staging.yaml（オーバーライド）**

```yaml
database:
  host: "postgres.k1s0-system.svc.cluster.local"
  ssl_mode: "require"
  max_open_conns: 30
```

**マージ結果**

```yaml
database:
  host: "postgres.k1s0-system.svc.cluster.local"  # staging で上書き
  port: 5432                                        # ベースを保持
  name: "mydb"                                      # ベースを保持
  ssl_mode: "require"                               # staging で上書き
  max_open_conns: 30                                # staging で上書き
```

#### 配列の完全置換の理由

配列のマージ（追加・差分比較）は要素の同一性判定が曖昧になるため、**完全置換** を採用する。Kafka brokers のように環境ごとに異なるホスト一覧を持つケースでは、部分マージよりも完全置換が安全である。

#### Go 実装

```go
// MergeYAML はベース設定を環境別設定でディープマージする。
// マップはキー単位で再帰マージ、配列とスカラーは完全置換する。
func MergeYAML(base, override map[string]interface{}) map[string]interface{} {
    result := make(map[string]interface{})
    for k, v := range base {
        result[k] = v
    }
    for k, v := range override {
        if v == nil {
            delete(result, k)
            continue
        }
        if baseMap, ok := result[k].(map[string]interface{}); ok {
            if overrideMap, ok := v.(map[string]interface{}); ok {
                result[k] = MergeYAML(baseMap, overrideMap)
                continue
            }
        }
        result[k] = v
    }
    return result
}
```

#### Rust 実装

```rust
/// ベース設定を環境別設定でディープマージする。
/// マップはキー単位で再帰マージ、配列とスカラーは完全置換する。
pub fn merge_yaml(base: &serde_yaml::Value, overlay: &serde_yaml::Value) -> serde_yaml::Value {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(base_map), serde_yaml::Value::Mapping(overlay_map)) => {
            let mut result = base_map.clone();
            for (key, value) in overlay_map {
                if value.is_null() {
                    result.remove(key);
                } else if let Some(base_value) = result.get(key) {
                    result.insert(key.clone(), merge_yaml(base_value, value));
                } else {
                    result.insert(key.clone(), value.clone());
                }
            }
            serde_yaml::Value::Mapping(result)
        }
        (_, overlay) => overlay.clone(),
    }
}
```

### 環境別差分の例（config.staging.yaml）

```yaml
database:
  ssl_mode: "require"
  max_open_conns: 30
  max_idle_conns: 8

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9093"
  security_protocol: "SASL_SSL"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: ""                   # Vault から注入
    password: ""                   # Vault から注入
  tls:
    ca_cert_path: "/etc/kafka/certs/ca.crt"

observability:
  log:
    level: "info"
  trace:
    sample_rate: 0.5
```

### 環境別差分の例（config.prod.yaml）

```yaml
server:
  read_timeout: "10s"
  write_timeout: "10s"

database:
  ssl_mode: "verify-full"
  max_open_conns: 50
  max_idle_conns: 10

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9093"
    - "kafka-1.messaging.svc.cluster.local:9093"
    - "kafka-2.messaging.svc.cluster.local:9093"
  security_protocol: "SASL_SSL"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: ""                   # Vault から注入
    password: ""                   # Vault から注入
  tls:
    ca_cert_path: "/etc/kafka/certs/ca.crt"

observability:
  log:
    level: "warn"
  trace:
    sample_rate: 0.1
```

## シークレット管理

| シークレット           | 注入元          | config.yaml 上の扱い                  |
| ---------------------- | --------------- | ------------------------------------- |
| DB パスワード          | Vault           | 空文字で定義                          |
| Redis パスワード       | Vault           | 空文字で定義                          |
| JWT 公開鍵             | Vault           | ファイルパスで参照                    |
| API キー               | Vault           | 空文字で定義                          |
| OIDC Client Secret     | Vault           | 空文字で定義                          |
| Kafka SASL ユーザー名  | Vault           | 空文字で定義                          |
| Kafka SASL パスワード  | Vault           | 空文字で定義                          |

Vault Agent Injector が Pod 起動時にシークレットをファイルとして注入し、アプリケーションが起動時に読み込む。
各シークレットの具体的な Vault パスとキー名は [認証認可設計.md](認証認可設計.md) の「シークレットパス体系」を参照。

## Vault ブートストラップ

### シークレットパス階層

Vault の KV v2 シークレットエンジンを使用し、以下のパス階層でシークレットを管理する。

```
secret/data/k1s0/
├── system/
│   ├── auth/
│   │   └── database          # password
│   ├── bff/
│   │   ├── oidc              # client_secret
│   │   └── redis             # password
│   ├── kafka/
│   │   └── sasl              # username, password
│   └── {service}/
│       ├── database          # password
│       └── redis             # password
├── business/
│   └── {service}/
│       ├── database          # password
│       └── redis             # password
└── service/
    └── {service}/
        ├── database          # password
        └── redis             # password
```

### Kubernetes Auth Method 設定手順

1. Vault で Kubernetes auth method を有効化する

```bash
vault auth enable kubernetes
```

2. Kubernetes API サーバーの接続情報を設定する

```bash
vault write auth/kubernetes/config \
    kubernetes_host="https://kubernetes.default.svc.cluster.local:443" \
    token_reviewer_jwt=@/var/run/secrets/kubernetes.io/serviceaccount/token \
    kubernetes_ca_cert=@/var/run/secrets/kubernetes.io/serviceaccount/ca.crt
```

3. Tier ごとの Vault ポリシーを作成する

```bash
# system tier 用ポリシー
vault policy write k1s0-system - <<EOF
path "secret/data/k1s0/system/*" {
  capabilities = ["read"]
}
EOF
```

4. Kubernetes ServiceAccount にロールをバインドする

```bash
vault write auth/kubernetes/role/k1s0-system-auth \
    bound_service_account_names=auth-server \
    bound_service_account_namespaces=k1s0-system \
    policies=k1s0-system \
    ttl=1h
```

### merge_vault_secrets() と Vault パスの対応表

| config.yaml キー | Vault パス | Vault キー | merge_vault_secrets() のルックアップキー |
| --- | --- | --- | --- |
| `database.password` | `secret/data/k1s0/{tier}/{service}/database` | `password` | `database.password` |
| `redis.password` | `secret/data/k1s0/{tier}/{service}/redis` | `password` | `redis.password` |
| `redis_session.password` | `secret/data/k1s0/system/bff/redis` | `password` | `redis_session.password` |
| `kafka.sasl.username` | `secret/data/k1s0/system/kafka/sasl` | `username` | `kafka.sasl.username` |
| `kafka.sasl.password` | `secret/data/k1s0/system/kafka/sasl` | `password` | `kafka.sasl.password` |
| `auth.oidc.client_secret` | `secret/data/k1s0/system/bff/oidc` | `client_secret` | `auth.oidc.client_secret` |

## Go での読み込み実装

```go
// internal/infra/config/config.go
type Config struct {
    App           AppConfig           `yaml:"app"`
    Server        ServerConfig        `yaml:"server"`
    GRPC          *GRPCConfig         `yaml:"grpc,omitempty"`
    Database      *DatabaseConfig     `yaml:"database,omitempty"`
    Kafka         *KafkaConfig        `yaml:"kafka,omitempty"`
    Redis         *RedisConfig        `yaml:"redis,omitempty"`
    Observability ObservabilityConfig `yaml:"observability"`
    Auth          AuthConfig          `yaml:"auth"`
}

type GRPCConfig struct {
    Port           int `yaml:"port" validate:"required,min=1,max=65535"`
    MaxRecvMsgSize int `yaml:"max_recv_msg_size"`
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

type AuthConfig struct {
    JWT  JWTConfig   `yaml:"jwt"`
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

func Load(basePath, envPath string) (*Config, error) {
    // 1. basePath を読み込み
    // 2. envPath で上書き
    // 3. Vault シークレットで上書き
}
```

## Rust での読み込み実装

```rust
// src/infra/config/mod.rs
#[derive(Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub grpc: Option<GrpcConfig>,
    pub database: Option<DatabaseConfig>,
    pub kafka: Option<KafkaConfig>,
    pub redis: Option<RedisConfig>,
    pub observability: ObservabilityConfig,
    pub auth: AuthConfig,
}

#[derive(Deserialize)]
pub struct GrpcConfig {
    pub port: u16,
    pub max_recv_msg_size: Option<usize>,
}

#[derive(Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub consumer_group: String,
    pub security_protocol: String,
    pub sasl: Option<KafkaSaslConfig>,
    pub tls: Option<KafkaTlsConfig>,
    pub topics: KafkaTopics,
}

#[derive(Deserialize)]
pub struct KafkaSaslConfig {
    pub mechanism: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct KafkaTlsConfig {
    pub ca_cert_path: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub oidc: Option<OidcConfig>,
}

#[derive(Deserialize)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub public_key_path: Option<String>,
}

#[derive(Deserialize)]
pub struct OidcConfig {
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub jwks_uri: String,
    pub jwks_cache_ttl: Option<String>,
}
```

## バリデーション

config.yaml の値はアプリケーション起動時にバリデーションを実行し、不正な設定値を早期に検出する。

### Go バリデーション

[go-playground/validator](https://github.com/go-playground/validator) を使用し、構造体タグでバリデーションルールを定義する。

```go
import "github.com/go-playground/validator/v10"

func (c *Config) Validate() error {
    validate := validator.New()
    if err := validate.Struct(c); err != nil {
        return fmt.Errorf("config validation failed: %w", err)
    }
    return nil
}
```

- 構造体タグ `validate:"required"` で必須フィールドをチェック
- ポート番号は `validate:"required,min=1,max=65535"` で範囲チェック
- URL 形式は `validate:"required,url"` で形式チェック

### Rust バリデーション

カスタムバリデーション関数を `config::validate()` として実装する。

```rust
impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.app.name.is_empty() {
            return Err(ConfigError::MissingField("app.name".into()));
        }
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue("server.port must be > 0".into()));
        }
        if self.auth.jwt.issuer.is_empty() {
            return Err(ConfigError::MissingField("auth.jwt.issuer".into()));
        }
        // ... 各フィールドの検証
        Ok(())
    }
}
```

### 実行タイミング

| タイミング             | 実行方法                           | 動作                                   |
| ---------------------- | ---------------------------------- | -------------------------------------- |
| アプリケーション起動時 | `config.Validate()` を `main()` 内で呼び出し | 失敗時は即座にエラー終了（`exit(1)`） |
| CI パイプライン        | `config validate` コマンド         | 事前検証でデプロイ前に不正設定を検出   |

- アプリケーション起動時にバリデーションを実行し、失敗時は即座にエラー終了する。不正な設定のまま稼働することを防止する
- CI パイプラインでも `config validate` コマンドによる事前検証を行い、デプロイ前に設定の整合性を保証する

## 設計上の制約

- config.yaml にシークレットの実値を記載してはならない（空文字またはファイルパスで定義）
- 環境別 YAML ファイルにもシークレットを含めない
- `config.yaml` の全キーに対してデフォルト値を定義し、環境別ファイルは差分のみ記載する
- 設定値の追加時は Config 構造体とスキーマの両方を更新する

## 関連ドキュメント

- [helm設計](helm設計.md)
- [認証認可設計](認証認可設計.md)
- [可観測性設計](可観測性設計.md)
- [docker-compose設計](docker-compose設計.md)
- [メッセージング設計](メッセージング設計.md)
- [API設計](API設計.md)
- [CLIフロー](CLIフロー.md)
- [CI-CD設計](CI-CD設計.md)
- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) — config.yaml テンプレートの詳細
- [system-config-server設計](system-config-server設計.md)
