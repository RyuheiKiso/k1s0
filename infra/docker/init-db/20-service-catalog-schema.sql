-- infra/docker/init-db/20-service-catalog-schema.sql
-- service-catalog スキーマ作成（全マイグレーション 001〜007 を統合した最終状態）
-- 権威ソース: regions/system/database/service-catalog-db/migrations/
-- service_catalog スキーマは service_catalog_db データベース内に作成する

\c service_catalog_db;

-- スキーマの作成
CREATE SCHEMA IF NOT EXISTS service_catalog;

-- ================================================================
-- teams テーブル（サービス所有チーム）
-- 権威: migration 001_create_teams
-- ================================================================
CREATE TABLE IF NOT EXISTS service_catalog.teams (
    id            UUID         PRIMARY KEY,
    name          VARCHAR(255) NOT NULL UNIQUE,
    description   TEXT,
    contact_email VARCHAR(255),
    slack_channel VARCHAR(255),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ================================================================
-- services テーブル（サービスカタログエントリ）
-- 権威: migration 002_create_services
-- ================================================================
CREATE TABLE IF NOT EXISTS service_catalog.services (
    id              UUID        PRIMARY KEY,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    team_id         UUID        NOT NULL REFERENCES service_catalog.teams(id) ON DELETE CASCADE,
    tier            VARCHAR(50)  NOT NULL DEFAULT 'standard',
    lifecycle       VARCHAR(50)  NOT NULL DEFAULT 'development',
    repository_url  TEXT,
    api_endpoint    TEXT,
    healthcheck_url TEXT,
    tags            JSONB        NOT NULL DEFAULT '[]'::jsonb,
    metadata        JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ================================================================
-- dependencies テーブル（サービス間依存関係）
-- 権威: migration 003_create_dependencies
-- ================================================================
CREATE TABLE IF NOT EXISTS service_catalog.dependencies (
    source_service_id UUID        NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    target_service_id UUID        NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    dependency_type   VARCHAR(50) NOT NULL DEFAULT 'runtime',
    description       TEXT,
    PRIMARY KEY (source_service_id, target_service_id)
);

-- ================================================================
-- health_status テーブル（ヘルスチェック結果）
-- 権威: migration 004_create_health_status
-- ================================================================
CREATE TABLE IF NOT EXISTS service_catalog.health_status (
    service_id       UUID       PRIMARY KEY REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    status           VARCHAR(50) NOT NULL DEFAULT 'unknown',
    message          TEXT,
    response_time_ms BIGINT,
    checked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- service_docs テーブル（サービスドキュメントリンク）
-- 権威: migration 005_create_service_docs
-- ================================================================
CREATE TABLE IF NOT EXISTS service_catalog.service_docs (
    id         UUID        PRIMARY KEY,
    service_id UUID        NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    title      VARCHAR(255) NOT NULL,
    url        TEXT         NOT NULL,
    doc_type   VARCHAR(50)  NOT NULL DEFAULT 'other',
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ================================================================
-- scorecards テーブル（サービス品質スコア）
-- 権威: migration 006_create_scorecards
-- ================================================================
CREATE TABLE IF NOT EXISTS service_catalog.scorecards (
    service_id             UUID             PRIMARY KEY REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    documentation_score    DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    test_coverage_score    DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    slo_compliance_score   DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    security_score         DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    overall_score          DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    evaluated_at           TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

-- ================================================================
-- インデックス（migration 007_create_indexes）
-- ================================================================

-- services テーブルのインデックス
CREATE INDEX IF NOT EXISTS idx_services_team_id     ON service_catalog.services(team_id);
CREATE INDEX IF NOT EXISTS idx_services_tier        ON service_catalog.services(tier);
CREATE INDEX IF NOT EXISTS idx_services_lifecycle   ON service_catalog.services(lifecycle);
CREATE INDEX IF NOT EXISTS idx_services_name        ON service_catalog.services(name);
CREATE INDEX IF NOT EXISTS idx_services_tags        ON service_catalog.services USING GIN (tags);
-- 全文検索インデックス（name + description）
CREATE INDEX IF NOT EXISTS idx_services_name_desc_search ON service_catalog.services USING GIN (
    to_tsvector('english', coalesce(name, '') || ' ' || coalesce(description, ''))
);

-- dependencies テーブルのインデックス
CREATE INDEX IF NOT EXISTS idx_dependencies_source ON service_catalog.dependencies(source_service_id);
CREATE INDEX IF NOT EXISTS idx_dependencies_target ON service_catalog.dependencies(target_service_id);

-- service_docs テーブルのインデックス
CREATE INDEX IF NOT EXISTS idx_service_docs_service_id ON service_catalog.service_docs(service_id);

-- health_status テーブルのインデックス
CREATE INDEX IF NOT EXISTS idx_health_status_checked_at ON service_catalog.health_status(checked_at);

-- k1s0 アプリユーザーへのスキーマ・テーブル権限付与
GRANT USAGE ON SCHEMA service_catalog TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA service_catalog TO k1s0;
-- 将来追加されるテーブルにも自動で権限を付与する
ALTER DEFAULT PRIVILEGES IN SCHEMA service_catalog
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;
