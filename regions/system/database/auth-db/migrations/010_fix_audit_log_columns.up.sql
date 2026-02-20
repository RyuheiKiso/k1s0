-- auth-db: audit_logs カラム名を正規設計に合わせる
-- migration 009 でリネームされた列名を元の設計（system-database設計.md）に戻す

ALTER TABLE auth.audit_logs RENAME COLUMN metadata TO detail;
ALTER TABLE auth.audit_logs RENAME COLUMN recorded_at TO created_at;
