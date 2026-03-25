-- Project Master (taskmanagement business tier)
\c k1s0_business;

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE SCHEMA IF NOT EXISTS project_master;

-- プロジェクトタイプテーブル（ソフトウェア開発・マーケティング等のテンプレート定義を管理する）
CREATE TABLE IF NOT EXISTS project_master.project_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    default_workflow JSONB,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_project_types_code ON project_master.project_types(code);
CREATE INDEX IF NOT EXISTS idx_project_types_active ON project_master.project_types(is_active);

-- ステータス定義テーブル（Open/In Progress/Review/Done 等の共通ステータス定義を管理する）
CREATE TABLE IF NOT EXISTS project_master.status_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_type_id UUID NOT NULL REFERENCES project_master.project_types(id) ON DELETE CASCADE,
    code VARCHAR(100) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(7),
    allowed_transitions JSONB,
    is_initial BOOLEAN NOT NULL DEFAULT FALSE,
    is_terminal BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_status_definitions_type_code UNIQUE (project_type_id, code)
);

CREATE INDEX IF NOT EXISTS idx_status_definitions_project_type ON project_master.status_definitions(project_type_id);

-- ステータス定義バージョンテーブル（ワークフロー変更の監査履歴を管理する）
CREATE TABLE IF NOT EXISTS project_master.status_definition_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status_definition_id UUID NOT NULL REFERENCES project_master.status_definitions(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    before_data JSONB,
    after_data JSONB,
    changed_by VARCHAR(255) NOT NULL,
    change_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_status_definition_versions UNIQUE (status_definition_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_status_definition_versions_status
    ON project_master.status_definition_versions(status_definition_id, created_at DESC);

-- テナントプロジェクト拡張テーブル（テナント毎のカスタマイズを管理する）
CREATE TABLE IF NOT EXISTS project_master.tenant_project_extensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    status_definition_id UUID NOT NULL REFERENCES project_master.status_definitions(id) ON DELETE CASCADE,
    display_name_override VARCHAR(255),
    attributes_override JSONB,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_tenant_project_extensions UNIQUE (tenant_id, status_definition_id)
);

CREATE INDEX IF NOT EXISTS idx_tenant_project_extensions_tenant ON project_master.tenant_project_extensions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_project_extensions_status ON project_master.tenant_project_extensions(status_definition_id);

-- k1s0 アプリユーザーへのスキーマ使用権限とテーブル操作権限を付与する
GRANT USAGE ON SCHEMA project_master TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA project_master TO k1s0;
ALTER DEFAULT PRIVILEGES IN SCHEMA project_master GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;

-- sqlx マイグレーションが ALTER TABLE を実行できるようにテーブルオーナーを k1s0 に変更する
ALTER TABLE project_master.project_types OWNER TO k1s0;
ALTER TABLE project_master.status_definitions OWNER TO k1s0;
ALTER TABLE project_master.status_definition_versions OWNER TO k1s0;
ALTER TABLE project_master.tenant_project_extensions OWNER TO k1s0;

-- sqlx が k1s0_business 内に新規スキーマを作成できるように DATABASE レベルの CREATE 権限を付与する
GRANT CREATE ON DATABASE k1s0_business TO k1s0;
-- sqlx マイグレーションが project_master スキーマ内に _sqlx_migrations テーブルを作成できるように
-- スキーマレベルの CREATE 権限を付与する（GRANT CREATE ON DATABASE とは別物）（C-2 監査対応）
GRANT CREATE ON SCHEMA project_master TO k1s0;
