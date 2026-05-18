-- k1s0 tier2 migration: 0004_create_tenant_master_and_pii_tables
-- tenant_master / pii_data / platform_config テーブルを作成する
-- table_classes.yaml の 4 クラス全てを物理化する（tenant_scoped / tenant_master / pii_segregated / platform_global）
-- 0001_initial_schema.sql で作成された domain_event / outbox_message / audit_event は触らない
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/

-- k1s0 スキーマを作成する（0001 で作成済みだが idempotent のため IF NOT EXISTS を付ける）
CREATE SCHEMA IF NOT EXISTS k1s0;

-- ============================================================
-- tenant_master テーブル: テナントマスタクラス（table_classes.yaml: tenant_master）
-- ============================================================
-- テナント固有のマスタデータ（通貨レート・製品カタログ等）を格納する
-- RLS FORCE + tenant_id 一致ポリシーを 0005 で適用する（本 migration では構造のみ）
CREATE TABLE IF NOT EXISTS k1s0.tenant_master (
    -- エントリの主キー（UUID v4）
    id UUID PRIMARY KEY,
    -- テナント識別子（RLS FORCE が tenant_id でアクセス制御するキー）
    tenant_id UUID NOT NULL,
    -- マスタロール名（admin / operator / viewer 等のマスタ区分）
    role TEXT NOT NULL,
    -- 書込日時（UTC タイムゾーン付き、INSERT 時に自動設定する）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 最終更新日時（UTC タイムゾーン付き、UPDATE 時にアプリ層が設定する）
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 楽観的ロックバージョン（同時更新競合の検出に使用する）
    version INT NOT NULL DEFAULT 0
);

-- tenant_master の tenant_id インデックスを作成する（テナントスコープ検索の高速化）
CREATE INDEX IF NOT EXISTS idx_tenant_master_tenant_id
    ON k1s0.tenant_master(tenant_id);

-- tenant_master の複合インデックスを作成する（tenant_id + role での絞り込み高速化）
CREATE INDEX IF NOT EXISTS idx_tenant_master_tenant_role
    ON k1s0.tenant_master(tenant_id, role);

-- ============================================================
-- pii_data テーブル: PII 隔離クラス（table_classes.yaml: pii_segregated）
-- ============================================================
-- 個人情報（PII）を他テーブルから物理的に隔離して格納する
-- 個人情報保護法・GDPR 対象データ。RLS FORCE + purpose check を 0005 で適用する
-- pgaudit SECURITY LABEL は 0006 で適用する
CREATE TABLE IF NOT EXISTS k1s0.pii_data (
    -- エントリの主キー（UUID v4）
    id UUID PRIMARY KEY,
    -- テナント識別子（RLS FORCE が tenant_id でアクセス制御するキー）
    tenant_id UUID NOT NULL,
    -- PII 種別（email / phone / address / name / tax_id のいずれか）
    -- CHECK 制約で列挙値以外の投入を防止する
    pii_type TEXT NOT NULL CHECK (
        pii_type IN ('email', 'phone', 'address', 'name', 'tax_id')
    ),
    -- redaction クラス（anonymize / pseudonymize / encrypt / delete のいずれか）
    -- データ削除・匿名化方針の分類に使用する
    redaction_class TEXT NOT NULL CHECK (
        redaction_class IN ('anonymize', 'pseudonymize', 'encrypt', 'delete')
    ),
    -- PII 本体（暗号化または平文の PII を jsonb 型で格納する）
    -- アプリ層で OpenBao Transit による暗号化を適用してから INSERT すること
    payload JSONB NOT NULL,
    -- 書込日時（UTC タイムゾーン付き、INSERT 時に自動設定する）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 最終更新日時（UTC タイムゾーン付き、UPDATE 時にアプリ層が設定する）
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 楽観的ロックバージョン（同時更新競合の検出に使用する）
    version INT NOT NULL DEFAULT 0
);

-- pii_data の tenant_id インデックスを作成する（テナントスコープ検索の高速化）
CREATE INDEX IF NOT EXISTS idx_pii_data_tenant_id
    ON k1s0.pii_data(tenant_id);

-- pii_data の pii_type インデックスを作成する（PII 種別による絞り込み高速化）
CREATE INDEX IF NOT EXISTS idx_pii_data_pii_type
    ON k1s0.pii_data(tenant_id, pii_type);

-- pii_data の redaction_class インデックスを作成する（削除・匿名化バッチ処理の高速化）
CREATE INDEX IF NOT EXISTS idx_pii_data_redaction_class
    ON k1s0.pii_data(redaction_class);

-- ============================================================
-- platform_config テーブル: プラットフォームグローバルクラス（table_classes.yaml: platform_global）
-- ============================================================
-- プラットフォーム全体で共有する設定を格納する（通貨 / 国 / 業界共通分類等）
-- tenant_id は不要（全テナントが参照可能）。書込は platform_admin role のみ許可する
-- RLS は適用しない（table_classes.yaml: rls_enabled: false）
CREATE TABLE IF NOT EXISTS k1s0.platform_config (
    -- エントリの主キー（UUID v4）
    id UUID PRIMARY KEY,
    -- 設定キー（プラットフォーム全体でユニークな識別子）
    config_key TEXT NOT NULL UNIQUE,
    -- 設定値（jsonb 型で任意の構造化データを格納する）
    config_value JSONB NOT NULL,
    -- 書込日時（UTC タイムゾーン付き、INSERT 時に自動設定する）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 最終更新日時（UTC タイムゾーン付き、UPDATE 時にアプリ層が設定する）
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 楽観的ロックバージョン（同時更新競合の検出に使用する）
    version INT NOT NULL DEFAULT 0
);

-- platform_config の config_key インデックスを作成する（キー名による高速検索）
-- UNIQUE 制約で既にインデックスは生成されるが明示的に名前を付与する
CREATE UNIQUE INDEX IF NOT EXISTS idx_platform_config_key
    ON k1s0.platform_config(config_key);

-- ============================================================
-- outbox_message の存在確認と条件付きコメント
-- ============================================================
-- outbox_message は 0001_initial_schema.sql で既に CREATE 済みであるため本 migration では作成しない
-- 万一 0001 が未適用の環境では 0001 を先に実行すること（migration の依存順序を守ること）
-- sqlx migrate run は migration を番号順に適用するため、0001 → 0002 → 0003 → 0004 の順が保証される
