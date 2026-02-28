# system-dlq-database設計

system Tier のデッドレターキュー管理データベース（dlq-db）の設計を定義する。
配置先: `regions/system/database/dlq-db/`

## 概要

dlq-db は system Tier に属する PostgreSQL 17 データベースであり、Kafka メッセージ処理に失敗したメッセージ（デッドレター）の管理・リトライ・アーカイブを担う。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、dlq-db へのアクセスは **system Tier の dlq-manager サーバーからのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli | - |
| ORM / クエリビルダー | sqlx（Rust） | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## ER図

```
┌──────────────────────┐
│   dlq_messages       │
├──────────────────────┤
│ id (PK)              │
│ original_topic       │
│ error_message        │
│ retry_count          │
│ max_retries          │
│ payload (JSONB)      │
│ status               │
│ created_at           │
│ updated_at           │
│ last_retry_at        │
└──────────────────────┘

┌──────────────────────┐
│ dlq_messages_archive │
├──────────────────────┤
│ (dlq_messages と     │
│  同一スキーマ)       │
└──────────────────────┘
```

### リレーション

| 関係 | 説明 |
|------|------|
| dlq_messages -> dlq_messages_archive | アーカイブプロシージャにより RESOLVED / DEAD 状態のメッセージを移動 |

---

## テーブル定義

### dlq_messages テーブル

Kafka メッセージ処理に失敗したメッセージを格納する。リトライ制御（retry_count / max_retries）とステータス管理（PENDING / RETRYING / RESOLVED / DEAD）を持つ。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | メッセージ識別子 |
| original_topic | VARCHAR(255) | NOT NULL | 失敗元の Kafka トピック名 |
| error_message | TEXT | NOT NULL | エラーメッセージ（処理失敗の理由） |
| retry_count | INT | NOT NULL DEFAULT 0 | 現在のリトライ回数 |
| max_retries | INT | NOT NULL DEFAULT 3 | 最大リトライ回数 |
| payload | JSONB | | 元メッセージのペイロード |
| status | VARCHAR(50) | NOT NULL DEFAULT 'PENDING' | メッセージステータス |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時（DLQ 投入日時） |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |
| last_retry_at | TIMESTAMPTZ | | 最終リトライ実行日時 |

**制約**: `status IN ('PENDING', 'RETRYING', 'RESOLVED', 'DEAD')`

### ステータス遷移

```
PENDING ──> RETRYING ──> PENDING   (リトライ成功せずカウント+1)
                    ──> RESOLVED  (リトライ成功)
                    ──> DEAD      (max_retries 到達)
```

| ステータス | 説明 |
|-----------|------|
| PENDING | リトライ待ち。dlq-manager がリトライ対象として取得する |
| RETRYING | リトライ処理中。dlq-manager が再処理を実行している |
| RESOLVED | リトライ成功。30日後にアーカイブへ移動 |
| DEAD | 最大リトライ回数到達。手動対応が必要 |

### dlq_messages_archive テーブル

dlq_messages と同一スキーマを持つアーカイブテーブル。RESOLVED / DEAD 状態で 30 日経過したメッセージが自動的にアーカイブされる。

`CREATE TABLE dlq.dlq_messages_archive (LIKE dlq.dlq_messages INCLUDING ALL)` で作成される。

---

## マイグレーションファイル

配置先: `regions/system/database/dlq-db/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠する。

```
migrations/
├── 001_create_schema.up.sql                  # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_dlq_messages.up.sql            # dlq_messages テーブル・インデックス・トリガー
├── 002_create_dlq_messages.down.sql
├── 003_add_partition_management.up.sql       # アーカイブテーブル・アーカイブプロシージャ
└── 003_add_partition_management.down.sql
```

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_schema | pgcrypto 拡張・dlq スキーマ・`update_updated_at()` 関数の作成 |
| 002 | create_dlq_messages | dlq_messages テーブル・インデックス・updated_at トリガー |
| 003 | add_partition_management | dlq_messages_archive テーブル作成・バッチ対応アーカイブプロシージャ |

### 001_create_schema.up.sql

```sql
-- dlq-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS dlq;

CREATE OR REPLACE FUNCTION dlq.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 002_create_dlq_messages.up.sql

dlq_messages テーブル・インデックス・updated_at トリガーを作成する。

### 003_add_partition_management.up.sql

dlq_messages_archive テーブルと、バッチ処理対応のアーカイブプロシージャを作成する。

```sql
-- アーカイブ用テーブル
CREATE TABLE IF NOT EXISTS dlq.dlq_messages_archive (
    LIKE dlq.dlq_messages INCLUDING ALL
);

-- アーカイブ実行プロシージャ（バッチ処理対応）
CREATE OR REPLACE PROCEDURE dlq.archive_old_dlq_messages(
    p_retention_days INT DEFAULT 30,
    p_batch_size INT DEFAULT 1000
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_cutoff TIMESTAMPTZ;
    v_archived_count INT := 0;
    v_batch_count INT;
BEGIN
    v_cutoff := NOW() - (p_retention_days || ' days')::INTERVAL;

    LOOP
        WITH to_archive AS (
            SELECT id FROM dlq.dlq_messages
            WHERE status IN ('RESOLVED', 'DEAD') AND updated_at < v_cutoff
            LIMIT p_batch_size FOR UPDATE SKIP LOCKED
        ),
        archived AS (
            INSERT INTO dlq.dlq_messages_archive
            SELECT m.* FROM dlq.dlq_messages m
            INNER JOIN to_archive ta ON m.id = ta.id
            RETURNING 1
        )
        SELECT COUNT(*) INTO v_batch_count FROM archived;

        DELETE FROM dlq.dlq_messages
        WHERE id IN (
            SELECT id FROM dlq.dlq_messages
            WHERE status IN ('RESOLVED', 'DEAD') AND updated_at < v_cutoff
            LIMIT p_batch_size
        );

        v_archived_count := v_archived_count + v_batch_count;
        EXIT WHEN v_batch_count = 0;
        COMMIT;
    END LOOP;

    RAISE NOTICE 'Archived % DLQ messages older than % days',
        v_archived_count, p_retention_days;
END;
$$;
```

---

## インデックス設計

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| dlq_messages | idx_dlq_messages_original_topic | original_topic | B-tree | トピック別のメッセージ検索 |
| dlq_messages | idx_dlq_messages_status | status | B-tree | ステータス別のメッセージ検索（リトライ対象取得） |
| dlq_messages | idx_dlq_messages_created_at | created_at | B-tree | 作成日時による範囲検索 |

### 設計方針

- **ステータスインデックス**: リトライ対象の PENDING メッセージ取得が最も頻繁なクエリであるため、status インデックスが最重要
- **トピックインデックス**: 特定トピックの障害状況を調査する際に使用する

---

## パーティション戦略

### 現在の設計

dlq_messages テーブルは現時点ではパーティショニングを適用しない。理由は以下の通り:

1. **メッセージ数が限定的**: DLQ は異常系のメッセージのみを格納するため、通常運用ではレコード数が少ない
2. **アーカイブプロシージャ**: 30日経過した RESOLVED / DEAD メッセージは `dlq.archive_old_dlq_messages()` プロシージャで自動的にアーカイブテーブルへ移動される
3. **テーブルサイズ管理**: アーカイブにより本テーブルのサイズが一定範囲に保たれる

### 将来的な拡張

DLQ メッセージ量が増加した場合は、以下のパーティション戦略を検討する:

- **dlq_messages**: created_at による月次レンジパーティショニング
- **dlq_messages_archive**: created_at による四半期レンジパーティショニング

---

## マイグレーションファイル

配置先: `regions/system/database/dlq-db/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠する。

```
migrations/
├── 001_create_schema.up.sql                    # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_dlq_messages.up.sql              # dlq_messages テーブル + インデックス + トリガー
├── 002_create_dlq_messages.down.sql
├── 003_add_partition_management.up.sql         # dlq_messages_archive テーブル + アーカイブプロシージャ
└── 003_add_partition_management.down.sql
```

### 001_create_schema.up.sql

```sql
-- dlq-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS dlq;

CREATE OR REPLACE FUNCTION dlq.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 001_create_schema.down.sql

```sql
DROP FUNCTION IF EXISTS dlq.update_updated_at();
DROP SCHEMA IF EXISTS dlq CASCADE;
DROP EXTENSION IF EXISTS "pgcrypto";
```

### 002_create_dlq_messages.up.sql

```sql
-- dlq-db: dlq_messages テーブル作成

CREATE TABLE IF NOT EXISTS dlq.dlq_messages (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    original_topic  VARCHAR(255) NOT NULL,
    error_message   TEXT         NOT NULL,
    retry_count     INT          NOT NULL DEFAULT 0,
    max_retries     INT          NOT NULL DEFAULT 3,
    payload         JSONB,
    status          VARCHAR(50)  NOT NULL DEFAULT 'PENDING',
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_retry_at   TIMESTAMPTZ,

    CONSTRAINT chk_dlq_messages_status CHECK (status IN ('PENDING', 'RETRYING', 'RESOLVED', 'DEAD'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_dlq_messages_original_topic ON dlq.dlq_messages (original_topic);
CREATE INDEX IF NOT EXISTS idx_dlq_messages_status ON dlq.dlq_messages (status);
CREATE INDEX IF NOT EXISTS idx_dlq_messages_created_at ON dlq.dlq_messages (created_at);

-- updated_at トリガー
CREATE TRIGGER trg_dlq_messages_updated_at
    BEFORE UPDATE ON dlq.dlq_messages
    FOR EACH ROW
    EXECUTE FUNCTION dlq.update_updated_at();
```

### 002_create_dlq_messages.down.sql

```sql
DROP TRIGGER IF EXISTS trg_dlq_messages_updated_at ON dlq.dlq_messages;
DROP TABLE IF EXISTS dlq.dlq_messages;
```

### 003_add_partition_management.up.sql

```sql
-- dlq-db: 古い DLQ メッセージのアーカイブ管理

-- アーカイブ用テーブル
CREATE TABLE IF NOT EXISTS dlq.dlq_messages_archive (
    LIKE dlq.dlq_messages INCLUDING ALL
);

-- アーカイブ実行プロシージャ（バッチ処理対応）
CREATE OR REPLACE PROCEDURE dlq.archive_old_dlq_messages(
    p_retention_days INT DEFAULT 30,
    p_batch_size INT DEFAULT 1000
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_cutoff TIMESTAMPTZ;
    v_archived_count INT := 0;
    v_batch_count INT;
BEGIN
    v_cutoff := NOW() - (p_retention_days || ' days')::INTERVAL;

    LOOP
        WITH to_archive AS (
            SELECT id FROM dlq.dlq_messages
            WHERE status IN ('RESOLVED', 'DEAD')
              AND updated_at < v_cutoff
            LIMIT p_batch_size
            FOR UPDATE SKIP LOCKED
        ),
        archived AS (
            INSERT INTO dlq.dlq_messages_archive
            SELECT m.* FROM dlq.dlq_messages m
            INNER JOIN to_archive ta ON m.id = ta.id
            RETURNING 1
        )
        SELECT COUNT(*) INTO v_batch_count FROM archived;

        DELETE FROM dlq.dlq_messages
        WHERE id IN (
            SELECT id FROM dlq.dlq_messages
            WHERE status IN ('RESOLVED', 'DEAD')
              AND updated_at < v_cutoff
            LIMIT p_batch_size
        );

        v_archived_count := v_archived_count + v_batch_count;
        EXIT WHEN v_batch_count = 0;
        COMMIT;
    END LOOP;

    RAISE NOTICE 'Archived % DLQ messages older than % days', v_archived_count, p_retention_days;
END;
$$;
```

### 003_add_partition_management.down.sql

```sql
DROP PROCEDURE IF EXISTS dlq.archive_old_dlq_messages(INT, INT);
DROP TABLE IF EXISTS dlq.dlq_messages_archive;
```

---

## リテンションポリシー

| 対象 | 保持期間 | 処理 |
|------|---------|------|
| dlq_messages (PENDING / RETRYING) | 無期限 | リトライ処理が完了するまで保持 |
| dlq_messages (RESOLVED / DEAD) | 30日 | アーカイブプロシージャで dlq_messages_archive へ移動 |
| dlq_messages_archive | 365日 | 年次で古いレコードを削除（手動または cron） |

### アーカイブプロシージャ

実装は `003_add_partition_management.up.sql` で定義された `dlq.archive_old_dlq_messages()` プロシージャを使用する。バッチ処理対応により、大量メッセージのアーカイブ時もロック時間を最小化する。

```sql
-- 実行例（デフォルト: 30日経過、バッチサイズ1000件）
CALL dlq.archive_old_dlq_messages();

-- カスタムパラメータ（60日経過、バッチサイズ500件）
CALL dlq.archive_old_dlq_messages(p_retention_days := 60, p_batch_size := 500);
```

このプロシージャは CronJob（`infra/kubernetes/system/partition-cronjob.yaml` のパターンを参照）で定期実行する。

---

## 主要クエリパターン

### リトライ対象メッセージの取得

```sql
-- PENDING 状態のメッセージを取得（リトライ対象）
SELECT id, original_topic, error_message, retry_count, max_retries, payload
FROM dlq.dlq_messages
WHERE status = 'PENDING'
  AND retry_count < max_retries
ORDER BY created_at ASC
LIMIT $1
FOR UPDATE SKIP LOCKED;
```

### リトライ実行

```sql
-- ステータスを RETRYING に更新
UPDATE dlq.dlq_messages
SET status = 'RETRYING', last_retry_at = NOW()
WHERE id = $1;

-- リトライ成功
UPDATE dlq.dlq_messages
SET status = 'RESOLVED'
WHERE id = $1;

-- リトライ失敗（カウント+1）
UPDATE dlq.dlq_messages
SET retry_count = retry_count + 1,
    status = CASE WHEN retry_count + 1 >= max_retries THEN 'DEAD' ELSE 'PENDING' END
WHERE id = $1;
```

### トピック別の障害状況

```sql
-- トピック別の DLQ メッセージ数
SELECT original_topic, status, COUNT(*) as count
FROM dlq.dlq_messages
GROUP BY original_topic, status
ORDER BY original_topic, status;
```

---

## 接続設定

### config.yaml（dlq-manager サーバー用）

```yaml
app:
  name: "dlq-manager"
  version: "1.0.0"
  tier: "system"
  environment: "dev"

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "dlq_db"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 10
  max_idle_conns: 3
  conn_max_lifetime: "5m"
```

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/dlq-manager/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/dlq-manager-rw` | TTL: 24時間 |
| 動的クレデンシャル（読み取り専用） | `database/creds/dlq-manager-ro` | TTL: 24時間 |

---

## 関連ドキュメント

- [system-database設計](../_common/database.md) -- auth-db テーブル設計（参照パターン）
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) -- ローカル開発用 PostgreSQL
- [可観測性設計](../../architecture/observability/可観測性設計.md) -- OpenTelemetry トレース ID 連携
