ALTER TABLE notification.notification_logs
    DROP CONSTRAINT IF EXISTS notification_logs_channel_id_fkey;

ALTER TABLE notification.notification_logs
    DROP CONSTRAINT IF EXISTS notification_logs_template_id_fkey;

ALTER TABLE notification.channels
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE VARCHAR(64)
    USING ('ch_' || replace(id::text, '-', ''));

ALTER TABLE notification.templates
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE VARCHAR(64)
    USING ('tpl_' || replace(id::text, '-', ''));

ALTER TABLE notification.notification_logs
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE VARCHAR(64)
    USING ('notif_' || replace(id::text, '-', '')),
    ALTER COLUMN channel_id TYPE VARCHAR(64)
    USING ('ch_' || replace(channel_id::text, '-', '')),
    ALTER COLUMN template_id TYPE VARCHAR(64)
    USING (
        CASE
            WHEN template_id IS NULL THEN NULL
            ELSE 'tpl_' || replace(template_id::text, '-', '')
        END
    );

ALTER TABLE notification.notification_logs
    ADD CONSTRAINT notification_logs_channel_id_fkey
    FOREIGN KEY (channel_id) REFERENCES notification.channels(id);

ALTER TABLE notification.notification_logs
    ADD CONSTRAINT notification_logs_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES notification.templates(id);
