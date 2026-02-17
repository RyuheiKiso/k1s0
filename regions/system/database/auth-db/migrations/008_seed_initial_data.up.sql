-- auth-db: 初期データ投入（デフォルトロール・権限・ロール権限マッピング）

-- デフォルトロール
-- 認証認可設計 D-005 の Tier 別ロール定義に対応する初期ロールを投入する
INSERT INTO auth.roles (name, description, tier) VALUES
    ('sys_admin',    'システム全体の管理者。すべてのリソースに対する全権限',         'system'),
    ('sys_operator', 'システム運用担当。監視・ログ閲覧・設定変更',                   'system'),
    ('sys_auditor',  '監査担当。全リソースの読み取り専用',                            'system')
ON CONFLICT (name) DO NOTHING;

-- デフォルト権限
-- 認証認可設計 D-005 のパーミッションマトリクスに対応する初期権限を投入する
INSERT INTO auth.permissions (resource, action, description) VALUES
    -- users リソース
    ('users',        'read',   'ユーザー情報の閲覧'),
    ('users',        'write',  'ユーザー情報の作成・更新'),
    ('users',        'delete', 'ユーザーの削除'),
    ('users',        'admin',  'ユーザー管理の全権限'),
    -- auth_config リソース
    ('auth_config',  'read',   '認証設定の閲覧'),
    ('auth_config',  'write',  '認証設定の作成・更新'),
    ('auth_config',  'delete', '認証設定の削除'),
    ('auth_config',  'admin',  '認証設定管理の全権限'),
    -- audit_logs リソース
    ('audit_logs',   'read',   '監査ログの閲覧'),
    -- api_gateway リソース
    ('api_gateway',  'read',   'API Gateway 設定の閲覧'),
    ('api_gateway',  'write',  'API Gateway 設定の作成・更新'),
    ('api_gateway',  'delete', 'API Gateway 設定の削除'),
    ('api_gateway',  'admin',  'API Gateway 管理の全権限'),
    -- vault_secrets リソース
    ('vault_secrets','read',   'Vault シークレットの閲覧'),
    ('vault_secrets','write',  'Vault シークレットの作成・更新'),
    ('vault_secrets','delete', 'Vault シークレットの削除'),
    ('vault_secrets','admin',  'Vault シークレット管理の全権限'),
    -- monitoring リソース
    ('monitoring',   'read',   '監視データの閲覧'),
    ('monitoring',   'write',  '監視設定の作成・更新'),
    ('monitoring',   'delete', '監視設定の削除'),
    ('monitoring',   'admin',  '監視管理の全権限')
ON CONFLICT (resource, action) DO NOTHING;

-- デフォルトロール・権限マッピング
-- 認証認可設計 D-005 の system Tier パーミッションマトリクスに対応する

-- sys_admin: すべてのリソースに対する全権限
INSERT INTO auth.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM auth.roles r
CROSS JOIN auth.permissions p
WHERE r.name = 'sys_admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- sys_operator: 監視・ログ閲覧・設定変更
INSERT INTO auth.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM auth.roles r
CROSS JOIN auth.permissions p
WHERE r.name = 'sys_operator'
  AND (
    (p.resource = 'users'        AND p.action = 'read')
    OR (p.resource = 'auth_config'  AND p.action IN ('read', 'write'))
    OR (p.resource = 'audit_logs'   AND p.action = 'read')
    OR (p.resource = 'api_gateway'  AND p.action = 'read')
    OR (p.resource = 'vault_secrets' AND p.action = 'read')
    OR (p.resource = 'monitoring'   AND p.action IN ('read', 'write'))
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- sys_auditor: 全リソースの読み取り専用（vault_secrets を除く）
INSERT INTO auth.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM auth.roles r
CROSS JOIN auth.permissions p
WHERE r.name = 'sys_auditor'
  AND p.action = 'read'
  AND p.resource != 'vault_secrets'
ON CONFLICT (role_id, permission_id) DO NOTHING;
