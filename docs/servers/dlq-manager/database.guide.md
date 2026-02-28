# system-dlq-database 設計ガイド

> **仕様**: テーブル定義・APIスキーマは [database.md](./database.md) を参照。

---

## マイグレーション SQL

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

## アーカイブプロシージャの実行例

```sql
-- 実行例（デフォルト: 30日経過、バッチサイズ1000件）
CALL dlq.archive_old_dlq_messages();

-- カスタムパラメータ（60日経過、バッチサイズ500件）
CALL dlq.archive_old_dlq_messages(p_retention_days := 60, p_batch_size := 500);
```

このプロシージャは CronJob（`infra/kubernetes/system/partition-cronjob.yaml` のパターンを参照）で定期実行する。

---

## パーティション戦略の設計背景

### 現在パーティショニングを適用しない理由

1. **メッセージ数が限定的**: DLQ は異常系のメッセージのみを格納するため、通常運用ではレコード数が少ない
2. **アーカイブプロシージャ**: 30日経過した RESOLVED / DEAD メッセージは自動的にアーカイブテーブルへ移動される
3. **テーブルサイズ管理**: アーカイブにより本テーブルのサイズが一定範囲に保たれる

### 将来的な拡張

DLQ メッセージ量が増加した場合は、以下のパーティション戦略を検討する:

- **dlq_messages**: created_at による月次レンジパーティショニング
- **dlq_messages_archive**: created_at による四半期レンジパーティショニング

---

## インデックス設計方針

- **ステータスインデックス**: リトライ対象の PENDING メッセージ取得が最も頻繁なクエリであるため、status インデックスが最重要
- **トピックインデックス**: 特定トピックの障害状況を調査する際に使用する

---

## 接続設定例

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
