# business-activity-server データベース設計

business Tier のアクティビティサービスデータベース（k1s0_service DB の `activity_service` スキーマ）の設計を定義する。
配置先: `regions/service/activity/database/postgres/`

> **注意**: board / task / activity の 3 サービスは同一 PostgreSQL データベース（k1s0_service）を共有し、
> スキーマで分離する設計となっている。テナント間の参照・操作リスクは RLS で防止する。

---

## スキーマ

スキーマ名: `activity_service`

```sql
CREATE SCHEMA IF NOT EXISTS activity_service;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| activities | タスクへのコメント・作業時間記録・ステータス変更等の操作履歴 |
| outbox_events | アクティビティ変更イベントの Outbox パターン送信キュー |

---

## ER 図

```
activities（単独テーブル。task_id は task_service.tasks を論理的に参照するが、
           スキーマ横断 FK は設けない設計。クロスサービス FK 不在の設計根拠は ADR 参照）
outbox_events（単独テーブル、activities との FK なし・Outbox パターン）
```

> **設計根拠**: クロスサービス FK を設けない理由は `docs/architecture/adr/` を参照。
> `activities.task_id` は UUID 型で task_service との整合性はアプリケーション層で担保する。

---

## テーブル定義

### activities（アクティビティ）

タスクに対するコメント・作業時間・ステータス変更等の操作履歴を管理する。
`idempotency_key` による冪等性保証を提供する。
migration 006 で `task_id` が TEXT から UUID 型に変更された（H-008 対応、型整合性強化）。

```sql
CREATE TABLE IF NOT EXISTS activity_service.activities (
    id               UUID         PRIMARY KEY,
    task_id          UUID         NOT NULL,
    actor_id         TEXT         NOT NULL,
    activity_type    TEXT         NOT NULL,
    content          TEXT,
    duration_minutes INTEGER,
    status           TEXT         NOT NULL DEFAULT 'active',
    metadata         JSONB,
    idempotency_key  TEXT         UNIQUE,
    version          INTEGER      NOT NULL DEFAULT 1,
    tenant_id        TEXT         NOT NULL DEFAULT 'system',
    updated_by       TEXT,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activities_task_id ON activity_service.activities (task_id);
CREATE INDEX IF NOT EXISTS idx_activities_actor_id ON activity_service.activities (actor_id);
CREATE INDEX IF NOT EXISTS idx_activities_activity_type ON activity_service.activities (activity_type);
CREATE INDEX IF NOT EXISTS idx_activities_status ON activity_service.activities (status);
CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activity_service.activities (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activities_tenant_id ON activity_service.activities (tenant_id);
CREATE INDEX IF NOT EXISTS idx_activities_tenant_task ON activity_service.activities (tenant_id, task_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| task_id | UUID | NOT NULL | 対象タスク ID（task_service.tasks.id を論理参照） |
| actor_id | TEXT | NOT NULL | 操作者 ID |
| activity_type | TEXT | NOT NULL | アクティビティ種別（例: `comment` / `time_log` / `status_change`） |
| content | TEXT | | アクティビティの内容（コメント本文等） |
| duration_minutes | INTEGER | | 作業時間（分）。time_log 種別で使用 |
| status | TEXT | NOT NULL, DEFAULT 'active' | ステータス（active / deleted 等） |
| metadata | JSONB | | 追加メタデータ（種別ごとに構造が異なる） |
| idempotency_key | TEXT | UNIQUE | 冪等性保証キー（重複リクエスト防止） |
| version | INTEGER | NOT NULL, DEFAULT 1 | 楽観的ロック用バージョン番号 |
| tenant_id | TEXT | NOT NULL, DEFAULT 'system' | テナント ID |
| updated_by | TEXT | | 最終更新者 ID（HIGH-07 監査対応で追加） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

#### RLS（Row Level Security）

migration 003 で RLS を有効化し、migration 007 で FORCE + AS RESTRICTIVE + WITH CHECK を追加した（HIGH-BIZ-002 対応）。

```sql
ALTER TABLE activity_service.activities ENABLE ROW LEVEL SECURITY;
ALTER TABLE activity_service.activities FORCE ROW LEVEL SECURITY;

-- テーブルオーナーを含む全ロールに RLS を強制し、INSERT/UPDATE 時のテナント検証も行う
CREATE POLICY tenant_isolation ON activity_service.activities
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

---

### outbox_events（Outbox イベント）

Outbox パターンによりアクティビティ変更イベントを Kafka へ確実に配信するためのキューテーブル。
`published_at IS NULL` の部分インデックスにより未送信イベントの高速スキャンを実現する。

```sql
CREATE TABLE IF NOT EXISTS activity_service.outbox_events (
    id             UUID         PRIMARY KEY,
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    tenant_id      TEXT         NOT NULL DEFAULT 'system',
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_outbox_unpublished
    ON activity_service.outbox_events (created_at)
    WHERE published_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_outbox_events_tenant_id ON activity_service.outbox_events (tenant_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（例: `activity`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID |
| event_type | TEXT | NOT NULL | イベント種別（例: `ActivityCreated`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| tenant_id | TEXT | NOT NULL, DEFAULT 'system' | テナント ID |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 送信完了日時（NULL = 未送信） |

#### RLS（Row Level Security）

migration 005 で RLS を設定（HIGH-005 対応）。バックグラウンドパブリッシャーは `set_config` を呼ばないため
`current_setting` が NULL の場合は全テナントのイベントを参照可能とする設計となっている。

```sql
ALTER TABLE activity_service.outbox_events ENABLE ROW LEVEL SECURITY;

-- バックグラウンドパブリッシャー（set_config 未呼出し）は全テナント参照可能
CREATE POLICY tenant_isolation ON activity_service.outbox_events
    USING (
        current_setting('app.current_tenant_id', true) IS NULL
        OR tenant_id = current_setting('app.current_tenant_id', true)::TEXT
    );
```

---

## マルチテナント対応

全テーブルに `tenant_id TEXT NOT NULL DEFAULT 'system'` カラムと RLS ポリシーを設定する。
コンテキスト設定は `set_config('app.current_tenant_id', tenant_id, true)` を各 DB 操作前に実行する。

### RLS コンテキスト設定（Go サービス側）

```go
// 各DBクエリの前にセッション変数でテナントIDを設定する
_, err := tx.ExecContext(ctx,
    "SELECT set_config('app.current_tenant_id', $1, true)",
    tenantID,
)
```

### RLS ポリシー設計の差異

| テーブル | FORCE | AS RESTRICTIVE | WITH CHECK | バックグラウンド NULL 許可 |
| --- | --- | --- | --- | --- |
| activities | あり | あり | あり | なし |
| outbox_events | なし | なし | なし | あり（Outbox パブリッシャー用） |

---

## マイグレーション

マイグレーションファイルは `regions/service/activity/database/postgres/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_activities.up.sql` | `activity_service` スキーマ・`activities` テーブル・インデックス作成 |
| `001_create_activities.down.sql` | テーブル削除 |
| `002_create_outbox.up.sql` | `outbox_events` テーブル・部分インデックス作成 |
| `002_create_outbox.down.sql` | テーブル削除 |
| `003_add_tenant_id_and_rls.up.sql` | `activities` に `tenant_id` カラム追加・RLS 有効化・ポリシー作成 |
| `003_add_tenant_id_and_rls.down.sql` | `tenant_id` カラム・RLS ポリシー削除 |
| `004_add_updated_by.up.sql` | `activities` に `updated_by TEXT` カラム追加（HIGH-07 監査対応） |
| `004_add_updated_by.down.sql` | `updated_by` カラム削除 |
| `005_add_outbox_rls.up.sql` | `outbox_events` に `tenant_id` カラム追加・RLS 有効化・ポリシー作成（HIGH-005 対応） |
| `005_add_outbox_rls.down.sql` | `tenant_id` カラム・RLS ポリシー削除 |
| `006_fix_task_id_type.up.sql` | `activities.task_id` を TEXT から UUID 型へ変更。不正値は NULL に変換（H-008 対応） |
| `006_fix_task_id_type.down.sql` | `task_id` を TEXT 型に戻す |
| `007_fix_rls_force_restrictive.up.sql` | `activities` に FORCE ROW LEVEL SECURITY・AS RESTRICTIVE・WITH CHECK 追加（HIGH-BIZ-002 対応） |
| `007_fix_rls_force_restrictive.down.sql` | FORCE・RESTRICTIVE・WITH CHECK を削除してポリシーを再作成 |

---

## 関連ドキュメント

- [server.md](server.md) -- Activity サーバー設計（API・アーキテクチャ）
- [implementation.md](implementation.md) -- Go 実装詳細
- [board/database.md](../board/database.md) -- Board サービスデータベース設計（同一 DB 内の別スキーマ）
- [task/database.md](../task/database.md) -- Task サービスデータベース設計（同一 DB 内の別スキーマ）
