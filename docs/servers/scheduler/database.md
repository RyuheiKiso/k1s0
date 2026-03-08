# system-scheduler-server データベース設計

## スキーマ

スキーマ名: `scheduler`

```sql
CREATE SCHEMA IF NOT EXISTS scheduler;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| scheduler_jobs | スケジューラジョブ定義 |
| job_executions | ジョブ実行履歴 |

---

## ER 図

```
scheduler_jobs 1──* job_executions
```

---

## テーブル定義

### scheduler_jobs（ジョブ定義）

cron 式で定期実行するジョブを定義する。ターゲット種別（Kafka 等）とタイムゾーンを持つ。

```sql
CREATE TABLE IF NOT EXISTS scheduler.scheduler_jobs (
    id              VARCHAR(64)  PRIMARY KEY,
    name            VARCHAR(255) NOT NULL UNIQUE,
    description     TEXT,
    cron_expression VARCHAR(255) NOT NULL,
    job_type        VARCHAR(50)  NOT NULL DEFAULT 'default',
    payload         JSONB        NOT NULL DEFAULT '{}',
    enabled         BOOLEAN      NOT NULL DEFAULT true,
    max_retries     INT          NOT NULL DEFAULT 3,
    timezone        VARCHAR(100) NOT NULL DEFAULT 'UTC',
    target_type     VARCHAR(50)  NOT NULL DEFAULT 'kafka',
    target          TEXT,
    last_run_at     TIMESTAMPTZ,
    next_run_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_name ON scheduler.scheduler_jobs (name);
CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_enabled ON scheduler.scheduler_jobs (enabled);
CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_next_run_at ON scheduler.scheduler_jobs (next_run_at);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | VARCHAR(64) | PK | 主キー（プレフィックス付き ID: `job_`） |
| name | VARCHAR(255) | UNIQUE, NOT NULL | ジョブ名 |
| description | TEXT | | 説明 |
| cron_expression | VARCHAR(255) | NOT NULL | cron 式 |
| job_type | VARCHAR(50) | NOT NULL, DEFAULT 'default' | ジョブ種別 |
| payload | JSONB | NOT NULL, DEFAULT '{}' | ペイロード |
| enabled | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| max_retries | INT | NOT NULL, DEFAULT 3 | 最大リトライ回数 |
| timezone | VARCHAR(100) | NOT NULL, DEFAULT 'UTC' | タイムゾーン |
| target_type | VARCHAR(50) | NOT NULL, DEFAULT 'kafka' | ターゲット種別 |
| target | TEXT | | ターゲット |
| last_run_at | TIMESTAMPTZ | | 最終実行日時 |
| next_run_at | TIMESTAMPTZ | | 次回実行予定日時 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### job_executions（ジョブ実行履歴）

ジョブの実行結果を記録する。トリガー元（scheduler/manual）も管理する。

```sql
CREATE TABLE IF NOT EXISTS scheduler.job_executions (
    id            VARCHAR(64)  PRIMARY KEY,
    job_id        VARCHAR(64)  NOT NULL REFERENCES scheduler.scheduler_jobs(id) ON DELETE CASCADE,
    status        VARCHAR(50)  NOT NULL DEFAULT 'running',
    triggered_by  VARCHAR(50)  NOT NULL DEFAULT 'scheduler',
    started_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at  TIMESTAMPTZ,
    error_message TEXT,

    CONSTRAINT chk_job_executions_status CHECK (status IN ('running', 'succeeded', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_job_executions_job_id ON scheduler.job_executions (job_id);
CREATE INDEX IF NOT EXISTS idx_job_executions_status ON scheduler.job_executions (status);
CREATE INDEX IF NOT EXISTS idx_job_executions_started_at ON scheduler.job_executions (started_at DESC);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | VARCHAR(64) | PK | 主キー（プレフィックス付き ID: `exec_`） |
| job_id | VARCHAR(64) | FK → scheduler_jobs.id, NOT NULL | ジョブ ID |
| status | VARCHAR(50) | NOT NULL, DEFAULT 'running' | ステータス（running/succeeded/failed） |
| triggered_by | VARCHAR(50) | NOT NULL, DEFAULT 'scheduler' | トリガー元 |
| started_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 開始日時 |
| completed_at | TIMESTAMPTZ | | 完了日時 |
| error_message | TEXT | | エラーメッセージ |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/scheduler-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `scheduler` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_scheduler_jobs.up.sql` | scheduler_jobs テーブル作成 |
| `002_create_scheduler_jobs.down.sql` | テーブル削除 |
| `003_create_job_executions.up.sql` | job_executions テーブル作成 |
| `003_create_job_executions.down.sql` | テーブル削除 |
| `004_add_job_fields.up.sql` | description・timezone・target_type・target カラム追加 |
| `004_add_job_fields.down.sql` | カラム削除 |
| `005_align_job_executions_status_and_triggered_by.up.sql` | triggered_by カラム追加・status 値変更 |
| `005_align_job_executions_status_and_triggered_by.down.sql` | カラム・値変更復元 |
| `006_convert_prefixed_ids.up.sql` | UUID → プレフィックス付き VARCHAR(64) ID に変換 |
| `006_convert_prefixed_ids.down.sql` | ID 変換復元 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION scheduler.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_scheduler_jobs_update_updated_at
    BEFORE UPDATE ON scheduler.scheduler_jobs
    FOR EACH ROW EXECUTE FUNCTION scheduler.update_updated_at();
```
