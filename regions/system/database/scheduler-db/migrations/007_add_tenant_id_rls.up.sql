-- scheduler_jobs と job_executions に tenant_id カラムを追加し、RLS でテナント分離を実現する。
-- CRIT-005 監査対応: マルチテナント SaaS として他テナントのデータ参照を防ぐ。
-- 既存データは tenant_id = 'system' でバックフィルし、その後 DEFAULT を削除して NOT NULL を維持する。

BEGIN;

-- scheduler_jobs テーブルに tenant_id カラムを追加する（既存データのバックフィルとして 'system' をデフォルト値とする）
ALTER TABLE scheduler.scheduler_jobs
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE scheduler.scheduler_jobs
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_tenant_id
    ON scheduler.scheduler_jobs (tenant_id);

-- テナントとステータスの複合インデックスを追加する（テナント横断クエリの高速化）
CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_tenant_enabled
    ON scheduler.scheduler_jobs (tenant_id, enabled);

-- job_executions テーブルに tenant_id カラムを追加する（親テーブル scheduler_jobs と整合性を保つ）
ALTER TABLE scheduler.job_executions
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE scheduler.job_executions
    ALTER COLUMN tenant_id DROP DEFAULT;

-- job_executions の tenant_id インデックスを追加する
CREATE INDEX IF NOT EXISTS idx_job_executions_tenant_id
    ON scheduler.job_executions (tenant_id);

-- scheduler_jobs テーブルの RLS を有効化する
ALTER TABLE scheduler.scheduler_jobs ENABLE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（app.current_tenant_id セッション変数でフィルタリング）
-- current_setting の第 2 引数 true = 変数未設定時に NULL を返すことでエラーを回避する
DROP POLICY IF EXISTS tenant_isolation ON scheduler.scheduler_jobs;
CREATE POLICY tenant_isolation ON scheduler.scheduler_jobs
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- job_executions テーブルの RLS を有効化する
ALTER TABLE scheduler.job_executions ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON scheduler.job_executions;
CREATE POLICY tenant_isolation ON scheduler.job_executions
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- スーパーユーザー・オーナーロールも RLS の適用対象とする（バイパスを防止）
ALTER TABLE scheduler.scheduler_jobs FORCE ROW LEVEL SECURITY;
ALTER TABLE scheduler.job_executions FORCE ROW LEVEL SECURITY;

COMMIT;
