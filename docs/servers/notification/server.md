# system-notification-server 設計

system tier の通知管理サーバー設計を定義する。メール・Slack・Webhook への通知配信を一元管理する。Kafka トピック `k1s0.system.notification.requested.v1` をトリガーに非同期配信を行い、配信結果を PostgreSQL に記録する。
Rust での実装を定義する。

## 概要

system tier の通知管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| 通知チャネル管理 | Email / Slack / Webhook の接続設定 CRUD |
| 通知テンプレート管理 | テンプレートの作成・更新・削除・一覧取得 |
| 通知送信 | REST による即時送信 / Kafka `k1s0.system.notification.requested.v1` 経由の非同期送信 |
| 配信履歴管理 | 配信状態・エラー内容を PostgreSQL に記録し一覧・詳細取得を提供 |
| リトライ管理 | 配信失敗時の自動リトライ（指数バックオフ）と手動再送 API |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

### 配置パス

配置: `regions/system/server/rust/notification/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| 通知配信方式 | Kafka コンシューマーで `k1s0.system.notification.requested.v1` を受信し非同期配信。REST `/send` エンドポイントでの即時配信も提供 |
| リトライ | 失敗時に指数バックオフ（初回 1 秒、最大 5 回、上限 60 秒）で自動リトライ |
| DB | PostgreSQL の `notification` スキーマ（notification_channels, notification_templates, notification_logs テーブル） |
| Kafka | コンシューマー（`k1s0.system.notification.requested.v1`）+ プロデューサー（`k1s0.system.notification.delivered.v1`） |
| テンプレートエンジン | Handlebars 形式のテンプレートを DB 管理し、送信時にプレースホルダーを置換 |
| 認証 | JWTによる認可。管理系エンドポイントは `sys_operator` / `sys_admin` ロールが必要 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_NOTIFY_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/channels` | チャネル一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/channels` | チャネル作成 | `sys_operator` 以上 |
| GET | `/api/v1/channels/:id` | チャネル詳細取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/channels/:id` | チャネル更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/channels/:id` | チャネル削除 | `sys_admin` のみ |
| GET | `/api/v1/templates` | テンプレート一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/templates` | テンプレート作成 | `sys_operator` 以上 |
| GET | `/api/v1/templates/:id` | テンプレート詳細取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/templates/:id` | テンプレート更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/templates/:id` | テンプレート削除 | `sys_admin` のみ |
| POST | `/api/v1/notifications` | 即時通知送信 | `sys_operator` 以上 |
| GET | `/api/v1/notifications` | 配信履歴一覧 | `sys_auditor` 以上 |
| GET | `/api/v1/notifications/:id` | 配信履歴詳細 | `sys_auditor` 以上 |
| POST | `/api/v1/notifications/:id/retry` | 通知再送 | `sys_operator` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/channels

通知チャネル一覧をページネーション付きで取得する。`channel_type` クエリパラメータでフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `channel_type` | string | No | - | チャネル種別でフィルタ（email/slack/webhook） |
| `enabled_only` | bool | No | false | 有効なチャネルのみ取得 |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "channels": [
    {
      "id": "ch_01JABCDEF1234567890",
      "name": "本番メール通知",
      "channel_type": "email",
      "enabled": true,
      "config": {
        "smtp_host": "smtp.example.com",
        "smtp_port": 587,
        "from_address": "noreply@example.com"
      },
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 5,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

#### POST /api/v1/channels

新しい通知チャネルを作成する。`channel_type` は `email` / `slack` / `webhook` のいずれかを指定する。

**リクエスト**

```json
{
  "name": "本番メール通知",
  "channel_type": "email",
  "enabled": true,
  "config": {
    "smtp_host": "smtp.example.com",
    "smtp_port": 587,
    "from_address": "noreply@example.com",
    "username": "smtp-user",
    "password": "smtp-password"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "id": "ch_01JABCDEF1234567890",
  "name": "本番メール通知",
  "channel_type": "email",
  "enabled": true,
  "config": {
    "smtp_host": "smtp.example.com",
    "smtp_port": 587,
    "from_address": "noreply@example.com"
  },
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_NOTIFY_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "channel_type", "message": "must be one of: email, slack, webhook"},
      {"field": "config.smtp_host", "message": "smtp_host is required for email channel"}
    ]
  }
}
```

#### POST /api/v1/notifications/send

指定チャネルへ即時通知を送信する。`template_id` を指定した場合はテンプレートを使用し、`body` を直接指定した場合はそのまま送信する。

**リクエスト**

```json
{
  "channel_id": "ch_01JABCDEF1234567890",
  "template_id": "tpl_01JABCDEF1234567890",
  "variables": {
    "user_name": "田中 太郎",
    "event_type": "ログイン"
  },
  "recipient": "tanaka@example.com"
}
```

**レスポンス（202 Accepted）**

```json
{
  "notification_id": "notif_01JABCDEF1234567890",
  "status": "queued",
  "channel_id": "ch_01JABCDEF1234567890",
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_NOTIFY_CHANNEL_NOT_FOUND",
    "message": "notification channel not found: ch_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/notifications

配信履歴一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `status` | string | No | - | 配信状態でフィルタ（queued/sent/failed） |
| `channel_id` | string | No | - | チャネル ID でフィルタ |
| `from` | string | No | - | 開始日時（RFC3339） |
| `to` | string | No | - | 終了日時（RFC3339） |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "notifications": [
    {
      "id": "notif_01JABCDEF1234567890",
      "channel_id": "ch_01JABCDEF1234567890",
      "channel_type": "email",
      "recipient": "tanaka@example.com",
      "status": "sent",
      "retry_count": 0,
      "sent_at": "2026-02-20T12:30:05.000+00:00",
      "created_at": "2026-02-20T12:30:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 100,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

#### GET /api/v1/notifications/:id

配信履歴の詳細を取得する。

**レスポンス（200 OK）**

```json
{
  "id": "notif_01JABCDEF1234567890",
  "channel_id": "ch_01JABCDEF1234567890",
  "channel_type": "email",
  "template_id": "tpl_01JABCDEF1234567890",
  "recipient": "tanaka@example.com",
  "subject": "ログイン通知",
  "body": "田中 太郎 様、ログインを検知しました。",
  "status": "sent",
  "retry_count": 0,
  "error_message": null,
  "sent_at": "2026-02-20T12:30:05.000+00:00",
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_NOTIFY_NOT_FOUND",
    "message": "notification not found: notif_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/notifications/:id/retry

失敗した通知を手動で再送する。`status` が `failed` の通知にのみ適用できる。

**レスポンス（202 Accepted）**

```json
{
  "notification_id": "notif_01JABCDEF1234567890",
  "status": "queued",
  "retry_count": 1
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_NOTIFY_INVALID_STATUS",
    "message": "notification is not in failed status: notif_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_NOTIFY_NOT_FOUND` | 404 | 指定された通知履歴が見つからない |
| `SYS_NOTIFY_CHANNEL_NOT_FOUND` | 404 | 指定されたチャネルが見つからない |
| `SYS_NOTIFY_TEMPLATE_NOT_FOUND` | 404 | 指定されたテンプレートが見つからない |
| `SYS_NOTIFY_ALREADY_EXISTS` | 409 | 同一名のチャネルが既に存在する |
| `SYS_NOTIFY_INVALID_STATUS` | 409 | 操作に対して通知のステータスが不正 |
| `SYS_NOTIFY_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_NOTIFY_DELIVERY_ERROR` | 502 | 外部サービスへの配信エラー |
| `SYS_NOTIFY_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.notification.v1;

service NotificationService {
  rpc SendNotification(SendNotificationRequest) returns (SendNotificationResponse);
  rpc GetNotification(GetNotificationRequest) returns (GetNotificationResponse);
}

message SendNotificationRequest {
  string channel_id = 1;
  optional string template_id = 2;
  map<string, string> variables = 3;
  string recipient = 4;
  optional string subject = 5;
  optional string body = 6;
}

message SendNotificationResponse {
  string notification_id = 1;
  string status = 2;
  string created_at = 3;
}

message GetNotificationRequest {
  string notification_id = 1;
}

message GetNotificationResponse {
  NotificationLog notification = 1;
}

message NotificationLog {
  string id = 1;
  string channel_id = 2;
  string channel_type = 3;
  optional string template_id = 4;
  string recipient = 5;
  optional string subject = 6;
  string body = 7;
  string status = 8;
  uint32 retry_count = 9;
  optional string error_message = 10;
  optional string sent_at = 11;
  string created_at = 12;
}
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `NotificationChannel`, `NotificationTemplate`, `NotificationLog` | エンティティ定義 |
| domain/repository | `NotificationChannelRepository`, `NotificationTemplateRepository`, `NotificationLogRepository` | リポジトリトレイト |
| domain/service | `NotificationDomainService` | テンプレート適用・リトライ判定ロジック |
| usecase | `SendNotificationUsecase`, `RetryNotificationUsecase`, `GetNotificationUsecase`, `ListNotificationsUsecase`, `CreateChannelUsecase`, `UpdateChannelUsecase`, `DeleteChannelUsecase`, `CreateTemplateUsecase`, `UpdateTemplateUsecase`, `DeleteTemplateUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic）, Kafka コンシューマー | プロトコル変換・メッセージ受信 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `NotificationChannelPostgresRepository`, `NotificationTemplatePostgresRepository`, `NotificationLogPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/messaging | `NotificationKafkaConsumer`, `NotificationDeliveredKafkaProducer` | Kafka コンシューマー・プロデューサー |
| infrastructure/delivery | `EmailDeliveryClient`, `SlackDeliveryClient`, `WebhookDeliveryClient` | 外部通知配信クライアント |

### ドメインモデル

#### NotificationChannel

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | チャネルの一意識別子 |
| `name` | String | チャネルの表示名 |
| `channel_type` | String | チャネル種別（email / slack / webhook） |
| `enabled` | bool | チャネルの有効/無効 |
| `config` | JSON | チャネル固有の接続設定（SMTP設定等） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### NotificationTemplate

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | テンプレートの一意識別子 |
| `name` | String | テンプレートの表示名 |
| `channel_type` | String | 対象チャネル種別 |
| `subject_template` | Option\<String\> | 件名テンプレート（メール用） |
| `body_template` | String | 本文テンプレート（Handlebars 形式） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### NotificationLog

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | 配信ログの一意識別子 |
| `channel_id` | String | 使用チャネル ID |
| `template_id` | Option\<String\> | 使用テンプレート ID |
| `recipient` | String | 宛先（メールアドレス / Slack チャンネル等） |
| `subject` | Option\<String\> | 件名（メール用） |
| `body` | String | 送信本文（テンプレート適用後） |
| `status` | String | 配信状態（queued / sent / failed） |
| `retry_count` | u32 | リトライ回数 |
| `error_message` | Option\<String\> | エラーメッセージ（失敗時） |
| `sent_at` | Option\<DateTime\<Utc\>\> | 配信完了日時 |
| `created_at` | DateTime\<Utc\> | 記録日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (notification_handler.rs)   │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_channels / create_channel /        │   │
                    │  │  update_channel / delete_channel         │   │
                    │  │  list_templates / create_template /      │   │
                    │  │  update_template / delete_template       │   │
                    │  │  send_notification / list_notifications  │   │
                    │  │  get_notification / retry_notification   │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (notification_grpc.rs)      │   │
                    │  │  SendNotification / GetNotification      │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ Kafka Consumer (notification_consumer.rs)│   │
                    │  │  k1s0.system.notification.requested.v1   │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  SendNotificationUsecase /                      │
                    │  RetryNotificationUsecase /                     │
                    │  GetNotificationUsecase /                       │
                    │  ListNotificationsUsecase /                     │
                    │  CreateChannelUsecase / UpdateChannelUsecase /  │
                    │  DeleteChannelUsecase /                         │
                    │  CreateTemplateUsecase / UpdateTemplateUsecase /│
                    │  DeleteTemplateUsecase                          │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  Notification   │              │ NotificationChannelRepo    │   │
    │  Channel,       │              │ NotificationTemplateRepo   │   │
    │  Template, Log  │              │ NotificationLogRepo        │   │
    └────────────────┘              │ (trait)                    │   │
              │                     └──────────┬─────────────────┘   │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ Notification   │            │                     │
                 │ DomainService  │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ Notification*Postgres   │  │
                    │  │ Consumer /   │  │ Repository (x3)         │  │
                    │  │ Producer     │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ Email / Slack /        │  │
                    │  │ Config       │  │ Webhook Delivery       │  │
                    │  │ Loader       │  │ Client                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## 詳細設計ドキュメント

- [system-notification-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-notification-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。
