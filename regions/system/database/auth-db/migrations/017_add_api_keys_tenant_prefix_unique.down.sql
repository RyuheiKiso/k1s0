-- HIGH-5 監査対応ロールバック: (tenant_id, prefix) 複合 UNIQUE 制約を削除する
ALTER TABLE auth.api_keys DROP CONSTRAINT IF EXISTS uk_api_keys_tenant_prefix;
