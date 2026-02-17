-- 初期データのロールバック
-- role_permissions -> roles / permissions の順に削除

-- ロール・権限マッピングの削除（初期ロールに紐づくもの）
DELETE FROM auth.role_permissions
WHERE role_id IN (
    SELECT id FROM auth.roles WHERE name IN ('sys_admin', 'sys_operator', 'sys_auditor')
);

-- デフォルト権限の削除
DELETE FROM auth.permissions
WHERE (resource, action) IN (
    ('users',        'read'),
    ('users',        'write'),
    ('users',        'delete'),
    ('users',        'admin'),
    ('auth_config',  'read'),
    ('auth_config',  'write'),
    ('auth_config',  'delete'),
    ('auth_config',  'admin'),
    ('audit_logs',   'read'),
    ('api_gateway',  'read'),
    ('api_gateway',  'write'),
    ('api_gateway',  'delete'),
    ('api_gateway',  'admin'),
    ('vault_secrets','read'),
    ('vault_secrets','write'),
    ('vault_secrets','delete'),
    ('vault_secrets','admin'),
    ('monitoring',   'read'),
    ('monitoring',   'write'),
    ('monitoring',   'delete'),
    ('monitoring',   'admin')
);

-- デフォルトロールの削除
DELETE FROM auth.roles WHERE name IN ('sys_admin', 'sys_operator', 'sys_auditor');
