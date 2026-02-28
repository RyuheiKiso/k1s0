# system-saga-database 設計ガイド

> **仕様**: テーブル定義・APIスキーマは [database.md](./database.md) を参照。

---

## マイグレーション SQL

### 001_create_schema.up.sql

```sql
-- saga-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

-- 拡張機能
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- スキーマ
CREATE SCHEMA IF NOT EXISTS saga;

-- updated_at 自動更新関数
CREATE OR REPLACE FUNCTION saga.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 001_create_schema.down.sql

```sql
DROP FUNCTION IF EXISTS saga.update_updated_at();
DROP SCHEMA IF EXISTS saga CASCADE;
DROP EXTENSION IF EXISTS "pgcrypto";
```

### 002_create_saga_states.up.sql

```sql
-- saga-db: saga_states テーブル作成

CREATE TABLE IF NOT EXISTS saga.saga_states (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_name   VARCHAR(255) NOT NULL,
    current_step    INT          NOT NULL DEFAULT 0,
    status          VARCHAR(50)  NOT NULL DEFAULT 'STARTED',
    payload         JSONB,
    correlation_id  VARCHAR(255),
    initiated_by    VARCHAR(255),
    error_message   TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_saga_states_status CHECK (status IN ('STARTED', 'RUNNING', 'COMPLETED', 'COMPENSATING', 'FAILED', 'CANCELLED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_saga_states_workflow_name ON saga.saga_states (workflow_name);
CREATE INDEX IF NOT EXISTS idx_saga_states_status ON saga.saga_states (status);
CREATE INDEX IF NOT EXISTS idx_saga_states_correlation_id ON saga.saga_states (correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_saga_states_created_at ON saga.saga_states (created_at);

-- updated_at トリガー
CREATE TRIGGER update_saga_states_updated_at
    BEFORE UPDATE ON saga.saga_states
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
```

### 002_create_saga_states.down.sql

```sql
DROP TRIGGER IF EXISTS update_saga_states_updated_at ON saga.saga_states;
DROP TABLE IF EXISTS saga.saga_states;
```

### 003_create_saga_step_logs.up.sql

```sql
-- saga-db: saga_step_logs テーブル作成

CREATE TABLE IF NOT EXISTS saga.saga_step_logs (
    id                UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_id           UUID         NOT NULL REFERENCES saga.saga_states(id) ON DELETE CASCADE,
    step_index        INT          NOT NULL,
    step_name         VARCHAR(255) NOT NULL,
    action            VARCHAR(50)  NOT NULL,
    status            VARCHAR(50)  NOT NULL,
    request_payload   JSONB,
    response_payload  JSONB,
    error_message     TEXT,
    started_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at      TIMESTAMPTZ,

    CONSTRAINT chk_saga_step_logs_action CHECK (action IN ('EXECUTE', 'COMPENSATE')),
    CONSTRAINT chk_saga_step_logs_status CHECK (status IN ('SUCCESS', 'FAILED', 'TIMEOUT', 'SKIPPED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_saga_id_step_index ON saga.saga_step_logs (saga_id, step_index);
```

### 003_create_saga_step_logs.down.sql

```sql
DROP TABLE IF EXISTS saga.saga_step_logs;
```

### 004_add_indexes.up.sql

```sql
-- saga-db: saga_states および saga_step_logs への追加インデックス

-- saga_step_logs: ステップ名での検索用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_step_name
    ON saga.saga_step_logs (step_name);

-- saga_step_logs: ステータスでのフィルタ用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_status
    ON saga.saga_step_logs (status);

-- saga_step_logs: アクションでのフィルタ用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_action
    ON saga.saga_step_logs (action);

-- saga_step_logs: 開始時刻での範囲検索用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_started_at
    ON saga.saga_step_logs (started_at);
```

### 004_add_indexes.down.sql

```sql
DROP INDEX IF EXISTS saga.idx_saga_step_logs_step_name;
DROP INDEX IF EXISTS saga.idx_saga_step_logs_status;
DROP INDEX IF EXISTS saga.idx_saga_step_logs_action;
DROP INDEX IF EXISTS saga.idx_saga_step_logs_started_at;
```

### 005_add_updated_at_trigger.up.sql

```sql
-- saga-db: saga_step_logs の updated_at 関連拡張
-- 注意: saga_states のトリガーは 002_create_saga_states.up.sql で作成済み
--       saga.update_updated_at() 関数は 001_create_schema.up.sql で作成済み

-- saga_step_logs に updated_at カラムを追加
ALTER TABLE saga.saga_step_logs
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- saga_step_logs の updated_at トリガー
CREATE TRIGGER trigger_saga_step_logs_update_updated_at
    BEFORE UPDATE ON saga.saga_step_logs
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
```

### 005_add_updated_at_trigger.down.sql

```sql
DROP TRIGGER IF EXISTS trigger_saga_step_logs_update_updated_at ON saga.saga_step_logs;
ALTER TABLE saga.saga_step_logs DROP COLUMN IF EXISTS updated_at;
```

---

## 主要クエリパターン

### Saga 管理

```sql
-- 新規 Saga の作成
INSERT INTO saga.saga_states (
    id, workflow_name, current_step, status, payload, correlation_id, initiated_by
) VALUES ($1, $2, 0, 'STARTED', $3, $4, $5)
RETURNING *;

-- Saga の状態をステップログと共に更新（トランザクション内）
UPDATE saga.saga_states
SET current_step = $2, status = $3, error_message = $4, updated_at = NOW()
WHERE id = $1;

-- Saga ID による詳細取得
SELECT * FROM saga.saga_states WHERE id = $1;

-- ステップログ一覧取得（ステップ順）
SELECT * FROM saga.saga_step_logs
WHERE saga_id = $1
ORDER BY step_index ASC, started_at ASC;
```

### 起動時リカバリ

```sql
-- 未完了 Saga の検索（起動時リカバリ）
SELECT * FROM saga.saga_states
WHERE status IN ('STARTED', 'RUNNING', 'COMPENSATING')
ORDER BY created_at ASC;
```

### 一覧取得（ページネーション）

```sql
-- ワークフロー名・ステータス・相関 ID でフィルタリング
SELECT * FROM saga.saga_states
WHERE
    ($1::VARCHAR IS NULL OR workflow_name = $1)
    AND ($2::VARCHAR IS NULL OR status = $2)
    AND ($3::VARCHAR IS NULL OR correlation_id = $3)
ORDER BY created_at DESC
LIMIT $4 OFFSET $5;

-- 総件数
SELECT COUNT(*) FROM saga.saga_states
WHERE
    ($1::VARCHAR IS NULL OR workflow_name = $1)
    AND ($2::VARCHAR IS NULL OR status = $2)
    AND ($3::VARCHAR IS NULL OR correlation_id = $3);
```

---

## トランザクション設計の背景

`SagaPostgresRepository::update_with_step_log` の実装パターン:

```sql
BEGIN;
  UPDATE saga.saga_states
  SET current_step = $2,
      status = $3,
      error_message = $4,
      updated_at = NOW()
  WHERE id = $1;

  INSERT INTO saga.saga_step_logs (
    id, saga_id, step_index, step_name, action, status,
    request_payload, response_payload, error_message,
    started_at, completed_at
  ) VALUES ($5, $1, $6, $7, $8, $9, $10, $11, $12, $13, $14);
COMMIT;
```

この原子性により、以下を保証する:
- ステップログが記録されているなら、saga_states も一貫した状態にある
- サーバー障害時も中途半端な状態が残らない
- 起動時リカバリで確実に未完了 Saga を検出できる

---

## インデックス設計方針

- **リカバリクエリの最適化**: 起動時リカバリでは `status IN ('STARTED', 'RUNNING', 'COMPENSATING')` の条件で未完了 Saga を検索する。`idx_saga_states_status` インデックスによりフルスキャンを回避する
- **部分インデックス**: `correlation_id` は NULL が多いため部分インデックスを使用し、インデックスサイズを削減する
- **複合インデックス**: ステップログは `saga_id` + `step_index` の複合インデックスにより、特定 Saga のログを順序付きで効率的に取得できる

---

## 接続設定例

### config.yaml（saga サーバー用）

```yaml
# config/config.yaml — saga サーバー
app:
  name: "saga-server"
  version: "0.1.0"
  tier: "system"
  environment: "dev"

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""                   # Vault パス: secret/data/k1s0/system/saga/database キー: password
  ssl_mode: "disable"            # dev 環境。staging: require、prod: verify-full
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
```

---

## バックアップ実行例

```bash
# フルバックアップ（pg_basebackup）
pg_basebackup -h postgres.k1s0-system.svc.cluster.local -U replication -D /backup/base -Ft -z -P

# 論理バックアップ（スキーマ単位）
pg_dump -h postgres.k1s0-system.svc.cluster.local -U app -d k1s0_system \
    -n saga -Fc -f /backup/k1s0_system_saga.dump
```
