-- 007_conditional_unique.up.sql のロールバック。
-- 条件付きユニーク制約を削除し、元の無条件 UNIQUE 制約に戻す。
-- 注意: ロールバック時、status = 'deleted' のテナントと同名のアクティブテナントが存在する場合、
-- ADD CONSTRAINT は失敗する。事前にデータ整合性を確認すること。

BEGIN;

-- 条件付きユニーク制約インデックスを削除する
DROP INDEX IF EXISTS tenant.uix_tenants_name_active;

-- 元の無条件 UNIQUE 制約を復元する
ALTER TABLE tenant.tenants ADD CONSTRAINT tenants_name_key UNIQUE (name);

COMMIT;
