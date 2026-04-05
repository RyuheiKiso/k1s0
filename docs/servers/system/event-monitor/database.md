# system-event-monitor-database 設計

system Tier のイベントモニターデータベース（event_monitor スキーマ）の設計を定義する。

> **権威ソース**: テーブル定義は `regions/system/server/rust/event-monitor/src/adapter/repository/` の Rust コードが正である。このドキュメントはそこから導出したものであり、食い違いがある場合は Rust 実装を優先すること。

## 概要

event_monitor スキーマは `k1s0_event_monitor` データベース内に作成される専用スキーマである（CRIT-001 監査対応: k1s0_system からの DB 分離）。system Tier のイベントモニターサービスが業務イベントフローの可視化・トランザクション追跡・KPI 集計に使用する。

`infra/docker/init-db/18-event-monitor-schema.sql` によって `docker-compose up` 時に自動的にスキーマ・テーブルが作成される。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、event_monitor スキーマへのアクセスは **system Tier のサーバーからのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| クエリ | sqlx（マイグレーション機能は未使用。init-db で初期化） | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## テーブル定義

### flow_definitions テーブル

業務フロー定義（期待されるイベントチェーン・SLO 定義）を管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | フロー定義の一意識別子 |
| name | VARCHAR(255) | UNIQUE NOT NULL | フロー名（例: task-assignment） |
| description | TEXT | NOT NULL DEFAULT '' | フローの説明 |
| domain | VARCHAR(255) | NOT NULL | ビジネスドメイン名 |
| steps | JSONB | NOT NULL DEFAULT '[]' | フロー手順列（FlowStep 配列: event_type / source / timeout_seconds / description） |
| slo_target_completion_secs | INT | NOT NULL DEFAULT 0 | SLO: 完了目標秒数 |
| slo_target_success_rate | DOUBLE PRECISION | NOT NULL DEFAULT 0.99 | SLO: 目標成功率（0.0〜1.0） |
| slo_alert_on_violation | BOOLEAN | NOT NULL DEFAULT TRUE | SLO 違反時にアラートを送出するか |
| enabled | BOOLEAN | NOT NULL DEFAULT TRUE | 有効フラグ |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

### event_records テーブル

Kafka から集約した業務イベントのレコード。correlation_id ベースのトランザクション追跡に使用する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | イベントレコードの一意識別子 |
| correlation_id | VARCHAR(255) | NOT NULL | 業務相関 ID |
| event_type | VARCHAR(255) | NOT NULL | イベント種別（例: task.created） |
| source | VARCHAR(255) | NOT NULL | イベント発生元サービス名 |
| domain | VARCHAR(255) | NOT NULL | ビジネスドメイン名 |
| trace_id | VARCHAR(64) | NOT NULL DEFAULT '' | OpenTelemetry トレース ID（未設定時は空文字） |
| timestamp | TIMESTAMPTZ | NOT NULL | イベント発生日時 |
| flow_id | UUID | FK flow_definitions(id) ON DELETE SET NULL, nullable | マッチしたフロー定義 ID |
| flow_step_index | INT | nullable | フロー内のステップ番号 |
| status | VARCHAR(50) | NOT NULL DEFAULT 'normal' | イベントステータス（'normal' 等） |
| received_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | モニター受信日時 |

### flow_instances テーブル

業務フロー定義に対する実行インスタンス。フロー全体の進捗と状態を管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | フローインスタンスの一意識別子 |
| flow_id | UUID | NOT NULL, FK flow_definitions(id) ON DELETE CASCADE | 関連するフロー定義の ID（`flow_definition_id` ではない） |
| correlation_id | VARCHAR(255) | UNIQUE NOT NULL | 業務相関 ID |
| status | VARCHAR(50) | NOT NULL DEFAULT 'in_progress', CHECK制約 | ステータス（in_progress / completed / failed / timeout） |
| current_step_index | INT | NOT NULL DEFAULT 0 | 現在処理中のステップ番号 |
| started_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | フロー開始日時 |
| completed_at | TIMESTAMPTZ | nullable | フロー完了日時 |
| duration_ms | BIGINT | nullable | フロー完了までの所要時間（ミリ秒） |

---

## インデックス設計

| テーブル | インデックス名 | カラム | 用途 |
|----------|---------------|--------|------|
| flow_definitions | idx_flow_definitions_name | name | フロー名による検索 |
| flow_definitions | idx_flow_definitions_domain | domain | ドメインでのフィルタリング |
| flow_definitions | idx_flow_definitions_enabled | enabled | 有効フラグでのフィルタリング |
| event_records | idx_event_records_correlation_id | correlation_id | 相関 ID によるイベント追跡 |
| event_records | idx_event_records_event_type | event_type | イベント種別でのフィルタリング |
| event_records | idx_event_records_source | source | 発生元サービスでのフィルタリング |
| event_records | idx_event_records_domain | domain | ドメインでのフィルタリング |
| event_records | idx_event_records_timestamp | timestamp | 時系列検索・ソート |
| event_records | idx_event_records_flow_id | flow_id (WHERE NOT NULL) | フロー別イベント検索（部分インデックス） |
| flow_instances | idx_flow_instances_flow_id | flow_id | フロー定義 ID によるフィルタリング |
| flow_instances | idx_flow_instances_correlation_id | correlation_id | 相関 ID によるフローインスタンス検索 |
| flow_instances | idx_flow_instances_status | status | ステータスによるフィルタリング |
| flow_instances | idx_flow_instances_started_at | started_at | 開始日時による範囲検索 |

---

## 接続設定

### 環境別設定

| 環境 | host | ssl_mode | max_open_conns |
|------|------|----------|----------------|
| dev | postgres (docker-compose) | disable | 10 |
| staging | postgres.k1s0-system.svc.cluster.local | require | 25 |
| prod | postgres.k1s0-system.svc.cluster.local | verify-full | 50 |

### docker-compose（ローカル開発）

共通 PostgreSQL インスタンスの `k1s0_system` データベースに `event_monitor` スキーマとして共存する。

接続 URL: `postgres://k1s0:dev-k1s0-local@postgres:5432/k1s0_system?sslmode=disable`

スキーマは `infra/docker/init-db/18-event-monitor-schema.sql` によって自動作成される。

### Vault によるクレデンシャル管理

| 用途 | Vault パス |
|------|-----------|
| 静的パスワード | `secret/data/k1s0/system/event-monitor/database` |
| 動的クレデンシャル（読み書き） | `database/creds/system-event-monitor-rw` |

---

## 関連ドキュメント

- [system-event-monitor-server設計](server.md) — イベントモニターサーバー設計
- [tier-architecture](../../architecture/overview/tier-architecture.md) — Tier アーキテクチャ・データベースアクセスルール
- [可観測性設計](../../architecture/observability/可観測性設計.md) — OpenTelemetry トレース ID 連携
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md) — Kafka イベント設計
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) — ローカル開発用 PostgreSQL
