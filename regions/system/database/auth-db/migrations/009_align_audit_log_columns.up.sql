ALTER TABLE auth.audit_logs RENAME COLUMN detail TO metadata;
ALTER TABLE auth.audit_logs RENAME COLUMN created_at TO recorded_at;
