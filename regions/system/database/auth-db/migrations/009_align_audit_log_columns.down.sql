ALTER TABLE auth.audit_logs RENAME COLUMN metadata TO detail;
ALTER TABLE auth.audit_logs RENAME COLUMN recorded_at TO created_at;
