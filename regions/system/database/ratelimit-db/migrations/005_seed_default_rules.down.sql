-- MED-003 監査対応: 005_seed_default_rules.up.sql のロールバック
-- graphql-gateway のデフォルトレート制限ルールを削除する

DELETE FROM ratelimit.rate_limit_rules
WHERE name = 'graphql-gateway-default';
