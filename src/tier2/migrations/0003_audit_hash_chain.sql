-- k1s0 tier2 migration: 0003_audit_hash_chain
-- migration_id: 0003_audit_hash_chain
-- description: audit hash chain 用カラムを audit_event テーブルに追加する（業務エラー監査 08 の hash chain 要件）
-- safety_level: safe
-- table_class: tenant_scoped
-- applied_at: 2026-05-17T00:00:00Z
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/

-- audit_event テーブルに hash_digest カラムを追加する
-- 各 audit_event エントリの SHA-256 ダイジェストを格納する
ALTER TABLE k1s0.audit_event
    ADD COLUMN IF NOT EXISTS hash_digest TEXT NULL;

-- audit_event テーブルに prev_digest カラムを追加する
-- 直前の audit_event エントリの hash_digest を参照するチェーン用カラム
-- 最初のエントリは NULL（チェーンの先頭を示す）
ALTER TABLE k1s0.audit_event
    ADD COLUMN IF NOT EXISTS prev_digest TEXT NULL;

-- audit_event テーブルに chain_sequence カラムを追加する
-- テナント内での audit_event の連番（hash chain の順序を保証する）
ALTER TABLE k1s0.audit_event
    ADD COLUMN IF NOT EXISTS chain_sequence BIGINT NULL;

-- hash_digest の非 null 制約は application 層が保証する（migration では null 許容のまま）
-- hash chain の整合性チェックは src/tier2/audit/hash_chain.rs が担う

-- テナント別 chain_sequence インデックスを作成する（hash chain の連続性検証に使用する）
CREATE INDEX IF NOT EXISTS idx_audit_event_tenant_chain_sequence
    ON k1s0.audit_event(tenant_id, chain_sequence);

-- hash_digest インデックスを作成する（prev_digest による前エントリの高速検索に使用する）
CREATE INDEX IF NOT EXISTS idx_audit_event_hash_digest
    ON k1s0.audit_event(hash_digest) WHERE hash_digest IS NOT NULL;
