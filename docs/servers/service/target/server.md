# service-target-server 設計

service tier のターゲット（OKR目標）管理サーバー設計を定義する。組織・チーム・個人の OKR 目標（Objective and Key Results）を REST/gRPC API で管理し、進捗更新イベントを Kafka に非同期配信する。
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
| `read` | GET（目標一覧・詳細取得） |
| `write` | POST / PUT / PATCH（作成・更新・進捗更新） |
| `admin` | DELETE（目標削除）、PUT（クローズ・アーカイブ） |

実装: `adapter/middleware/rbac.rs` の `require_permission` + `k1s0-server-common` の `check_permission(Tier::Service, ...)` を使用。認証は Bearer JWT 検証（JWKS）。`/healthz`・`/readyz`・`/metrics` は認証除外。

service tier のターゲット管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| 目標作成 API | Objective（目標）と Key Results（主要成果）を作成する |
| 目標一覧取得 API | プロジェクト・オーナー・ステータスによるフィルタリング付きの一覧を取得する |
| 目標詳細取得 API | 目標 ID を指定して Objective と Key Results を取得する |
| 進捗更新 API | Key Result の進捗値（0〜100）を更新する |
| 目標クローズ API | 目標を `open → closed` に遷移させる |
| 目標アーカイブ API | 目標を `closed → archived` に遷移させる |
| 進捗更新イベント配信 | Kafka トピックへの進捗更新イベントの非同期配信（Outbox pattern） |

### 目標ステータス ステートマシン

```
┌──────┐   ┌────────┐   ┌──────────┐
│ open │──>│ closed │──>│ archived │
└──────┘   └────────┘   └──────────┘
```

| 遷移元 | 遷移先 | 操作 |
| --- | --- | --- |
| open | closed | close |
| closed | archived | archive |

`archived` は終端ステータス。

### Key Result 進捗ルール

- `progress` は 0〜100 の整数値（パーセンテージ）
- Objective の進捗は Key Results の平均値として自動計算する
- `progress = 100` かつ Objective の全 Key Results が 100 の場合、Objective を自動的に `closed` に遷移することを推奨するが、最終判断は API 呼び出し元に委ねる

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Kafka クライアント | rdkafka v0.37 |
| バリデーション | validator v0.18 |

### 配置パス

配置: `regions/service/target/server/rust/target/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SVC_TARGET_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/objectives` | 目標（Objective）作成 | `target:write` |
| GET | `/api/v1/objectives` | 目標一覧取得 | `target:read` |
| GET | `/api/v1/objectives/{objective_id}` | 目標詳細取得 | `target:read` |
| PUT | `/api/v1/objectives/{objective_id}` | 目標更新 | `target:write` |
| POST | `/api/v1/objectives/{objective_id}/close` | 目標クローズ | `target:admin` |
| POST | `/api/v1/objectives/{objective_id}/archive` | 目標アーカイブ | `target:admin` |
| PATCH | `/api/v1/objectives/{objective_id}/key-results/{kr_id}/progress` | Key Result 進捗更新 | `target:write` |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

---

## エラーコード

| エラーコード | HTTP Status | 説明 |
| --- | --- | --- |
| `SVC_TARGET_NOT_FOUND` | 404 | 指定された目標が見つからない |
| `SVC_TARGET_KEY_RESULT_NOT_FOUND` | 404 | 指定された Key Result が見つからない |
| `SVC_TARGET_VALIDATION_FAILED` | 400 | リクエストのバリデーションエラー |
| `SVC_TARGET_INVALID_STATUS_TRANSITION` | 400 | 不正なステータス遷移 |
| `SVC_TARGET_INVALID_PROGRESS` | 400 | 進捗値が 0〜100 の範囲外 |
| `SVC_TARGET_VERSION_CONFLICT` | 409 | 楽観的ロックによるバージョン競合 |
| `SVC_TARGET_INTERNAL_ERROR` | 500 | 内部サーバーエラー |

---

## Kafka イベント

目標の進捗・ステータス変更イベントを Outbox pattern で Kafka トピックに非同期配信する。

| トピック | イベント | トリガー |
| --- | --- | --- |
| `k1s0.service.target.progress_updated.v1` | target.progress_updated | Key Result 進捗更新時 |
| `k1s0.service.target.status_changed.v1` | target.status_changed | 目標ステータス変更時（close/archive） |

---

## 設定フィールド

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8340` | REST API ポート |
| `grpc_port` | int | `9340` | gRPC ポート |

### database

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `schema` | string | `target_service` | スキーマ名 |

### kafka

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `brokers` | string[] | `["kafka:9092"]` | Kafka ブローカーアドレス一覧 |
| `target_progress_updated_topic` | string | `k1s0.service.target.progress_updated.v1` | 進捗更新イベントのトピック名 |
| `target_status_changed_topic` | string | `k1s0.service.target.status_changed.v1` | ステータス変更イベントのトピック名 |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

| レイヤー | 主要モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Objective`, `KeyResult`, `ObjectiveStatus` | エンティティ定義（進捗計算ロジック含む） |
| domain/repository | `ObjectiveRepository`, `KeyResultRepository` | リポジトリトレイト |
| domain/error | `TargetError` | ドメインエラー型 |
| usecase | `CreateObjectiveUseCase`, `GetObjectiveUseCase`, `ListObjectivesUseCase`, `UpdateObjectiveUseCase`, `CloseObjectiveUseCase`, `ArchiveObjectiveUseCase`, `UpdateKeyResultProgressUseCase` | ユースケース |
| usecase | `TargetEventPublisher` | イベント発行トレイト |
| adapter/handler | REST ハンドラー + gRPC サービス | プロトコル変換 |
| infrastructure/persistence | `ObjectivePostgresRepository`, `KeyResultPostgresRepository` | PostgreSQL + Outbox |
| infrastructure/messaging | `TargetKafkaProducer` | Kafka プロデューサー |

---

## クライアント実装

| プラットフォーム | 配置パス | 技術スタック |
|----------------|---------|-------------|
| React | `regions/service/target/client/react/target/` | TanStack Query + Router, Zod, Axios |
| Flutter | `regions/service/target/client/flutter/target/` | Riverpod, go_router, Dio |

## 詳細設計ドキュメント

- [service-target-server-implementation.md](implementation.md) -- Rust 実装詳細
- [service-target-database.md](database.md) -- データベーススキーマ・マイグレーション
