import 'package:test/test.dart';
import 'package:k1s0_config/config.dart';

Config minimalConfig({
  DatabaseConfig? database,
  RedisConfig? redis,
  RedisConfig? redisSession,
  KafkaConfig? kafka,
  AuthConfig? auth,
}) {
  return Config(
    app: AppConfig(
        name: 'test', version: '1.0', tier: 'system', environment: 'dev'),
    server: ServerConfig(host: '0.0.0.0', port: 8080),
    database: database,
    redis: redis,
    redisSession: redisSession,
    kafka: kafka,
    observability: ObservabilityConfig(
      log: LogConfig(level: 'info', format: 'json'),
      trace: TraceConfig(enabled: false),
      metrics: MetricsConfig(enabled: false),
    ),
    auth: auth ??
        AuthConfig(jwt: JwtConfig(issuer: 'http://localhost', audience: 'test')),
  );
}

void main() {
  group('mergeVaultSecrets', () {
    test('should merge database.password', () {
      final cfg = minimalConfig(
        database: DatabaseConfig(
          host: 'localhost',
          port: 5432,
          name: 'test_db',
          user: 'app',
          password: 'old',
        ),
      );
      mergeVaultSecrets(cfg, {'database.password': 'vault-db-pass'});
      expect(cfg.database!.password, 'vault-db-pass');
    });

    test('should merge redis.password', () {
      final cfg = minimalConfig(
        redis: RedisConfig(host: 'localhost', port: 6379, password: 'old'),
      );
      mergeVaultSecrets(cfg, {'redis.password': 'vault-redis-pass'});
      expect(cfg.redis!.password, 'vault-redis-pass');
    });

    test('should merge kafka.sasl credentials', () {
      final cfg = minimalConfig(
        kafka: KafkaConfig(
          brokers: ['localhost:9092'],
          consumerGroup: 'test.default',
          securityProtocol: 'SASL_SSL',
          sasl: KafkaSaslConfig(
            mechanism: 'SCRAM-SHA-512',
            username: '',
            password: '',
          ),
          topics: KafkaTopics(publish: [], subscribe: []),
        ),
      );
      mergeVaultSecrets(cfg, {
        'kafka.sasl.username': 'vault-kafka-user',
        'kafka.sasl.password': 'vault-kafka-pass',
      });
      expect(cfg.kafka!.sasl!.username, 'vault-kafka-user');
      expect(cfg.kafka!.sasl!.password, 'vault-kafka-pass');
    });

    test('should merge redis_session.password', () {
      final cfg = minimalConfig(
        redisSession:
            RedisConfig(host: 'localhost', port: 6380, password: ''),
      );
      mergeVaultSecrets(
          cfg, {'redis_session.password': 'vault-session-pass'});
      expect(cfg.redisSession!.password, 'vault-session-pass');
    });

    test('should merge oidc.client_secret', () {
      final cfg = minimalConfig(
        auth: AuthConfig(
          jwt: JwtConfig(issuer: 'http://localhost', audience: 'test'),
          oidc: OidcConfig(
            discoveryUrl: 'http://localhost/.well-known',
            clientId: 'test',
            redirectUri: 'http://localhost/callback',
            scopes: ['openid'],
            jwksUri: 'http://localhost/jwks',
          ),
        ),
      );
      mergeVaultSecrets(
          cfg, {'auth.oidc.client_secret': 'vault-oidc-secret'});
      expect(cfg.auth.oidc!.clientSecret, 'vault-oidc-secret');
    });

    test('should not change config when secrets are empty', () {
      final cfg = minimalConfig(
        database: DatabaseConfig(
          host: 'localhost',
          port: 5432,
          name: 'test_db',
          user: 'app',
          password: 'original',
        ),
        redis:
            RedisConfig(host: 'localhost', port: 6379, password: 'original'),
      );
      mergeVaultSecrets(cfg, {});
      expect(cfg.database!.password, 'original');
      expect(cfg.redis!.password, 'original');
    });

    test('should handle nil sections safely', () {
      final cfg = minimalConfig();
      // All optional sections are null - should not throw
      mergeVaultSecrets(cfg, {
        'database.password': 'secret',
        'redis.password': 'secret',
        'kafka.sasl.username': 'user',
        'kafka.sasl.password': 'pass',
        'redis_session.password': 'secret',
        'auth.oidc.client_secret': 'secret',
      });
      expect(cfg.database, isNull);
      expect(cfg.redis, isNull);
      expect(cfg.kafka, isNull);
      expect(cfg.redisSession, isNull);
      expect(cfg.auth.oidc, isNull);
    });

    test('should merge only existing partial secrets', () {
      final cfg = minimalConfig(
        database: DatabaseConfig(
          host: 'localhost',
          port: 5432,
          name: 'test_db',
          user: 'app',
          password: 'old-db',
        ),
        redis:
            RedisConfig(host: 'localhost', port: 6379, password: 'old-redis'),
        auth: AuthConfig(
          jwt: JwtConfig(issuer: 'http://localhost', audience: 'test'),
          oidc: OidcConfig(
            discoveryUrl: 'http://localhost/.well-known',
            clientId: 'test',
            clientSecret: 'old-oidc',
            redirectUri: 'http://localhost/callback',
            scopes: ['openid'],
            jwksUri: 'http://localhost/jwks',
          ),
        ),
      );
      // Only database.password is provided
      mergeVaultSecrets(cfg, {'database.password': 'new-db'});
      expect(cfg.database!.password, 'new-db');
      expect(cfg.redis!.password, 'old-redis');
      expect(cfg.auth.oidc!.clientSecret, 'old-oidc');
    });
  });
}
