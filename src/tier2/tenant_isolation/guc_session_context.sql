-- k1s0 tier2 session GUC 定義
-- PostgreSQL custom GUC（app.tenant_id / app.actor_id / app.purpose / app.delegation_chain）を
-- postgresql.conf または postgresql.auto.conf に追加する設定
-- CloudNativePG の additionalConfig セクションに記述する

-- ============================================================
-- GUC 定義（postgresql.conf に追記する形式）
-- ============================================================

-- app.tenant_id: テナント UUID を保持する GUC
-- SET LOCAL app.tenant_id = '<uuid>';
-- custom_variable_classes = 'app'  # postgresql.conf で要設定

-- app.actor_id: Keycloak subject（認証済み actor の識別子）
-- SET LOCAL app.actor_id = '<keycloak-sub>';

-- app.purpose: セッション目的（business_op / support / export / migration / emergency）
-- SET LOCAL app.purpose = 'business_op';

-- app.delegation_chain: 委譲元 user の配列（空配列 '{}' が通常）
-- SET LOCAL app.delegation_chain = '{}';

-- ============================================================
-- GUC の fail-safe デフォルト設定
-- GUC が SET されていない場合に RLS が 0 行を返す動作を確保する
-- ============================================================

-- GUC 未設定時の current_setting('app.tenant_id', true) は null を返す
-- tenant_id = null::uuid は常に false になるため RLS が 0 行を返す（fail-safe）

-- GUC 型検証クエリ（kind cluster 適用後に実行して確認する）
-- SELECT
--   current_setting('app.tenant_id', true) AS tenant_id,
--   current_setting('app.actor_id', true)  AS actor_id,
--   current_setting('app.purpose', true)   AS purpose,
--   current_setting('app.delegation_chain', true) AS delegation_chain;

-- ============================================================
-- CloudNativePG additionalConfig 形式（cnpg cluster.yaml に記述する）
-- ============================================================
-- additionalConfig:
--   custom_variable_classes: "app"

-- ============================================================
-- tier2 テスト用 GUC 設定スクリプト（kind cluster 確認用）
-- ============================================================

-- tenant_a のコンテキストを設定する
-- BEGIN;
-- SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000001';
-- SET LOCAL app.actor_id = 'keycloak-sub-tenant-a';
-- SET LOCAL app.purpose = 'business_op';
-- SET LOCAL app.delegation_chain = '{}';
-- SELECT * FROM k1s0.domain_event;  -- tenant_a のデータのみが返ることを確認する
-- ROLLBACK;

-- tenant_b のコンテキストを設定して tenant_a のデータが取得できないことを確認する
-- BEGIN;
-- SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000002';
-- SET LOCAL app.actor_id = 'keycloak-sub-tenant-b';
-- SET LOCAL app.purpose = 'business_op';
-- SET LOCAL app.delegation_chain = '{}';
-- SELECT COUNT(*) FROM k1s0.domain_event WHERE tenant_id = '00000000-0000-0000-0000-000000000001';
-- 期待値: 0（tenant_b から tenant_a のデータは見えない）
-- ROLLBACK;
