-- config-db: 初期データ投入（system Tier サービスの設定値）

-- system.auth.database — 認証サーバー DB 接続設定
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.auth.database', 'host',     '"localhost"',           'DB ホスト名',          'migration', 'migration'),
    ('system.auth.database', 'port',     '5432',                 'DB ポート番号',        'migration', 'migration'),
    ('system.auth.database', 'name',     '"k1s0_system"',        'DB 名',               'migration', 'migration'),
    ('system.auth.database', 'ssl_mode', '"disable"',            'SSL モード',           'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

-- system.auth.server — 認証サーバー設定
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.auth.server', 'port',          '8081',   'gRPC リッスンポート',     'migration', 'migration'),
    ('system.auth.server', 'read_timeout',  '30',     '読み取りタイムアウト（秒）', 'migration', 'migration'),
    ('system.auth.server', 'write_timeout', '30',     '書き込みタイムアウト（秒）', 'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

-- system.config.database — 設定サーバー DB 接続設定
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.config.database', 'host',     '"localhost"',           'DB ホスト名',          'migration', 'migration'),
    ('system.config.database', 'port',     '5432',                 'DB ポート番号',        'migration', 'migration'),
    ('system.config.database', 'name',     '"k1s0_system"',        'DB 名',               'migration', 'migration'),
    ('system.config.database', 'ssl_mode', '"disable"',            'SSL モード',           'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

-- system.config.server — 設定サーバー設定
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.config.server', 'port',          '8082',   'gRPC リッスンポート',     'migration', 'migration'),
    ('system.config.server', 'read_timeout',  '30',     '読み取りタイムアウト（秒）', 'migration', 'migration'),
    ('system.config.server', 'write_timeout', '30',     '書き込みタイムアウト（秒）', 'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;
