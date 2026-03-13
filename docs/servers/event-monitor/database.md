# system-event-monitor-database 設計

system Tier のイベントモニターデータベース（event-monitor-db）の設計を定義する。
配置先: `regions/system/database/event-monitor-db/`

## 概要

event-monitor-db は system Tier に属する PostgreSQL 17 データベースであり、業務イベントフローの可視化・トランザクション追跡・KPI 集計に必要なデータを管理する。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、event-monitor-db へのアクセスは **system Tier のサーバーからのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli | - |
| ORM / クエリビルダー | sqlx | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## テーブル定義

### flow_definitions テーブル

業務フロー定義（期待されるイベントチェーン）を管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | フロー定義の一意識別子 |
| name | VARCHAR(255) | UNIQUE NOT NULL | フロー名（例: order-fulfillment） |
| description | TEXT | | フローの説明 |
| definition | JSONB | NOT NULL | フロー定義本体（期待イベントチェーン・SLO 定義を含む） |
| version | INT | NOT NULL DEFAULT 1 | 定義バージョン |
| enabled | BOOLEAN | NOT NULL DEFAULT TRUE | 有効フラグ |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

### event_records テーブル

Kafka から集約した業務イベントのレコード。correlation-id ベースのトランザクション追跡に使用する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | イベントレコードの一意識別子 |
| correlation_id | VARCHAR(255) | NOT NULL | 業務相関 ID |
| event_type | VARCHAR(255) | NOT NULL | イベント種別（例: order.created, payment.completed） |
| source_service | VARCHAR(255) | NOT NULL | イベント発生元サービス名 |
| payload | JSONB | | イベントペイロード |
| trace_id | VARCHAR(64) | | OpenTelemetry トレース ID |
| occurred_at | TIMESTAMPTZ | NOT NULL | イベント発生日時 |
| received_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | モニター受信日時 |

### flow_instances テーブル

業務フロー定義に対する実行インスタンス。フロー全体の進捗と状態を管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | フローインスタンスの一意識別子 |
| flow_definition_id | UUID | FK flow_definitions(id) ON DELETE SET NULL | 関連するフロー定義の ID |
| correlation_id | VARCHAR(255) | UNIQUE NOT NULL | 業務相関 ID |
| status | VARCHAR(50) | NOT NULL DEFAULT 'IN_PROGRESS', CHECK制約 | ステータス（IN_PROGRESS / COMPLETED / FAILED / SLO_VIOLATED） |
| started_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | フロー開始日時 |
| completed_at | TIMESTAMPTZ | | フロー完了日時 |
| duration_ms | BIGINT | | フロー完了までの所要時間（ミリ秒） |
| error_message | TEXT | | 失敗時のエラーメッセージ |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | レコード作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

---

## インデックス設計

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| flow_definitions | idx_flow_definitions_name | name | B-tree | フロー名による検索 |
| flow_definitions | idx_flow_definitions_enabled | enabled | B-tree | 有効フラグでのフィルタリング |
| event_records | idx_event_records_correlation_id | correlation_id | B-tree | 相関 ID によるイベント追跡 |
| event_records | idx_event_records_event_type | event_type | B-tree | イベント種別でのフィルタリング |
| event_records | idx_event_records_source_service | source_service | B-tree | 発生元サービスでのフィルタリング |
| event_records | idx_event_records_occurred_at | occurred_at | B-tree | 時系列検索・ソート |
| event_records | idx_event_records_trace_id | trace_id (WHERE NOT NULL) | B-tree（部分） | OpenTelemetry トレース ID 検索 |
| flow_instances | idx_flow_instances_correlation_id | correlation_id | B-tree | 相関 ID によるフローインスタンス検索 |
| flow_instances | idx_flow_instances_flow_definition_id | flow_definition_id | B-tree | フロー定義 ID によるフィルタリング |
| flow_instances | idx_flow_instances_status | status | B-tree | ステータスによるフィルタリング |
| flow_instances | idx_flow_instances_started_at | started_at | B-tree | 開始日時による範囲検索 |

---

## マイグレーションファイル

配置先: `regions/system/database/event-monitor-db/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠する。

```
migrations/
├── 001_create_schema.up.sql                    # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_flow_definitions.up.sql          # flow_definitions テーブル
├── 002_create_flow_definitions.down.sql
├── 003_create_event_records.up.sql             # event_records テーブル
├── 003_create_event_records.down.sql
├── 004_create_flow_instances.up.sql            # flow_instances テーブル
└── 004_create_flow_instances.down.sql
```

### マイグレーション SQL

#### 001_create_schema.up.sql

```sql
-- event-monitor-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE SCHEMA IF NOT EXISTS event_monitor;

CREATE OR REPLACE FUNCTION event_monitor.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

#### 002_create_flow_definitions.up.sql

```sql
-- event-monitor-db: flow_definitions テーブル作成

CREATE TABLE IF NOT EXISTS event_monitor.flow_definitions (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    definition  JSONB        NOT NULL,
    version     INT          NOT NULL DEFAULT 1,
    enabled     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_flow_definitions_name
    ON event_monitor.flow_definitions (name);
CREATE INDEX IF NOT EXISTS idx_flow_definitions_enabled
    ON event_monitor.flow_definitions (enabled);

CREATE TRIGGER trigger_flow_definitions_update_updated_at
    BEFORE UPDATE ON event_monitor.flow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION event_monitor.update_updated_at();
```

#### 003_create_event_records.up.sql

```sql
-- event-monitor-db: event_records テーブル作成

CREATE TABLE IF NOT EXISTS event_monitor.event_records (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    correlation_id  VARCHAR(255) NOT NULL,
    event_type      VARCHAR(255) NOT NULL,
    source_service  VARCHAR(255) NOT NULL,
    payload         JSONB,
    trace_id        VARCHAR(64),
    occurred_at     TIMESTAMPTZ  NOT NULL,
    received_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_records_correlation_id
    ON event_monitor.event_records (correlation_id);
CREATE INDEX IF NOT EXISTS idx_event_records_event_type
    ON event_monitor.event_records (event_type);
CREATE INDEX IF NOT EXISTS idx_event_records_source_service
    ON event_monitor.event_records (source_service);
CREATE INDEX IF NOT EXISTS idx_event_records_occurred_at
    ON event_monitor.event_records (occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_records_trace_id
    ON event_monitor.event_records (trace_id)
    WHERE trace_id IS NOT NULL;
```

#### 004_create_flow_instances.up.sql

```sql
-- event-monitor-db: flow_instances テーブル作成

CREATE TABLE IF NOT EXISTS event_monitor.flow_instances (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flow_definition_id  UUID         REFERENCES event_monitor.flow_definitions(id) ON DELETE SET NULL,
    correlation_id      VARCHAR(255) UNIQUE NOT NULL,
    status              VARCHAR(50)  NOT NULL DEFAULT 'IN_PROGRESS',
    started_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at        TIMESTAMPTZ,
    duration_ms         BIGINT,
    error_message       TEXT,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_flow_instances_status CHECK (status IN ('IN_PROGRESS', 'COMPLETED', 'FAILED', 'SLO_VIOLATED'))
);

CREATE INDEX IF NOT EXISTS idx_flow_instances_correlation_id
    ON event_monitor.flow_instances (correlation_id);
CREATE INDEX IF NOT EXISTS idx_flow_instances_flow_definition_id
    ON event_monitor.flow_instances (flow_definition_id);
CREATE INDEX IF NOT EXISTS idx_flow_instances_status
    ON event_monitor.flow_instances (status);
CREATE INDEX IF NOT EXISTS idx_flow_instances_started_at
    ON event_monitor.flow_instances (started_at);

CREATE TRIGGER trigger_flow_instances_update_updated_at
    BEFORE UPDATE ON event_monitor.flow_instances
    FOR EACH ROW
    EXECUTE FUNCTION event_monitor.update_updated_at();
```

---

## 接続設定

[config設計](../../cli/config/config設計.md) の database セクションに従い、event-monitor-db への接続を設定する。

### 環境別設定

| 環境 | host | ssl_mode | max_open_conns | max_idle_conns |
|------|------|----------|----------------|----------------|
| dev | localhost (docker-compose) | disable | 10 | 3 |
| staging | postgres.k1s0-system.svc.cluster.local | require | 25 | 5 |
| prod | postgres.k1s0-system.svc.cluster.local | verify-full | 50 | 10 |

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/event-monitor/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/system-event-monitor-rw` | Vault Database エンジンで自動生成（TTL: 24時間） |
| 動的クレデンシャル（読み取り専用） | `database/creds/system-event-monitor-ro` | Vault Database エンジンで自動生成（TTL: 24時間） |

### docker-compose（ローカル開発）

[docker-compose設計](../../infrastructure/docker/docker-compose設計.md) の共通 PostgreSQL インスタンスに `k1s0_system` データベースを使用する。event-monitor-db は `k1s0_system` データベース内の `event_monitor` スキーマとして共存する。

---

## 関連ドキュメント

- [system-event-monitor-server設計](server.md) -- イベントモニターサーバー設計
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [可観測性設計](../../architecture/observability/可観測性設計.md) -- OpenTelemetry トレース ID 連携
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md) -- Kafka イベント設計
- [config設計](../../cli/config/config設計.md) -- config.yaml スキーマ（database セクション）
- [テンプレート仕様-データベース](../../templates/data/データベース.md) -- マイグレーション命名規則・テンプレート
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) -- ローカル開発用 PostgreSQL
