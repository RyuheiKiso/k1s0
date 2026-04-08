# business-board-server データベース設計

> **MED-021 監査対応 — 配置ティアの明確化**
> task / board / activity は **service tier** に属するサービスであり、実装は `regions/service/board/` にある。
> `docs/servers/business/board/` は旧来の配置ミスによる残存ファイルであり、DB 設計はここで管理するが、
> サーバー設計・デプロイ設計は **[service tier 設計書](../../service/board/server.md)** を参照すること。

business Tier のボードサービスデータベース（k1s0_service DB の `board_service` スキーマ）の設計を定義する。
配置先: `regions/service/board/database/postgres/`

> **注意**: board / task / activity の 3 サービスは同一 PostgreSQL データベース（k1s0_service）を共有し、
> スキーマで分離する設計となっている。テナント間の参照・操作リスクは RLS で防止する。

---

## スキーマ

スキーマ名: `board_service`

```sql
CREATE SCHEMA IF NOT EXISTS board_service;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| board_columns | Kanban ボードのカラム定義（project_id × status_code の組み合わせ） |
| outbox_events | ボードカラム変更イベントの Outbox パターン送信キュー |

---

## ER 図

```
board_columns（単独テーブル、外部 FK なし）
outbox_events（単独テーブル、board_columns との FK なし・Outbox パターン）
```

---

## テーブル定義

### board_columns（ボードカラム）

Kanban ボードのカラムを管理する。`project_id × status_code` の組み合わせで一意性を保証する。
`version` フィールドにより楽観的ロックを実現する。
migration 005 で `project_id` が TEXT から UUID 型に変更された（型整合性強化）。

```sql
CREATE TABLE IF NOT EXISTS board_service.board_columns (
    id          UUID         PRIMARY KEY,
    project_id  UUID         NOT NULL,
    status_code TEXT         NOT NULL,
    wip_limit   INTEGER      NOT NULL DEFAULT 0,
    task_count  INTEGER      NOT NULL DEFAULT 0,
    version     INTEGER      NOT NULL DEFAULT 1,
    tenant_id   TEXT         NOT NULL DEFAULT 'system',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_board_columns_project_status UNIQUE (project_id, status_code),
    CONSTRAINT chk_task_count CHECK (task_count >= 0)
);

CREATE INDEX IF NOT EXISTS idx_board_columns_project_id ON board_service.board_columns (project_id);
CREATE INDEX IF NOT EXISTS idx_board_columns_status_code ON board_service.board_columns (status_code);
CREATE INDEX IF NOT EXISTS idx_board_columns_tenant_id ON board_service.board_columns (tenant_id);
CREATE INDEX IF NOT EXISTS idx_board_columns_tenant_project ON board_service.board_columns (tenant_id, project_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| project_id | UUID | UNIQUE(project_id, status_code), NOT NULL | 所属プロジェクト ID |
| status_code | TEXT | UNIQUE(project_id, status_code), NOT NULL | ステータスコード |
| wip_limit | INTEGER | NOT NULL, DEFAULT 0 | WIP 上限（0 = 無制限） |
| task_count | INTEGER | NOT NULL, DEFAULT 0, CHECK (>= 0) | 現在のタスク数 |
| version | INTEGER | NOT NULL, DEFAULT 1 | 楽観的ロック用バージョン番号 |
| tenant_id | TEXT | NOT NULL, DEFAULT 'system' | テナント ID |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

#### RLS（Row Level Security）

migration 003 で RLS を有効化し、migration 006 で FORCE + AS RESTRICTIVE + WITH CHECK を追加した（HIGH-BIZ-002 対応）。

```sql
ALTER TABLE board_service.board_columns ENABLE ROW LEVEL SECURITY;
ALTER TABLE board_service.board_columns FORCE ROW LEVEL SECURITY;

-- テーブルオーナーを含む全ロールに RLS を強制し、INSERT/UPDATE 時のテナント検証も行う
CREATE POLICY tenant_isolation ON board_service.board_columns
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

---

### outbox_events（Outbox イベント）

Outbox パターンによりボードカラム変更イベントを Kafka へ確実に配信するためのキューテーブル。
`published_at IS NULL` の部分インデックスにより未送信イベントの高速スキャンを実現する。

```sql
CREATE TABLE IF NOT EXISTS board_service.outbox_events (
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
    ON board_service.outbox_events (created_at)
    WHERE published_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_outbox_events_tenant_id ON board_service.outbox_events (tenant_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（例: `board_column`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID |
| event_type | TEXT | NOT NULL | イベント種別（例: `BoardColumnCreated`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| tenant_id | TEXT | NOT NULL, DEFAULT 'system' | テナント ID |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 送信完了日時（NULL = 未送信） |

#### RLS（Row Level Security）

migration 004 で RLS を設定（HIGH-005 対応）。バックグラウンドパブリッシャーは `set_config` を呼ばないため
`current_setting` が NULL の場合は全テナントのイベントを参照可能とする設計となっている。

```sql
ALTER TABLE board_service.outbox_events ENABLE ROW LEVEL SECURITY;

-- バックグラウンドパブリッシャー（set_config 未呼出し）は全テナント参照可能
-- アプリケーションコードが set_config() でテナントを設定した場合は当該テナントのみ参照可能
CREATE POLICY tenant_isolation ON board_service.outbox_events
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
| board_columns | あり | あり | あり | なし |
| outbox_events | なし | なし | なし | あり（Outbox パブリッシャー用） |

---

## マイグレーション

マイグレーションファイルは `regions/service/board/database/postgres/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_board_columns.up.sql` | `board_service` スキーマ・`board_columns` テーブル・インデックス作成 |
| `001_create_board_columns.down.sql` | テーブル削除 |
| `002_create_outbox.up.sql` | `outbox_events` テーブル・部分インデックス作成 |
| `002_create_outbox.down.sql` | テーブル削除 |
| `003_add_tenant_id_and_rls.up.sql` | `board_columns` に `tenant_id` カラム追加・RLS 有効化・ポリシー作成 |
| `003_add_tenant_id_and_rls.down.sql` | `tenant_id` カラム・RLS ポリシー削除 |
| `004_add_outbox_rls.up.sql` | `outbox_events` に `tenant_id` カラム追加・RLS 有効化・ポリシー作成（HIGH-005 対応） |
| `004_add_outbox_rls.down.sql` | `tenant_id` カラム・RLS ポリシー削除 |
| `005_project_id_uuid.up.sql` | `board_columns.project_id` を TEXT から UUID 型へ変更（M-007 対応） |
| `005_project_id_uuid.down.sql` | `project_id` を TEXT 型に戻す |
| `006_fix_rls_force_restrictive.up.sql` | `board_columns` に FORCE ROW LEVEL SECURITY・AS RESTRICTIVE・WITH CHECK 追加（HIGH-BIZ-002 対応） |
| `006_fix_rls_force_restrictive.down.sql` | FORCE・RESTRICTIVE・WITH CHECK を削除してポリシーを再作成 |

---

## 関連ドキュメント

- [server.md](server.md) -- Board サーバー設計（API・アーキテクチャ）
- [implementation.md](implementation.md) -- Go 実装詳細
- [task/database.md](../task/database.md) -- Task サービスデータベース設計（同一 DB 内の別スキーマ）
- [activity/database.md](../activity/database.md) -- Activity サービスデータベース設計（同一 DB 内の別スキーマ）
