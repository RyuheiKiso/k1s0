# system-event-store-server データベース設計

## スキーマ

スキーマ名: `eventstore`

```sql
CREATE SCHEMA IF NOT EXISTS eventstore;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| event_streams | イベントストリーム（集約ルート単位） |
| events | ストリーム内の個別イベント |
| snapshots | ストリームのスナップショット |

---

## ER 図

```
event_streams 1──* events
event_streams 1──* snapshots
```

---

## テーブル定義

### event_streams（イベントストリーム）

集約タイプごとのイベントストリームを管理する。current_version でストリーム内の最新バージョンを追跡する。

```sql
CREATE TABLE IF NOT EXISTS eventstore.event_streams (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type VARCHAR(255) NOT NULL,
    current_version BIGINT      NOT NULL DEFAULT 0,
    metadata       JSONB        NOT NULL DEFAULT '{}',
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_streams_aggregate_type ON eventstore.event_streams (aggregate_type);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー（集約 ID） |
| aggregate_type | VARCHAR(255) | NOT NULL | 集約タイプ |
| current_version | BIGINT | NOT NULL, DEFAULT 0 | 現在のバージョン番号 |
| metadata | JSONB | NOT NULL, DEFAULT '{}' | メタデータ |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### events（イベント）

ストリーム内の個別イベントを格納する。stream_id + sequence の複合ユニーク制約で順序を保証する。

```sql
CREATE TABLE IF NOT EXISTS eventstore.events (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id  UUID         NOT NULL REFERENCES eventstore.event_streams(id) ON DELETE CASCADE,
    sequence   BIGINT       NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    version    BIGINT       NOT NULL DEFAULT 1,
    payload    JSONB        NOT NULL DEFAULT '{}',
    metadata   JSONB        NOT NULL DEFAULT '{}',
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stored_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_events_stream_sequence UNIQUE (stream_id, sequence)
);

CREATE INDEX IF NOT EXISTS idx_events_stream_id ON eventstore.events (stream_id);
CREATE INDEX IF NOT EXISTS idx_events_event_type ON eventstore.events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_stream_sequence ON eventstore.events (stream_id, sequence);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| stream_id | UUID | FK → event_streams.id, NOT NULL | ストリーム ID |
| sequence | BIGINT | UNIQUE(stream_id, sequence), NOT NULL | ストリーム内シーケンス番号 |
| event_type | VARCHAR(255) | NOT NULL | イベント種別 |
| version | BIGINT | NOT NULL, DEFAULT 1 | イベントスキーマバージョン |
| payload | JSONB | NOT NULL, DEFAULT '{}' | イベントペイロード |
| metadata | JSONB | NOT NULL, DEFAULT '{}' | メタデータ |
| occurred_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | イベント発生日時 |
| stored_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 格納日時 |

---

### snapshots（スナップショット）

ストリームの状態スナップショットを保持する。リプレイ高速化のために任意のバージョン時点の集約状態を保存する。

```sql
CREATE TABLE IF NOT EXISTS eventstore.snapshots (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id        UUID         NOT NULL REFERENCES eventstore.event_streams(id) ON DELETE CASCADE,
    snapshot_version BIGINT       NOT NULL,
    aggregate_type   VARCHAR(255) NOT NULL DEFAULT '',
    state            JSONB        NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_snapshots_stream_id ON eventstore.snapshots (stream_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_snapshots_stream_version ON eventstore.snapshots (stream_id, snapshot_version);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| stream_id | UUID | FK → event_streams.id, NOT NULL | ストリーム ID |
| snapshot_version | BIGINT | UNIQUE(stream_id, snapshot_version), NOT NULL | スナップショット時点のバージョン |
| aggregate_type | VARCHAR(255) | NOT NULL, DEFAULT '' | 集約タイプ |
| state | JSONB | NOT NULL, DEFAULT '{}' | スナップショット状態 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/event-store-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `eventstore` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_event_streams.up.sql` | event_streams テーブル作成 |
| `002_create_event_streams.down.sql` | テーブル削除 |
| `003_create_events.up.sql` | events テーブル作成 |
| `003_create_events.down.sql` | テーブル削除 |
| `004_create_snapshots.up.sql` | snapshots テーブル作成 |
| `004_create_snapshots.down.sql` | テーブル削除 |
| `005_add_events_sequence_identity.up.sql` | events テーブルに sequence IDENTITY カラム追加 |
| `006_add_tenant_id_rls.up.sql` | 全テーブルに `tenant_id VARCHAR(255) NOT NULL` と RLS ポリシー追加 |
| `007_add_rls_with_check.up.sql` | RLS ポリシーに AS RESTRICTIVE + WITH CHECK 追加 |
| `008_alter_tenant_id_to_text.up.sql` | `tenant_id` を TEXT 型に変更、RLS ポリシー再作成（CRITICAL-DB-002 対応） |

---

## マルチテナント対応（CRITICAL-DB-002）

全テーブル（`event_streams` / `events` / `snapshots`）の `tenant_id` を TEXT 型に統一（migration 008）。

```sql
-- 各テーブルに設定済み
ALTER TABLE eventstore.{table} ENABLE ROW LEVEL SECURITY;
ALTER TABLE eventstore.{table} FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON eventstore.{table}
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION eventstore.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_event_streams_update_updated_at
    BEFORE UPDATE ON eventstore.event_streams
    FOR EACH ROW EXECUTE FUNCTION eventstore.update_updated_at();
```
