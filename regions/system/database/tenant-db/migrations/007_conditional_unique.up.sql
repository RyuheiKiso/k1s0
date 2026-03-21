-- ソフトデリート後の同名テナント再作成を可能にするため、
-- 無条件ユニーク制約を削除後、論理削除済みを除くユニーク制約（条件付きインデックス）に変更する。
-- 設計根拠: tenant.tenants の status = 'deleted' は論理削除済みを表す。
-- 同名テナントの再プロビジョニングシナリオで UNIQUE 違反が発生するため、条件付き制約に移行する。

BEGIN;

-- 既存の無条件 UNIQUE 制約を削除する（PostgreSQL 自動生成の制約名: tenants_name_key）
-- 002_create_tenants.up.sql で定義された "name VARCHAR(255) NOT NULL UNIQUE" に対応する
ALTER TABLE tenant.tenants DROP CONSTRAINT IF EXISTS tenants_name_key;

-- 論理削除済み（status = 'deleted'）を除いたアクティブテナントのみに適用する条件付きユニーク制約を追加する
-- これにより、削除済みテナントと同名の新規テナントを再作成できる
CREATE UNIQUE INDEX IF NOT EXISTS uix_tenants_name_active
    ON tenant.tenants (name)
    WHERE status != 'deleted';

COMMIT;
