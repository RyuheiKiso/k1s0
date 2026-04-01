-- MED-003 監査対応: graphql-gateway 用のデフォルトレート制限ルールをシードする
-- このマイグレーションは 002_create_rate_limit_rules.up.sql + 003_add_scope_and_identifier_pattern.up.sql
-- の後に実行されるため、rate_limit_rules テーブルが確実に存在する。
-- ON CONFLICT (name) DO NOTHING により冪等に実行可能（再適用・ロールバック後の再実行でも安全）。

INSERT INTO ratelimit.rate_limit_rules (
    id,
    name,
    scope,
    identifier_pattern,
    limit_count,
    window_secs,
    algorithm,
    enabled
)
VALUES (
    gen_random_uuid(),
    'graphql-gateway-default',
    'graphql-gateway',
    '*',
    100,
    60,
    'sliding_window',
    true
)
ON CONFLICT (name) DO NOTHING;
