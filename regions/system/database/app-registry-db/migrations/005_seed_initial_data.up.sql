-- Development seed data
INSERT INTO app_registry.apps (id, name, description, category, icon_url) VALUES
    ('order-client', '受注管理クライアント', '受注業務を管理するデスクトップアプリケーション', 'business', '/icons/order-client.png'),
    ('inventory-client', '在庫管理クライアント', '在庫状況をリアルタイムで確認・管理するアプリケーション', 'business', '/icons/inventory-client.png'),
    ('admin-tool', '管理ツール', 'システム管理者向けの設定・監視ツール', 'system', '/icons/admin-tool.png')
ON CONFLICT (id) DO NOTHING;

INSERT INTO app_registry.app_versions (app_id, version, platform, arch, size_bytes, checksum_sha256, s3_key, release_notes, mandatory) VALUES
    ('order-client', '1.0.0', 'windows', 'x64', 52428800, 'abc123def456', 'order-client/1.0.0/windows-x64/order-client.exe', '初回リリース', false),
    ('order-client', '1.0.0', 'linux', 'x64', 48234496, 'def456abc123', 'order-client/1.0.0/linux-x64/order-client.AppImage', '初回リリース', false),
    ('order-client', '1.0.0', 'macos', 'arm64', 55574528, 'ghi789jkl012', 'order-client/1.0.0/macos-arm64/order-client.dmg', '初回リリース', false)
ON CONFLICT DO NOTHING;
