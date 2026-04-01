ALTER TABLE notification.notification_logs
    DROP CONSTRAINT IF EXISTS notification_logs_channel_id_fkey;

ALTER TABLE notification.notification_logs
    DROP CONSTRAINT IF EXISTS notification_logs_template_id_fkey;

ALTER TABLE notification.notification_logs
    ALTER COLUMN id TYPE UUID
    USING regexp_replace(substr(id, 7), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid,
    ALTER COLUMN id SET DEFAULT gen_random_uuid(),
    ALTER COLUMN channel_id TYPE UUID
    USING regexp_replace(substr(channel_id, 4), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid,
    ALTER COLUMN template_id TYPE UUID
    USING (
        CASE
            WHEN template_id IS NULL THEN NULL
            ELSE regexp_replace(substr(template_id, 5), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid
        END
    );

ALTER TABLE notification.channels
    ALTER COLUMN id TYPE UUID
    USING regexp_replace(substr(id, 4), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid,
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

ALTER TABLE notification.templates
    ALTER COLUMN id TYPE UUID
    USING regexp_replace(substr(id, 5), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid,
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

ALTER TABLE notification.notification_logs
    ADD CONSTRAINT notification_logs_channel_id_fkey
    FOREIGN KEY (channel_id) REFERENCES notification.channels(id);

ALTER TABLE notification.notification_logs
    ADD CONSTRAINT notification_logs_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES notification.templates(id);
