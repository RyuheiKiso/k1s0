import 'types.dart';

/// Vault から取得したシークレットで設定値を上書きする。
Config mergeVaultSecrets(Config config, Map<String, String> secrets) {
  if (secrets.containsKey('database.password') && config.database != null) {
    config.database!.password = secrets['database.password']!;
  }
  if (secrets.containsKey('redis.password') && config.redis != null) {
    config.redis!.password = secrets['redis.password'];
  }
  if (secrets.containsKey('kafka.sasl.username') &&
      config.kafka != null &&
      config.kafka!.sasl != null) {
    config.kafka!.sasl!.username = secrets['kafka.sasl.username']!;
  }
  if (secrets.containsKey('kafka.sasl.password') &&
      config.kafka != null &&
      config.kafka!.sasl != null) {
    config.kafka!.sasl!.password = secrets['kafka.sasl.password']!;
  }
  if (secrets.containsKey('redis_session.password') &&
      config.redisSession != null) {
    config.redisSession!.password = secrets['redis_session.password'];
  }
  if (secrets.containsKey('auth.oidc.client_secret') &&
      config.auth.oidc != null) {
    config.auth.oidc!.clientSecret = secrets['auth.oidc.client_secret'];
  }
  return config;
}
