# service-activity-server 設計

service tier のアクティビティ管理サーバー設計を定義する。タスクへのコメント・作業時間・ステータス変更等の操作履歴を REST/gRPC API で記録し、承認フローと冪等性保証を提供する。
Rust で実装する。

## 概要

### RBAC対応表

service tier のロールに基づいてアクセス制御する。

| ロール | read | write | admin |
|--------|------|-------|-------|
| `sys_admin` | ✅ | ✅ | ✅ |
| `svc_admin` | ✅ | ✅ | ✅ |
| `svc_operator` | ✅ | ✅ | ❌ |
| `svc_viewer` | ✅ | ❌ | ❌ |

| アクション | 対象エンドポイント |
|-----------|-----------------|
| `read` | GET（アクティビティ一覧・詳細取得） |
| `write` | POST / PUT（作成・提出） |
| `admin` | PUT（承認・却下：`/approve`・`/reject`） |

実装: `adapter/middleware/rbac.rs` の `require_permission` + `k1s0-server-common` の `check_permission(Tier::Service, ...)` を使用。認証は Bearer JWT 検証（JWKS）。`/healthz`・`/readyz`・`/metrics` は認証除外。

service tier のアクティビティ管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| アクティビティ作成 API | タスクへのコメント・作業時間・ステータス変更等の記録を作成する（冪等性保証） |
| アクティビティ一覧取得 API | タスクID・担当者・タイプによるフィルタリング付きの一覧を取得する |
| アクティビティ詳細取得 API | アクティビティID を指定して取得する |
| 提出 API | アクティビティを `active → submitted` に遷移させる（承認待ち） |
| 承認 API | アクティビティを `submitted → approved` に遷移させる |
| 却下 API | アクティビティを `submitted → rejected` に遷移させる |
| アクティビティイベント配信 | Kafka トピックへのイベントの非同期配信（Outbox pattern） |

### 冪等性保証

`idempotency_key` フィールドを使用して、同一リクエストの重複実行を防ぐ。

- 作成時に `idempotency_key`（クライアントが生成するUUID等）を指定可能
- 同一キーのリクエストは既存レコードを返す（べき等）
- キー未指定時は冪等性なし（都度新規作成）

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Kafka クライアント | rdkafka v0.37 |
| バリデーション | validator v0.18 |

### 配置パス

配置: `regions/service/activity/server/rust/activity/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SVC_ACTIVITY_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/activities` | アクティビティ作成 | `activity:write` |
| GET | `/api/v1/activities` | アクティビティ一覧取得 | `activity:read` |
| GET | `/api/v1/activities/{activity_id}` | アクティビティ詳細取得 | `activity:read` |
| POST | `/api/v1/activities/{activity_id}/submit` | 提出（承認待ち） | `activity:write` |
| POST | `/api/v1/activities/{activity_id}/approve` | 承認 | `activity:approve` |
| POST | `/api/v1/activities/{activity_id}/reject` | 却下 | `activity:approve` |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

---

## アクティビティステータス ステートマシン

```
┌────────┐   ┌───────────┐   ┌──────────┐
│ active │──>│ submitted │──>│ approved │
└────────┘   └─────┬─────┘   └──────────┘
                   │
                   ▼
              ┌──────────┐
              │ rejected │
              └──────────┘
```

| 遷移元 | 遷移先 | 操作 |
| --- | --- | --- |
| active | submitted | submit |
| submitted | approved | approve |
| submitted | rejected | reject |

`approved` と `rejected` は終端ステータス。

## アクティビティタイプ

| タイプ | 説明 |
| --- | --- |
| `comment` | タスクへのコメント |
| `time_entry` | 作業時間の記録（duration_minutes 必須） |
| `status_change` | ステータス変更の記録 |
| `assignment` | 担当者変更の記録 |

---

## エラーコード

| エラーコード | HTTP Status | 説明 |
| --- | --- | --- |
| `SVC_ACTIVITY_NOT_FOUND` | 404 | 指定されたアクティビティが見つからない |
| `SVC_ACTIVITY_VALIDATION_FAILED` | 400 | リクエストのバリデーションエラー |
| `SVC_ACTIVITY_INVALID_STATUS_TRANSITION` | 400 | 不正なステータス遷移 |
| `SVC_ACTIVITY_DUPLICATE_IDEMPOTENCY_KEY` | 200 | 冪等キー重複（既存レコード返却） |
| `SVC_ACTIVITY_INTERNAL_ERROR` | 500 | 内部サーバーエラー |

---

## Kafka イベント

アクティビティのライフサイクルイベントを Outbox pattern で Kafka トピックに非同期配信する。

| トピック | イベント | トリガー |
| --- | --- | --- |
| `k1s0.service.activity.created.v1` | activity.created | アクティビティ作成時 |
| `k1s0.service.activity.approved.v1` | activity.approved | アクティビティ承認時 |

---

## 設定フィールド

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8330` | REST API ポート |
| `grpc_port` | int | `9330` | gRPC ポート |

### database

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `schema` | string | `activity_service` | スキーマ名 |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

| レイヤー | 主要モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Activity`, `ActivityStatus`, `ActivityType` | エンティティ定義 |
| domain/repository | `ActivityRepository` | リポジトリトレイト（find_by_idempotency_key 含む） |
| domain/error | `ActivityError` | ドメインエラー型 |
| usecase | `CreateActivityUseCase`, `GetActivityUseCase`, `ListActivitiesUseCase`, `SubmitActivityUseCase`, `ApproveActivityUseCase`, `RejectActivityUseCase` | ユースケース |
| usecase | `ActivityEventPublisher` | イベント発行トレイト |
| adapter/handler | REST ハンドラー + gRPC サービス | プロトコル変換 |
| infrastructure/persistence | `ActivityPostgresRepository` | PostgreSQL + Outbox + 冪等性 |
| infrastructure/messaging | `ActivityKafkaProducer` | Kafka プロデューサー |

---

## クライアント実装

| プラットフォーム | 配置パス | 技術スタック |
|----------------|---------|-------------|
| React | `regions/service/activity/client/react/activity/` | TanStack Query + Router, Zod, Axios |
| Flutter | `regions/service/activity/client/flutter/activity/` | Riverpod, go_router, Dio |

## 詳細設計ドキュメント

- [service-activity-server-implementation.md](implementation.md) -- Rust 実装詳細
- [service-activity-database.md](database.md) -- データベーススキーマ・マイグレーション
