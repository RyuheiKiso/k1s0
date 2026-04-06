-- project_types および status_definitions テーブルにテナント分離のための
-- tenant_id カラムと RLS ポリシーを追加する。
-- CRITICAL-BIZ-002 対応: グローバルマスターデータをテナント毎に分離する。
-- NULL 許容で追加し、既存データは 'system'（システム共通）テナントとして扱う。
-- テナント固有のカスタマイズは tenant_project_extensions テーブルで管理し続けるが、
-- プロジェクトタイプ自体もテナント毎に作成可能にする。
BEGIN;

-- project_types テーブルに tenant_id カラムを追加する
-- NULL 許容（既存データは 'system' テナントに属する）
ALTER TABLE project_master.project_types
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- status_definitions テーブルに tenant_id カラムを追加する
ALTER TABLE project_master.status_definitions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- project_types テーブルの RLS を有効化する
ALTER TABLE project_master.project_types ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_master.project_types FORCE ROW LEVEL SECURITY;

-- 既存ポリシーを削除して冪等性を保証する
DROP POLICY IF EXISTS tenant_isolation ON project_master.project_types;
-- テナント分離ポリシー: 自テナントの行または 'system' 共通行のみアクセス可能にする
CREATE POLICY tenant_isolation ON project_master.project_types
    USING (
        tenant_id = current_setting('app.current_tenant_id', true)::TEXT
        OR tenant_id = 'system'
    )
    WITH CHECK (
        tenant_id = current_setting('app.current_tenant_id', true)::TEXT
        OR tenant_id = 'system'
    );

-- status_definitions テーブルの RLS を有効化する
ALTER TABLE project_master.status_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_master.status_definitions FORCE ROW LEVEL SECURITY;

-- 既存ポリシーを削除して冪等性を保証する
DROP POLICY IF EXISTS tenant_isolation ON project_master.status_definitions;
-- テナント分離ポリシー: 自テナントの行または 'system' 共通行のみアクセス可能にする
CREATE POLICY tenant_isolation ON project_master.status_definitions
    USING (
        tenant_id = current_setting('app.current_tenant_id', true)::TEXT
        OR tenant_id = 'system'
    )
    WITH CHECK (
        tenant_id = current_setting('app.current_tenant_id', true)::TEXT
        OR tenant_id = 'system'
    );

-- tenant_id による検索を高速化するためのインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_project_types_tenant_id
    ON project_master.project_types (tenant_id);

CREATE INDEX IF NOT EXISTS idx_status_definitions_tenant_id
    ON project_master.status_definitions (tenant_id);

COMMIT;
