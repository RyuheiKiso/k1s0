-- HIGH-5 監査対応: api_keys テーブルに (tenant_id, prefix) の複合 UNIQUE 制約を追加する
-- プレフィックスによるテナント横断列挙攻撃を防止するためテナント内での一意性を強制する
-- Migration 012 で service_name→tenant_id, key_prefix→prefix にリネーム済みのカラムを使用する
ALTER TABLE auth.api_keys
  ADD CONSTRAINT uk_api_keys_tenant_prefix UNIQUE (tenant_id, prefix);
