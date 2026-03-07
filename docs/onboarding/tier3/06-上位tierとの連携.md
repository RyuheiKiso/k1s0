# service tier 上位 tier との連携

## 概要

service tier は system tier と business tier の両方に依存する。本章では、上位 tier が提供する機能をどのように利用するかを具体的に説明する。

```
service tier
  ├── system tier の利用（認証SDK、共通ライブラリ50+、gRPC API）
  └── business tier の利用（領域共通サーバー、共通UI、領域ライブラリ）
```

---

## system tier の利用

### 認証 SDK（JWT 検証、ログイン/ログアウト）

system tier の auth ライブラリがサーバー・クライアント両方の認証機能を提供する。

#### サーバー側: JWT 検証

```rust
use k1s0_auth::{JwtValidator, Claims};

// axum ミドルウェアとして組み込み
let app = Router::new()
    .route("/api/v1/orders", get(list_orders))
    .layer(k1s0_auth::jwt_layer(jwks_client));

// ハンドラーで認証情報を取得
async fn list_orders(
    claims: Claims,  // ミドルウェアが自動抽出
) -> Result<Json<Vec<Order>>, AppError> {
    let user_id = claims.sub;  // 認証済みユーザーID
    let tenant_id = claims.tenant_id;  // テナントID
    // ...
}
```

#### Go BFF 側: JWT 検証

```go
import "github.com/k1s0/system/library/go/auth"

func main() {
    authMiddleware := auth.NewJWTMiddleware(auth.Config{
        JwksURL: cfg.Auth.JwksURL,
    })

    mux := http.NewServeMux()
    handler := authMiddleware.Wrap(mux)
    http.ListenAndServe(":8080", handler)
}
```

#### クライアント側: ログイン/ログアウト

```typescript
// React: system-client SDK
import { useAuth, AuthProvider } from "system-client/auth";

function LoginPage() {
  const { login, logout, isAuthenticated, user } = useAuth();

  if (isAuthenticated) {
    return <div>ようこそ {user.name}さん <button onClick={logout}>ログアウト</button></div>;
  }
  return <button onClick={() => login()}>ログイン</button>;
}
```

```dart
// Flutter: system_client SDK
import 'package:system_client/auth.dart';

class LoginScreen extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final auth = ref.watch(authProvider);
    return auth.when(
      authenticated: (user) => Text('ようこそ ${user.name}さん'),
      unauthenticated: () => ElevatedButton(
        onPressed: () => ref.read(authProvider.notifier).login(),
        child: const Text('ログイン'),
      ),
    );
  }
}
```

### 共通ライブラリ

system tier は 50 以上の共通ライブラリを提供する。service tier でよく利用するものを以下にまとめる。

#### config（設定管理）

```rust
use k1s0_config::Config;

#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
    kafka: KafkaConfig,
}

let config: AppConfig = Config::load("config.yaml")?;
```

#### telemetry（ログ・トレース・メトリクス）

```rust
use k1s0_telemetry::{init_telemetry, TelemetryConfig};

// サーバー起動時に初期化
init_telemetry(TelemetryConfig {
    service_name: "order-service".into(),
    otlp_endpoint: config.telemetry.otlp_endpoint.clone(),
    log_level: config.telemetry.log_level.clone(),
})?;

// 構造化ログ
tracing::info!(order_id = %id, "注文を作成しました");
```

#### messaging（Kafka イベント発行）

```rust
use k1s0_messaging::{EventProducer, EventEnvelope};

let producer = EventProducer::new(&config.kafka).await?;

let event = EventEnvelope::new(
    "order.created",
    OrderCreatedEvent {
        order_id: order.id,
        customer_id: order.customer_id,
        total: order.total,
    },
);
producer.publish("service.order.events", event).await?;
```

#### cache（Redis 分散キャッシュ）

```rust
use k1s0_cache::CacheClient;

let cache = CacheClient::new(&config.redis).await?;

// キャッシュの読み書き
cache.set("order:order-1", &order, Duration::from_secs(300)).await?;
let cached: Option<Order> = cache.get("order:order-1").await?;
```

#### health（ヘルスチェック）

```rust
use k1s0_health::{HealthChecker, Check};

let health = HealthChecker::new()
    .add_check("database", Check::pg_pool(pool.clone()))
    .add_check("redis", Check::redis(redis.clone()))
    .add_check("kafka", Check::kafka(producer.clone()));

// /health/live, /health/ready エンドポイントを自動登録
app = app.merge(health.routes());
```

#### その他よく利用するライブラリ

| ライブラリ | 用途 | 利用シーン |
| --- | --- | --- |
| k1s0-correlation | 相関ID管理 | リクエスト追跡 |
| k1s0-pagination | ページネーション | 一覧 API |
| k1s0-idempotency | 冪等性保証 | 更新系 API |
| k1s0-retry | リトライ | 外部 API 呼び出し |
| k1s0-circuit-breaker | サーキットブレーカー | 外部サービス障害時 |
| k1s0-validation | バリデーション | 入力値検証 |
| k1s0-migration | DB マイグレーション | スキーマ管理 |
| k1s0-outbox | トランザクショナルアウトボックス | Kafka 発行の信頼性保証 |

### gRPC API 呼び出し

system tier のサーバーが提供する gRPC API を呼び出す。

```rust
use k1s0_serviceauth::ServiceAuthClient;

// サービス間認証付き gRPC クライアント
let auth_client = ServiceAuthClient::new(&config.upstream.auth_grpc).await?;

// ユーザー情報取得
let user = auth_client
    .get_user(GetUserRequest { user_id: claims.sub.clone() })
    .await?;

// 設定取得
let config_client = ConfigServiceClient::new(&config.upstream.config_grpc).await?;
let feature_flags = config_client
    .get_features(GetFeaturesRequest { service: "order-service".into() })
    .await?;
```

---

## business tier の利用

### 領域共通サーバー API

所属する業務領域（例: accounting）の共通サーバーが提供する API を利用する。

```rust
// business tier の領域共通サーバーへの gRPC 呼び出し
let accounting_client = AccountingServiceClient::new(
    &config.upstream.business_api
).await?;

// 領域共通のマスタデータ取得
let tax_rates = accounting_client
    .get_tax_rates(GetTaxRatesRequest { country: "JP".into() })
    .await?;
```

### 共通 UI コンポーネント（React / Flutter）

business tier のクライアントパッケージが提供する領域共通の UI コンポーネントを利用する。

#### React

```typescript
import {
  AccountingLayout,       // 領域共通レイアウト
  CurrencyInput,          // 通貨入力フィールド
  TaxCalculator,          // 税計算コンポーネント
  InvoiceTable,           // 請求書テーブル
} from "business-accounting-client";

function OrderSummary({ order }: { order: Order }) {
  return (
    <AccountingLayout>
      <InvoiceTable items={order.items} />
      <CurrencyInput value={order.total} currency="JPY" readOnly />
    </AccountingLayout>
  );
}
```

#### Flutter

```dart
import 'package:business_accounting_client/widgets.dart';

class OrderSummaryScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return AccountingScaffold(
      child: Column(
        children: [
          InvoiceTable(items: order.items),
          CurrencyDisplay(amount: order.total, currency: 'JPY'),
        ],
      ),
    );
  }
}
```

### 領域共通ライブラリ

business tier のライブラリが提供する領域共通のドメインロジックを利用する。

```rust
// Rust サーバーでの利用
use accounting_common::{TaxCalculator, Currency, RoundingMode};

let tax = TaxCalculator::new(Currency::JPY, RoundingMode::HalfUp);
let total_with_tax = tax.calculate(subtotal, tax_rate);
```

```go
// Go BFF での利用
import accounting "github.com/k1s0/business/accounting/library/go"

total := accounting.CalculateTax(subtotal, taxRate, accounting.CurrencyJPY)
```

---

## 可観測性の統合

### トレース伝播

分散トレーシングにより、クライアント → BFF → 業務サーバー → system/business サーバー の全リクエストチェインを追跡できる。

```
[React/Flutter] → [BFF] → [業務サーバー] → [system server]
     ↓               ↓           ↓                ↓
   traceparent     traceparent  traceparent    traceparent
     ↓               ↓           ↓                ↓
   ──────────── Jaeger / Grafana Tempo ────────────
```

#### サーバー側の設定

```rust
use k1s0_telemetry::init_telemetry;
use k1s0_tracing::trace_layer;
use k1s0_correlation::CorrelationLayer;

// telemetry 初期化（OTLP エクスポーター設定）
init_telemetry(config.telemetry.clone())?;

let app = Router::new()
    .route("/api/v1/orders", get(list_orders))
    // トレースコンテキストの自動抽出・注入
    .layer(trace_layer())
    // 相関 ID の自動伝播
    .layer(CorrelationLayer::new());
```

#### gRPC 呼び出し時のトレース伝播

```rust
// tonic の gRPC 呼び出しにトレースコンテキストを自動注入
let channel = Channel::from_static(&config.upstream.auth_grpc)
    .connect()
    .await?;
let client = AuthServiceClient::with_interceptor(
    channel,
    k1s0_tracing::grpc_interceptor(),  // W3C TraceContext を自動注入
);
```

### メトリクス

```rust
use k1s0_telemetry::metrics;

// カスタムメトリクスの記録
metrics::counter!("orders_created_total", "status" => "success").increment(1);
metrics::histogram!("order_processing_duration_seconds").record(duration.as_secs_f64());
```

### ログ

構造化ログを使用し、トレース ID・相関 ID を自動的に付与する。

```rust
// k1s0_telemetry が自動的にトレース情報を付与
tracing::info!(
    order_id = %order.id,
    customer_id = %order.customer_id,
    total = %order.total,
    "注文を作成しました"
);

// 出力例:
// {
//   "timestamp": "2025-01-15T10:30:00Z",
//   "level": "INFO",
//   "message": "注文を作成しました",
//   "order_id": "order-1",
//   "customer_id": "customer-1",
//   "total": "1000",
//   "trace_id": "abc123...",
//   "span_id": "def456...",
//   "correlation_id": "ghi789...",
//   "service": "order-service"
// }
```

---

## Kafka による非同期メッセージング

サービス間の非同期通信は Kafka を使用する。BFF 間の直接通信は禁止されているため、イベント駆動で連携する。

### イベント発行

```rust
use k1s0_messaging::{EventProducer, EventEnvelope};
use k1s0_outbox::OutboxPublisher;

// 方法1: 直接発行
let producer = EventProducer::new(&config.kafka).await?;
producer.publish("service.order.events", event).await?;

// 方法2: トランザクショナルアウトボックス（推奨）
// DB トランザクションとイベント発行の一貫性を保証
let outbox = OutboxPublisher::new(pool.clone(), producer);

let mut tx = pool.begin().await?;
sqlx::query!("INSERT INTO orders ...").execute(&mut *tx).await?;
outbox.enqueue(&mut tx, "service.order.events", &event).await?;
tx.commit().await?;
// outbox がバックグラウンドで Kafka に発行
```

### イベント購読

```rust
use k1s0_messaging::{EventConsumer, EventHandler};

let consumer = EventConsumer::new(&config.kafka, "order-service").await?;

consumer.subscribe("service.inventory.events", |event: EventEnvelope| async move {
    match event.event_type.as_str() {
        "inventory.reserved" => {
            let payload: InventoryReservedEvent = event.deserialize()?;
            // 在庫確保完了 → 注文ステータス更新
            order_usecase.confirm_order(payload.order_id).await?;
        }
        "inventory.insufficient" => {
            let payload: InventoryInsufficientEvent = event.deserialize()?;
            // 在庫不足 → 注文キャンセル
            order_usecase.cancel_order(payload.order_id, "在庫不足").await?;
        }
        _ => {}
    }
    Ok(())
}).await?;
```

### トピック命名規則

```
{tier}.{サービス名}.{イベントカテゴリ}
```

| 例 | 説明 |
| --- | --- |
| `service.order.events` | order サービスのドメインイベント |
| `service.order.commands` | order サービスへのコマンド |
| `business.accounting.events` | accounting 領域の共通イベント |

### DLQ（Dead Letter Queue）

処理に失敗したメッセージは自動的に DLQ に送られる。system tier の dlq-client ライブラリで再処理・モニタリングが可能。

```rust
use k1s0_dlq_client::DlqClient;

let dlq = DlqClient::new(&config.dlq_server_url);

// DLQ メッセージの確認
let failed_messages = dlq.list("service.order.events").await?;

// 再処理
dlq.retry("service.order.events", message_id).await?;
```

## 関連ドキュメント

- [認証認可設計](../../architecture/auth/認証認可設計.md) — 認証・認可の全体設計
- [JWT 設計](../../architecture/auth/JWT設計.md) — JWT トークン設計
- [サービス間認証設計](../../architecture/auth/サービス間認証設計.md) — mTLS・OAuth2 Client Credentials
- [認証設計](../../architecture/auth/認証設計.md) — 認証フロー設計
- [system-library 概要](../../libraries/_common/概要.md) — 全ライブラリ一覧
- [authlib 設計](../../libraries/auth-security/authlib.md) — 認証ライブラリ詳細
- [config 設計](../../libraries/config/config.md) — 設定管理詳細
- [telemetry 設計](../../libraries/observability/telemetry.md) — テレメトリ詳細
- [tracing 設計](../../libraries/observability/tracing.md) — 分散トレーシング詳細
- [correlation 設計](../../libraries/observability/correlation.md) — 相関 ID 管理
- [messaging 設計](../../libraries/messaging/messaging.md) — メッセージング詳細
- [kafka 設計](../../libraries/messaging/kafka.md) — Kafka 接続管理
- [outbox 設計](../../libraries/messaging/outbox.md) — アウトボックスパターン
- [dlq-client 設計](../../libraries/messaging/dlq-client.md) — DLQ クライアント
- [可観測性設計](../../architecture/observability/可観測性設計.md) — 可観測性の全体設計
- [トレーシング設計](../../architecture/observability/トレーシング設計.md) — トレーシング設計
- [ログ設計](../../architecture/observability/ログ設計.md) — ログ設計方針
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md) — メッセージング全体設計
