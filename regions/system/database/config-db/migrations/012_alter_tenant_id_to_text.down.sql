-- config-db の tenant_id を TEXT から UUID に戻す（ロールバック用）
-- 注意: TEXT 値が有効な UUID 形式でない場合は失敗する
-- 'system' 等の非 UUID 値が存在する場合はロールバック前にデータ修正が必要
SET LOCAL search_path TO config, public;

-- config_entries テーブルの tenant_id を UUID 型に戻す
-- USING 句で TEXT を UUID に変換する（有効な UUID 形式のみ変換可能）
ALTER TABLE config.config_entries
    ALTER COLUMN tenant_id TYPE UUID USING tenant_id::UUID;
ALTER TABLE config.config_entries
    ALTER COLUMN tenant_id SET DEFAULT '00000000-0000-0000-0000-000000000001';

-- config_change_logs テーブルの tenant_id を UUID 型に戻す
-- config_entries と同一の型・デフォルト値に揃える
ALTER TABLE config.config_change_logs
    ALTER COLUMN tenant_id TYPE UUID USING tenant_id::UUID;
ALTER TABLE config.config_change_logs
    ALTER COLUMN tenant_id SET DEFAULT '00000000-0000-0000-0000-000000000001';
