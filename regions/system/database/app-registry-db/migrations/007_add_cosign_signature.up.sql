-- STATIC-CRITICAL-002 監査対応: cosign_signature カラムを app_versions テーブルに追加する
-- コンテナイメージ・バイナリの Cosign 署名を保存し、サプライチェーン攻撃を防ぐ
ALTER TABLE app_registry.app_versions
    ADD COLUMN IF NOT EXISTS cosign_signature TEXT;

-- 署名が存在するバージョンの検索用インデックス
CREATE INDEX IF NOT EXISTS idx_app_versions_cosign_signature
    ON app_registry.app_versions (id)
    WHERE cosign_signature IS NOT NULL;
