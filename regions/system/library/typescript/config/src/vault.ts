import type { Config } from './config.js';

/**
 * Vault から取得したシークレットで設定値を上書きする。
 * 元の Config を変更せず、新しい Config を返す。
 */
export function mergeVaultSecrets(
  config: Config,
  secrets: Record<string, string>,
): Config {
  const merged = structuredClone(config);

  if (secrets['database.password'] && merged.database) {
    merged.database.password = secrets['database.password'];
  }
  if (secrets['redis.password'] && merged.redis) {
    merged.redis.password = secrets['redis.password'];
  }
  if (secrets['kafka.sasl.username'] && merged.kafka?.sasl) {
    merged.kafka.sasl.username = secrets['kafka.sasl.username'];
  }
  if (secrets['kafka.sasl.password'] && merged.kafka?.sasl) {
    merged.kafka.sasl.password = secrets['kafka.sasl.password'];
  }
  if (secrets['redis_session.password'] && merged.redis_session) {
    merged.redis_session.password = secrets['redis_session.password'];
  }
  if (secrets['auth.oidc.client_secret'] && merged.auth.oidc) {
    merged.auth.oidc.client_secret = secrets['auth.oidc.client_secret'];
  }

  return merged;
}
