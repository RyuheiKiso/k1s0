-- infra/docker/init-db/18-event-monitor-schema.sql
-- event-monitor スキーマ作成（Rust 実装コードの実テーブル定義に完全準拠）
-- 権威ソース: regions/system/server/rust/event-monitor/src/adapter/repository/
-- event_monitor スキーマは k1s0_system データベース内に作成する

\c k1s0_system;

-- スキーマ・拡張機能の作成
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS event_monitor;

-- updated_at 自動更新トリガー関数
CREATE OR REPLACE FUNCTION event_monitor.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ================================================================
-- flow_definitions テーブル（業務フロー定義）
-- 権威: flow_definition_postgres.rs / flow_definition.rs
-- ================================================================
CREATE TABLE IF NOT EXISTS event_monitor.flow_definitions (
    id                          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name                        VARCHAR(255)    UNIQUE NOT NULL,
    description                 TEXT            NOT NULL DEFAULT '',
    domain                      VARCHAR(255)    NOT NULL,
    -- フロー手順列（FlowStep 配列を JSON で格納: event_type/source/timeout_seconds/description）
    steps                       JSONB           NOT NULL DEFAULT '[]',
    slo_target_completion_secs  INT             NOT NULL DEFAULT 0,
    slo_target_success_rate     DOUBLE PRECISION NOT NULL DEFAULT 0.99,
    slo_alert_on_violation      BOOLEAN         NOT NULL DEFAULT TRUE,
    enabled                     BOOLEAN         NOT NULL DEFAULT TRUE,
    created_at                  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_flow_definitions_name
    ON event_monitor.flow_definitions (name);
CREATE INDEX IF NOT EXISTS idx_flow_definitions_domain
    ON event_monitor.flow_definitions (domain);
CREATE INDEX IF NOT EXISTS idx_flow_definitions_enabled
    ON event_monitor.flow_definitions (enabled);

CREATE TRIGGER trigger_flow_definitions_update_updated_at
    BEFORE UPDATE ON event_monitor.flow_definitions
    FOR EACH ROW EXECUTE FUNCTION event_monitor.update_updated_at();

-- ================================================================
-- event_records テーブル（Kafka から集約した業務イベント）
-- 権威: event_record_postgres.rs / event_record.rs
-- ================================================================
CREATE TABLE IF NOT EXISTS event_monitor.event_records (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    correlation_id  VARCHAR(255)    NOT NULL,
    event_type      VARCHAR(255)    NOT NULL,
    source          VARCHAR(255)    NOT NULL,
    domain          VARCHAR(255)    NOT NULL,
    trace_id        VARCHAR(64)     NOT NULL DEFAULT '',
    timestamp       TIMESTAMPTZ     NOT NULL,
    -- フローマッチング済みイベントには flow_id / flow_step_index が設定される
    flow_id         UUID            REFERENCES event_monitor.flow_definitions(id) ON DELETE SET NULL,
    flow_step_index INT,
    status          VARCHAR(50)     NOT NULL DEFAULT 'normal',
    received_at     TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_records_correlation_id
    ON event_monitor.event_records (correlation_id);
CREATE INDEX IF NOT EXISTS idx_event_records_event_type
    ON event_monitor.event_records (event_type);
CREATE INDEX IF NOT EXISTS idx_event_records_source
    ON event_monitor.event_records (source);
CREATE INDEX IF NOT EXISTS idx_event_records_domain
    ON event_monitor.event_records (domain);
CREATE INDEX IF NOT EXISTS idx_event_records_timestamp
    ON event_monitor.event_records (timestamp);
CREATE INDEX IF NOT EXISTS idx_event_records_flow_id
    ON event_monitor.event_records (flow_id)
    WHERE flow_id IS NOT NULL;

-- ================================================================
-- flow_instances テーブル（業務フロー実行インスタンス）
-- 権威: flow_instance_postgres.rs / flow_instance.rs
-- ================================================================
CREATE TABLE IF NOT EXISTS event_monitor.flow_instances (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    -- flow_definition_id ではなく flow_id（実コードに準拠）
    flow_id             UUID            NOT NULL REFERENCES event_monitor.flow_definitions(id) ON DELETE CASCADE,
    correlation_id      VARCHAR(255)    UNIQUE NOT NULL,
    -- status 値: 'in_progress' | 'completed' | 'failed' | 'timeout'（FlowInstanceStatus::as_str() に準拠）
    status              VARCHAR(50)     NOT NULL DEFAULT 'in_progress',
    current_step_index  INT             NOT NULL DEFAULT 0,
    started_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    completed_at        TIMESTAMPTZ,
    duration_ms         BIGINT,

    CONSTRAINT chk_flow_instances_status
        CHECK (status IN ('in_progress', 'completed', 'failed', 'timeout'))
);

CREATE INDEX IF NOT EXISTS idx_flow_instances_flow_id
    ON event_monitor.flow_instances (flow_id);
CREATE INDEX IF NOT EXISTS idx_flow_instances_correlation_id
    ON event_monitor.flow_instances (correlation_id);
CREATE INDEX IF NOT EXISTS idx_flow_instances_status
    ON event_monitor.flow_instances (status);
CREATE INDEX IF NOT EXISTS idx_flow_instances_started_at
    ON event_monitor.flow_instances (started_at);

-- k1s0 アプリユーザーへのスキーマ・テーブル権限付与
GRANT USAGE ON SCHEMA event_monitor TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA event_monitor TO k1s0;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA event_monitor TO k1s0;
-- 将来追加されるテーブルにも自動で権限を付与する
ALTER DEFAULT PRIVILEGES IN SCHEMA event_monitor
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;
