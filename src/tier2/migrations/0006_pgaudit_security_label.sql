-- k1s0 tier2 migration: 0006_pgaudit_security_label
-- pgaudit 拡張を有効化し、pii_segregated クラスのテーブルに SECURITY LABEL を適用する
-- table_classes.yaml: pii_segregated.pgaudit_required = true の要件を物理化する
-- pgaudit_config.yaml の log_catalog (ddl / read / write / role) を DB 層で強制する
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/
--
-- 前提条件:
--   shared_preload_libraries に 'pgaudit' が設定されていること
--   (CloudNativePG cluster.yaml の additionalConfig に設定する)

-- ============================================================
-- pgaudit 拡張の有効化
-- ============================================================
-- pgaudit 拡張が未インストールの場合に CREATE EXTENSION を実行する
-- shared_preload_libraries に pgaudit が含まれていない場合はエラーになる
CREATE EXTENSION IF NOT EXISTS pgaudit;

-- ============================================================
-- pii_data テーブルへの SECURITY LABEL 適用
-- ============================================================
-- table_classes.yaml: pii_segregated.pgaudit_required = true
-- pii_data テーブルへの全 DML（SELECT / INSERT / UPDATE / DELETE）を pgaudit でキャプチャする
-- pgaudit_config.yaml: log_catalog = [ddl, read, write, role]
-- 'OBJECT' ラベルはテーブル単位の全操作記録を有効化するラベル値（pgaudit 仕様に準拠）
SECURITY LABEL FOR pgaudit ON TABLE k1s0.pii_data IS 'OBJECT';

-- ============================================================
-- audit_event テーブルへの SECURITY LABEL 適用
-- ============================================================
-- 監査ログ自体（audit_event）への書込・参照を pgaudit でキャプチャする
-- 監査ログの改ざん検知を目的とした二重監査（audit of audit）を実現する
-- pgaudit_config.yaml のコメント: 「pii_segregated テーブルに pgaudit.log_relation を設定する」に対応する
SECURITY LABEL FOR pgaudit ON TABLE k1s0.audit_event IS 'OBJECT';

-- ============================================================
-- tenant_master テーブルへの SECURITY LABEL 適用
-- ============================================================
-- table_classes.yaml: tenant_master.audit_required = true
-- テナントマスタへの変更操作を pgaudit でキャプチャする（WRITE カテゴリでキャプチャされる）
-- READ（SELECT）はキャプチャするが pgaudit.log_catalog の read 設定に依存する
SECURITY LABEL FOR pgaudit ON TABLE k1s0.tenant_master IS 'OBJECT';

-- ============================================================
-- pgaudit セッションレベル設定の確認コメント
-- ============================================================
-- 以下の GUC は CloudNativePG cluster.yaml の additionalConfig で設定すること:
--   pgaudit.log = 'ddl,read,write,role'   -- pgaudit_config.yaml: log_catalog 4 カテゴリ
--   pgaudit.log_parameter = 'off'          -- pgaudit_config.yaml: log_parameter = false
--   pgaudit.log_statement_once = 'on'      -- pgaudit_config.yaml: log_statement_once = true
-- 上記 GUC は migration SQL では設定できないため cluster 設定で管理すること
-- 検証クエリ: SELECT * FROM pg_extension WHERE extname = 'pgaudit';
-- 検証クエリ: SELECT seclabel FROM pg_seclabels WHERE objname = 'pii_data';
