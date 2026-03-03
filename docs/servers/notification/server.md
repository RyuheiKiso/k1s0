# system-notification-server 設計

system tier の通知管理サーバー設計を定義する。メール・Slack・Webhook・SMS・Push への通知配信を一元管理する。Kafka トピック `k1s0.system.notification.requested.v1` をトリガーに非同期配信を行い、配信結果を PostgreSQL に記録する。
Rust での実装を定義する。

## 概要

system tier の通知管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| 通知チャネル管理 | Email / Slack / Webhook / SMS / Push の接続設定 CRUD |
| 通知テンプレート管理 | テンプレートの作成・更新・削除・一覧取得 |
| 通知送信 | REST `POST /api/v1/notifications` による即時送信 / Kafka `k1s0.system.notification.requested.v1` 経由の非同期送信 |
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
| 通知配信方式 | Kafka コンシューマーで `k1s0.system.notification.requested.v1` を受信し非同期配信。REST `POST /api/v1/notifications` での即時配信も提供 |
| リトライ | 失敗時に指数バックオフ（初回 1 秒、最大 5 回、上限 60 秒）で自動リトライ |
| DB | PostgreSQL の `notification` スキーマ（notification_channels, notification_templates, notification_logs テーブル） |
| Kafka | コンシューマー（`k1s0.system.notification.requested.v1`）+ プロデューサー（`k1s0.system.notification.delivered.v1`） |
| テンプレートエンジン | Handlebars 形式のテンプレートを DB 管理し、送信時にプレースホルダーを置換 |
| 認証 | JWTによる認可。管理系エンドポイントは `sys_operator` / `sys_admin` ロールが必要 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_NOTIF_` とする。

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
| `channel_type` | string | No | - | チャネル種別でフィルタ（email/slack/webhook/sms/push） |
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
      "created_at": "2026-02-20T10:00:00.000+00:00"
    }
  ],
  "total_count": 5,
  "page": 1,
  "page_size": 20,
  "has_next": false
}
```

#### POST /api/v1/channels

新しい通知チャネルを作成する。`channel_type` は `email` / `slack` / `webhook` / `sms` / `push` のいずれかを指定する。

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
    "code": "SYS_NOTIF_VALIDATION_ERROR",
    "message": "validation failed"
  }
}
```

#### GET /api/v1/channels/:id

指定したチャネル ID の詳細を取得する。

**レスポンス（200 OK）**

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
  "updated_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_NOTIF_CHANNEL_NOT_FOUND",
    "message": "notification channel not found: ch_01JABCDEF1234567890"
  }
}
```

#### PUT /api/v1/channels/:id

指定したチャネルを更新する。`name` / `enabled` / `config` の部分更新に対応する。

**リクエスト**

```json
{
  "name": "本番メール通知（Primary）",
  "enabled": true,
  "config": {
    "smtp_host": "smtp.example.com",
    "smtp_port": 587,
    "from_address": "noreply@example.com"
  }
}
```

**レスポンス（200 OK）**

```json
{
  "id": "ch_01JABCDEF1234567890",
  "name": "本番メール通知（Primary）",
  "channel_type": "email",
  "enabled": true,
  "updated_at": "2026-02-20T13:00:00.000+00:00"
}
```

#### DELETE /api/v1/channels/:id

指定したチャネルを削除する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "channel ch_01JABCDEF1234567890 deleted"
}
```

#### GET /api/v1/templates

テンプレート一覧をページネーション付きで取得する。`channel_type` でフィルタ可能。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `channel_type` | string | No | - | チャネル種別でフィルタ（email/slack/webhook/sms/push） |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "templates": [
    {
      "id": "tpl_01JABCDEF1234567890",
      "name": "login-notification",
      "channel_type": "email",
      "created_at": "2026-02-20T10:00:00.000+00:00"
    }
  ],
  "total_count": 8,
  "page": 1,
  "page_size": 20,
  "has_next": false
}
```

#### POST /api/v1/templates

新しい通知テンプレートを作成する。

**リクエスト**

```json
{
  "name": "login-notification",
  "channel_type": "email",
  "subject_template": "ログイン通知",
  "body_template": "{{user_name}} 様、{{event_type}} を検知しました。"
}
```

**レスポンス（201 Created）**

```json
{
  "id": "tpl_01JABCDEF1234567890",
  "name": "login-notification",
  "channel_type": "email",
  "created_at": "2026-02-20T10:00:00.000+00:00"
}
```

#### GET /api/v1/templates/:id

指定したテンプレートの詳細を取得する。

**レスポンス（200 OK）**

```json
{
  "id": "tpl_01JABCDEF1234567890",
  "name": "login-notification",
  "channel_type": "email",
  "subject_template": "ログイン通知",
  "body_template": "{{user_name}} 様、{{event_type}} を検知しました。",
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
}
```

#### PUT /api/v1/templates/:id

指定したテンプレートを更新する。`name` / `subject_template` / `body_template` の部分更新に対応する。

**リクエスト**

```json
{
  "name": "login-notification-v2",
  "subject_template": "ログイン通知（更新）",
  "body_template": "{{user_name}} 様、{{event_type}} が発生しました。"
}
```

**レスポンス（200 OK）**

```json
{
  "id": "tpl_01JABCDEF1234567890",
  "name": "login-notification-v2",
  "channel_type": "email",
  "updated_at": "2026-02-20T13:00:00.000+00:00"
}
```

#### DELETE /api/v1/templates/:id

指定したテンプレートを削除する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "template tpl_01JABCDEF1234567890 deleted"
}
```

#### POST /api/v1/notifications

指定チャネルへ即時通知を送信する。`template_id` を指定した場合はテンプレートを使用し、`body` を直接指定した場合はそのまま送信する。

**リクエスト**

```json
{
  "channel_id": "ch_01JABCDEF1234567890",
  "template_id": "tpl_01JABCDEF1234567890",
  "recipient": "tanaka@example.com",
  "subject": "ログイン通知",
  "body": "田中 太郎 様、ログインを検知しました。",
  "template_variables": {
    "user_name": "田中 太郎",
    "event_type": "ログイン"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "notification_id": "notif_01JABCDEF1234567890",
  "status": "sent",
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_NOTIF_CHANNEL_NOT_FOUND",
    "message": "notification channel not found: ch_01JABCDEF1234567890"
  }
}
```

#### GET /api/v1/notifications

配信履歴一覧を取得する。`channel_id` クエリパラメータでフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `channel_id` | string | No | - | チャネル ID でフィルタ |

**レスポンス（200 OK）**

```json
{
  "notifications": [
    {
      "id": "notif_01JABCDEF1234567890",
      "channel_id": "ch_01JABCDEF1234567890",
      "recipient": "tanaka@example.com",
      "status": "sent",
      "retry_count": 0,
      "sent_at": "2026-02-20T12:30:05.000+00:00",
      "created_at": "2026-02-20T12:30:00.000+00:00"
    }
  ],
  "total_count": 12,
  "page": 1,
  "page_size": 20,
  "has_next": false
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
    "code": "SYS_NOTIF_NOT_FOUND",
    "message": "notification not found: notif_01JABCDEF1234567890"
  }
}
```

#### POST /api/v1/notifications/:id/retry

失敗した通知を手動で再送する。`status` が `failed` の通知にのみ適用できる。

**レスポンス（200 OK）**

```json
{
  "notification_id": "notif_01JABCDEF1234567890",
  "status": "sent",
  "message": "notification retried successfully"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_NOTIF_NOT_FOUND",
    "message": "notification not found: notif_01JABCDEF1234567890"
  }
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_NOTIF_ALREADY_SENT",
    "message": "notification already sent: notif_01JABCDEF1234567890"
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_NOTIF_INVALID_ID` | 400 | 無効な UUID フォーマット |
| `SYS_NOTIF_CHANNEL_DISABLED` | 400 | 対象チャネルが無効化されている |
| `SYS_NOTIF_NOT_FOUND` | 404 | 指定された通知履歴が見つからない |
| `SYS_NOTIF_CHANNEL_NOT_FOUND` | 404 | 指定されたチャネルが見つからない |
| `SYS_NOTIF_TEMPLATE_NOT_FOUND` | 404 | 指定されたテンプレートが見つからない |
| `SYS_NOTIF_ALREADY_SENT` | 409 | 通知はすでに送信済み（再送不可） |
| `SYS_NOTIF_SEND_FAILED` | 500 | 通知送信処理に失敗 |
| `SYS_NOTIF_LIST_FAILED` | 500 | 通知一覧取得処理に失敗 |
| `SYS_NOTIF_GET_FAILED` | 500 | 通知取得処理に失敗 |
| `SYS_NOTIF_RETRY_FAILED` | 500 | 通知再送処理に失敗 |
| `SYS_NOTIF_CHANNEL_CREATE_FAILED` | 500 | チャネル作成処理に失敗 |
| `SYS_NOTIF_CHANNEL_LIST_FAILED` | 500 | チャネル一覧取得処理に失敗 |
| `SYS_NOTIF_CHANNEL_GET_FAILED` | 500 | チャネル取得処理に失敗 |
| `SYS_NOTIF_CHANNEL_UPDATE_FAILED` | 500 | チャネル更新処理に失敗 |
| `SYS_NOTIF_CHANNEL_DELETE_FAILED` | 500 | チャネル削除処理に失敗 |
| `SYS_NOTIF_TEMPLATE_CREATE_FAILED` | 500 | テンプレート作成処理に失敗 |
| `SYS_NOTIF_TEMPLATE_LIST_FAILED` | 500 | テンプレート一覧取得処理に失敗 |
| `SYS_NOTIF_TEMPLATE_GET_FAILED` | 500 | テンプレート取得処理に失敗 |
| `SYS_NOTIF_TEMPLATE_UPDATE_FAILED` | 500 | テンプレート更新処理に失敗 |
| `SYS_NOTIF_TEMPLATE_DELETE_FAILED` | 500 | テンプレート削除処理に失敗 |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.notification.v1;

service NotificationService {
  rpc SendNotification(SendNotificationRequest) returns (SendNotificationResponse);
  rpc GetNotification(GetNotificationRequest) returns (GetNotificationResponse);
  rpc ListChannels(ListChannelsRequest) returns (ListChannelsResponse);
  rpc CreateChannel(CreateChannelRequest) returns (CreateChannelResponse);
  rpc GetChannel(GetChannelRequest) returns (GetChannelResponse);
  rpc UpdateChannel(UpdateChannelRequest) returns (UpdateChannelResponse);
  rpc DeleteChannel(DeleteChannelRequest) returns (DeleteChannelResponse);
  rpc ListTemplates(ListTemplatesRequest) returns (ListTemplatesResponse);
  rpc CreateTemplate(CreateTemplateRequest) returns (CreateTemplateResponse);
  rpc GetTemplate(GetTemplateRequest) returns (GetTemplateResponse);
  rpc UpdateTemplate(UpdateTemplateRequest) returns (UpdateTemplateResponse);
  rpc DeleteTemplate(DeleteTemplateRequest) returns (DeleteTemplateResponse);
}

message SendNotificationRequest {
  string channel_id = 1;
  optional string template_id = 2;
  map<string, string> template_variables = 3;
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

message Channel {
  string id = 1;
  string name = 2;
  string channel_type = 3;
  string config_json = 4;
  bool enabled = 5;
  string created_at = 6;
  string updated_at = 7;
}

message ListChannelsRequest {
  optional string channel_type = 1;
  bool enabled_only = 2;
  uint32 page = 3;
  uint32 page_size = 4;
}

message ListChannelsResponse {
  repeated Channel channels = 1;
  uint64 total = 2;
}

message CreateChannelRequest {
  string name = 1;
  string channel_type = 2;
  string config_json = 3;
  bool enabled = 4;
}

message CreateChannelResponse {
  Channel channel = 1;
}

message GetChannelRequest {
  string id = 1;
}

message GetChannelResponse {
  Channel channel = 1;
}

message UpdateChannelRequest {
  string id = 1;
  optional string name = 2;
  optional bool enabled = 3;
  optional string config_json = 4;
}

message UpdateChannelResponse {
  Channel channel = 1;
}

message DeleteChannelRequest {
  string id = 1;
}

message DeleteChannelResponse {
  bool success = 1;
  string message = 2;
}

message Template {
  string id = 1;
  string name = 2;
  string channel_type = 3;
  optional string subject_template = 4;
  string body_template = 5;
  string created_at = 6;
  string updated_at = 7;
}

message ListTemplatesRequest {
  optional string channel_type = 1;
  uint32 page = 2;
  uint32 page_size = 3;
}

message ListTemplatesResponse {
  repeated Template templates = 1;
  uint64 total = 2;
}

message CreateTemplateRequest {
  string name = 1;
  string channel_type = 2;
  optional string subject_template = 3;
  string body_template = 4;
}

message CreateTemplateResponse {
  Template template = 1;
}

message GetTemplateRequest {
  string id = 1;
}

message GetTemplateResponse {
  Template template = 1;
}

message UpdateTemplateRequest {
  string id = 1;
  optional string name = 2;
  optional string subject_template = 3;
  optional string body_template = 4;
}

message UpdateTemplateResponse {
  Template template = 1;
}

message DeleteTemplateRequest {
  string id = 1;
}

message DeleteTemplateResponse {
  bool success = 1;
  string message = 2;
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
| infrastructure/delivery | `EmailDeliveryClient`, `SlackDeliveryClient`, `WebhookDeliveryClient`, `SmsDeliveryClient`, `PushDeliveryClient` | 外部通知配信クライアント |

### ドメインモデル

#### NotificationChannel

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | チャネルの一意識別子 |
| `name` | String | チャネルの表示名 |
| `channel_type` | String | チャネル種別（email / slack / webhook / sms / push） |
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
                    │  │  send_notification (POST /api/v1/notifications) │   │
                    │  │  get_notification / retry_notification   │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (notification_grpc.rs)      │   │
                    │  │  Send/GetNotification                    │   │
                    │  │  List/Create/Get/Update/DeleteChannel    │   │
                    │  │  List/Create/Get/Update/DeleteTemplate   │   │
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
                    │  │ Config       │  │ Webhook / SMS /        │  │
                    │  │ Loader       │  │ Push Delivery Client   │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## 詳細設計ドキュメント

- [system-notification-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-notification-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

## Doc Sync (2026-03-03)

### gRPC Canonical RPCs (proto)
- `SendNotification`, `GetNotification`, `RetryNotification`, `ListNotifications`
- `ListChannels`, `CreateChannel`, `GetChannel`, `UpdateChannel`, `DeleteChannel`
- `ListTemplates`, `CreateTemplate`, `GetTemplate`, `UpdateTemplate`, `DeleteTemplate`

### Pagination Corrections
- `ListChannelsResponse` uses `k1s0.system.common.v1.PaginationResult`.
- `ListTemplatesResponse` uses `k1s0.system.common.v1.PaginationResult`.

### Optional UseCase Parameters
- Notification create/retry behavior supports optional parameters used by current Rust usecase signatures.
