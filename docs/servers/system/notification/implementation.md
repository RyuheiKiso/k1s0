# system-notification-server 実装設計

> **注記**: 本ドキュメントは notification-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-notification-server（通知サーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（通知送信・チャネル管理・テンプレート管理・リトライ） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・配信クライアント・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/notification/)

### ディレクトリ構成

```
regions/system/server/rust/notification/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── notification_channel.rs      # NotificationChannel エンティティ（接続設定）
│   │   │   ├── notification_template.rs     # NotificationTemplate エンティティ（Handlebars テンプレート）
│   │   │   └── notification_log.rs          # NotificationLog エンティティ（配信履歴）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── notification_channel_repository.rs
│   │   │   ├── notification_template_repository.rs
│   │   │   └── notification_log_repository.rs
│   │   └── service/
│   │       ├── mod.rs
│   │       ├── delivery_client.rs           # DeliveryClient トレイト（配信抽象化）
│   │       └── notification_domain_service.rs # テンプレート置換・配信ルーティング
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── send_notification.rs             # 通知送信（即時 / Kafka 非同期）
│   │   ├── retry_notification.rs            # 通知再送（指数バックオフ）
│   │   ├── create_channel.rs                # チャネル作成
│   │   ├── list_channels.rs                 # チャネル一覧
│   │   ├── get_channel.rs                   # チャネル取得
│   │   ├── update_channel.rs                # チャネル更新
│   │   ├── delete_channel.rs                # チャネル削除
│   │   ├── create_template.rs               # テンプレート作成
│   │   ├── list_templates.rs                # テンプレート一覧
│   │   ├── get_template.rs                  # テンプレート取得
│   │   ├── update_template.rs               # テンプレート更新
│   │   └── delete_template.rs               # テンプレート削除
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── notification_handler.rs      # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── notification_grpc.rs         # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── channel_postgres.rs          # ChannelRepository PostgreSQL 実装
│   │       ├── template_postgres.rs         # TemplateRepository PostgreSQL 実装
│   │       └── notification_log_postgres.rs # NotificationLogRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── delivery/
│   │   │   ├── mod.rs
│   │   │   ├── email_client.rs              # Email 配信クライアント（SMTP）
│   │   │   ├── slack_client.rs              # Slack 配信クライアント（Webhook）
│   │   │   ├── webhook_client.rs            # Webhook 配信クライアント
│   │   │   ├── sms_client.rs                # SMS 配信クライアント
│   │   │   └── push_client.rs               # Push 配信クライアント
│   │   ├── kafka_producer.rs                # Kafka プロデューサー（通知送信完了イベント）
│   │   ├── kafka_consumer.rs                # Kafka コンシューマー（通知リクエスト受信）
│   │   └── startup.rs                       # 起動シーケンス・DI
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **DeliveryClient** トレイト: 配信チャネルの抽象化インターフェース。各チャネル（Email/Slack/Webhook/SMS/Push）が実装する
- **NotificationDomainService**: Handlebars テンプレートのプレースホルダー置換と配信チャネルへのルーティングを行う

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `SendNotificationUseCase` | 通知送信。REST 即時送信と Kafka 非同期送信の両方をサポートする |
| `RetryNotificationUseCase` | 配信失敗通知の再送（指数バックオフ: 初回 1秒、最大 5回、上限 60秒） |
| `CreateChannelUseCase` 等 | 通知チャネルの CRUD |
| `CreateTemplateUseCase` 等 | 通知テンプレートの CRUD |

#### 配信クライアント

| クライアント | 設定方法 |
|------------|---------|
| `EmailDeliveryClient` | 環境変数 `SMTP_HOST`, `SMTP_USERNAME`, `SMTP_PASSWORD` |
| `SlackDeliveryClient` | 環境変数 `SLACK_WEBHOOK_URL` |
| `WebhookDeliveryClient` | 環境変数 `WEBHOOK_URL` |
| `SmsDeliveryClient` | 環境変数 `SMS_API_ENDPOINT` |
| `PushDeliveryClient` | 環境変数 `PUSH_API_ENDPOINT` |

未設定のクライアントはスキップされる。全クライアント未設定時はテンプレートリポジトリのみで動作する。

#### Kafka 連携

- **Consumer** (`infrastructure/kafka_consumer.rs`): `k1s0.system.notification.requested.v1` を受信し非同期配信を行う。バックグラウンドタスクとして起動する
- **Producer** (`infrastructure/kafka_producer.rs`): `k1s0.system.notification.sent.v1` に配信完了イベントを発行する

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_NOTIFY_`
- 配信失敗時はログに記録し、リトライキューに投入する

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | テンプレート置換・配信ルーティング | mockall によるリポジトリ・DeliveryClient モック |
| InMemory テスト | リポジトリ | startup.rs 内の InMemory 実装による DB 不要テスト |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
