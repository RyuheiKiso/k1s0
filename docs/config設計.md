# config.yaml 設計

k1s0 では環境変数の直接参照を禁止し、`config/config.yaml` で設定を一元管理する。
本ドキュメントでは config.yaml のスキーマと運用ルールを定義する。

## 基本方針

- アプリケーションコード内で `os.Getenv` / `std::env::var` 等による環境変数の直接参照を禁止する
- すべての設定値は `config/config.yaml` に定義し、起動時に構造体へバインドする
- 環境別の差分は Kubernetes ConfigMap / Secret から YAML ファイルとしてマウントする
- シークレット（DB パスワード、API キー等）は HashiCorp Vault から注入する

## スキーマ

```yaml
# config/config.yaml

app:
  name: "order-service"          # サービス名
  version: "1.0.0"               # アプリケーションバージョン
  environment: "dev"             # dev | staging | prod

server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"

grpc:                            # gRPC 有効時のみ
  port: 50051
  max_recv_msg_size: 4194304     # 4MB

database:                        # DB 有効時のみ
  host: "localhost"
  port: 5432
  name: "order_db"
  user: "app"
  password: ""                   # Vault から注入
  ssl_mode: "disable"            # disable | require | verify-full
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:                           # Kafka 有効時のみ
  brokers:
    - "kafka-0:9092"
    - "kafka-1:9092"
  consumer_group: "order-service"
  topics:
    publish:
      - "order.created"
      - "order.updated"
    subscribe:
      - "payment.completed"
      - "inventory.reserved"

redis:                           # Redis 有効時のみ
  host: "localhost"
  port: 6379
  password: ""                   # Vault から注入
  db: 0
  pool_size: 10

observability:
  log:
    level: "info"                # debug | info | warn | error
    format: "json"               # json | text
  trace:
    enabled: true
    endpoint: "jaeger:4317"      # OTLP gRPC エンドポイント
    sample_rate: 1.0             # 0.0 〜 1.0
  metrics:
    enabled: true
    endpoint: "prometheus:9090"

auth:
  jwt:
    issuer: "https://auth.example.com"
    audience: "order-service"
    public_key_path: "/etc/secrets/jwt-public.pem"
```

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

### 環境別差分の例（config.prod.yaml）

```yaml
server:
  read_timeout: "10s"
  write_timeout: "10s"

database:
  ssl_mode: "verify-full"
  max_open_conns: 50
  max_idle_conns: 10

observability:
  log:
    level: "warn"
  trace:
    sample_rate: 0.1
```

## シークレット管理

| シークレット       | 注入元          | config.yaml 上の扱い     |
| ------------------ | --------------- | ------------------------ |
| DB パスワード      | Vault           | 空文字で定義             |
| Redis パスワード   | Vault           | 空文字で定義             |
| JWT 公開鍵         | Vault           | ファイルパスで参照       |
| API キー           | Vault           | 空文字で定義             |

Vault Agent Injector が Pod 起動時にシークレットをファイルとして注入し、アプリケーションが起動時に読み込む。

## Go での読み込み実装

```go
// internal/infra/config/config.go
type Config struct {
    App           AppConfig           `yaml:"app"`
    Server        ServerConfig        `yaml:"server"`
    Database      *DatabaseConfig     `yaml:"database,omitempty"`
    Kafka         *KafkaConfig        `yaml:"kafka,omitempty"`
    Redis         *RedisConfig        `yaml:"redis,omitempty"`
    Observability ObservabilityConfig `yaml:"observability"`
    Auth          AuthConfig          `yaml:"auth"`
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
    pub database: Option<DatabaseConfig>,
    pub kafka: Option<KafkaConfig>,
    pub redis: Option<RedisConfig>,
    pub observability: ObservabilityConfig,
    pub auth: AuthConfig,
}
```

## 設計上の制約

- config.yaml にシークレットの実値を記載してはならない（空文字またはファイルパスで定義）
- 環境別 YAML ファイルにもシークレットを含めない
- `config.yaml` の全キーに対してデフォルト値を定義し、環境別ファイルは差分のみ記載する
- 設定値の追加時は Config 構造体とスキーマの両方を更新する
