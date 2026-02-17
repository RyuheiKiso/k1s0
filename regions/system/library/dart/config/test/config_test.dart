import 'dart:io';
import 'package:test/test.dart';
import 'package:k1s0_config/config.dart';

const minimalConfigYaml = '''
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
''';

String writeConfig(Directory dir, String filename, String content) {
  final file = File('${dir.path}/$filename');
  file.writeAsStringSync(content);
  return file.path;
}

void main() {
  group('loadConfig', () {
    test('should load a valid config', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final path = writeConfig(dir, 'config.yaml', minimalConfigYaml);

      final cfg = loadConfig(path);
      expect(cfg.app.name, 'test-server');
      expect(cfg.server.port, 8080);
    });

    test('should throw on file not found', () {
      expect(
        () => loadConfig('/nonexistent/config.yaml'),
        throwsA(isA<FileSystemException>()),
      );
    });

    test('should merge env override', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final basePath = writeConfig(dir, 'config.yaml', minimalConfigYaml);
      final envPath = writeConfig(dir, 'config.staging.yaml', '''
app:
  environment: staging
server:
  port: 9090
observability:
  log:
    level: info
''');

      final cfg = loadConfig(basePath, envPath);
      expect(cfg.app.environment, 'staging');
      expect(cfg.server.port, 9090);
      expect(cfg.app.name, 'test-server'); // base value preserved
    });
  });

  group('validateConfig', () {
    test('should pass for valid config', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final path = writeConfig(dir, 'config.yaml', minimalConfigYaml);

      final cfg = loadConfig(path);
      expect(() => validateConfig(cfg), returnsNormally);
    });

    test('should reject empty app name', () {
      final cfg = Config(
        app: AppConfig(name: '', version: '1.0', tier: 'system', environment: 'dev'),
        server: ServerConfig(host: '0.0.0.0', port: 8080),
        observability: ObservabilityConfig(
          log: LogConfig(level: 'info', format: 'json'),
          trace: TraceConfig(enabled: false),
          metrics: MetricsConfig(enabled: false),
        ),
        auth: AuthConfig(jwt: JwtConfig(issuer: 'x', audience: 'x')),
      );
      expect(
        () => validateConfig(cfg),
        throwsA(isA<ConfigValidationError>()),
      );
    });

    test('should reject invalid tier', () {
      final cfg = Config(
        app: AppConfig(name: 'test', version: '1.0', tier: 'invalid', environment: 'dev'),
        server: ServerConfig(host: '0.0.0.0', port: 8080),
        observability: ObservabilityConfig(
          log: LogConfig(level: 'info', format: 'json'),
          trace: TraceConfig(enabled: false),
          metrics: MetricsConfig(enabled: false),
        ),
        auth: AuthConfig(jwt: JwtConfig(issuer: 'x', audience: 'x')),
      );
      expect(
        () => validateConfig(cfg),
        throwsA(isA<ConfigValidationError>()),
      );
    });

    test('should reject invalid environment', () {
      final cfg = Config(
        app: AppConfig(name: 'test', version: '1.0', tier: 'system', environment: 'invalid'),
        server: ServerConfig(host: '0.0.0.0', port: 8080),
        observability: ObservabilityConfig(
          log: LogConfig(level: 'info', format: 'json'),
          trace: TraceConfig(enabled: false),
          metrics: MetricsConfig(enabled: false),
        ),
        auth: AuthConfig(jwt: JwtConfig(issuer: 'x', audience: 'x')),
      );
      expect(
        () => validateConfig(cfg),
        throwsA(isA<ConfigValidationError>()),
      );
    });

    test('should reject invalid port', () {
      final cfg = Config(
        app: AppConfig(name: 'test', version: '1.0', tier: 'system', environment: 'dev'),
        server: ServerConfig(host: '0.0.0.0', port: 0),
        observability: ObservabilityConfig(
          log: LogConfig(level: 'info', format: 'json'),
          trace: TraceConfig(enabled: false),
          metrics: MetricsConfig(enabled: false),
        ),
        auth: AuthConfig(jwt: JwtConfig(issuer: 'x', audience: 'x')),
      );
      expect(
        () => validateConfig(cfg),
        throwsA(isA<ConfigValidationError>()),
      );
    });

    test('should reject empty jwt issuer', () {
      final cfg = Config(
        app: AppConfig(name: 'test', version: '1.0', tier: 'system', environment: 'dev'),
        server: ServerConfig(host: '0.0.0.0', port: 8080),
        observability: ObservabilityConfig(
          log: LogConfig(level: 'info', format: 'json'),
          trace: TraceConfig(enabled: false),
          metrics: MetricsConfig(enabled: false),
        ),
        auth: AuthConfig(jwt: JwtConfig(issuer: '', audience: 'x')),
      );
      expect(
        () => validateConfig(cfg),
        throwsA(isA<ConfigValidationError>()),
      );
    });
  });

  group('mergeVaultSecrets', () {
    test('should merge database password', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final path = writeConfig(dir, 'config.yaml', '''
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 5432
  name: test_db
  user: app
  password: ""
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
''');

      final cfg = loadConfig(path);
      mergeVaultSecrets(cfg, {'database.password': 'secret123'});
      expect(cfg.database!.password, 'secret123');
    });

    test('should merge redis password', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final path = writeConfig(dir, 'config.yaml', '''
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
redis:
  host: localhost
  port: 6379
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
''');

      final cfg = loadConfig(path);
      mergeVaultSecrets(cfg, {'redis.password': 'redis-secret'});
      expect(cfg.redis!.password, 'redis-secret');
    });

    test('should merge oidc client secret', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final path = writeConfig(dir, 'config.yaml', '''
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
  oidc:
    discovery_url: "http://localhost/.well-known"
    client_id: "test"
    redirect_uri: "http://localhost/callback"
    scopes: ["openid"]
    jwks_uri: "http://localhost/jwks"
''');

      final cfg = loadConfig(path);
      mergeVaultSecrets(cfg, {'auth.oidc.client_secret': 'oidc-secret'});
      expect(cfg.auth.oidc!.clientSecret, 'oidc-secret');
    });

    test('should merge kafka sasl credentials', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final path = writeConfig(dir, 'config.yaml', '''
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
kafka:
  brokers:
    - "localhost:9092"
  consumer_group: "test.default"
  security_protocol: "SASL_SSL"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: ""
    password: ""
  topics:
    publish: []
    subscribe: []
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
''');

      final cfg = loadConfig(path);
      mergeVaultSecrets(cfg, {
        'kafka.sasl.username': 'kafka-user',
        'kafka.sasl.password': 'kafka-pass',
      });
      expect(cfg.kafka!.sasl!.username, 'kafka-user');
      expect(cfg.kafka!.sasl!.password, 'kafka-pass');
    });

    test('should handle nil optional fields safely', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final path = writeConfig(dir, 'config.yaml', minimalConfigYaml);
      final cfg = loadConfig(path);
      // Should not throw when optional fields are null
      mergeVaultSecrets(cfg, {
        'database.password': 'secret',
        'redis.password': 'secret',
        'kafka.sasl.username': 'user',
        'kafka.sasl.password': 'pass',
        'auth.oidc.client_secret': 'secret',
      });
      expect(cfg.database, isNull);
      expect(cfg.redis, isNull);
      expect(cfg.kafka, isNull);
      expect(cfg.auth.oidc, isNull);
    });
  });

  group('full config', () {
    test('should load and validate full config', () {
      final dir = Directory.systemTemp.createTempSync('k1s0_');
      final path = writeConfig(dir, 'config.yaml', '''
app:
  name: order-server
  version: "1.0.0"
  tier: service
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"
grpc:
  port: 50051
  max_recv_msg_size: 4194304
database:
  host: "localhost"
  port: 5432
  name: "order_db"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
kafka:
  brokers:
    - "localhost:9092"
  consumer_group: "order-server.default"
  security_protocol: "PLAINTEXT"
  topics:
    publish:
      - "k1s0.service.order.created.v1"
    subscribe:
      - "k1s0.service.payment.completed.v1"
redis:
  host: "localhost"
  port: 6379
  db: 0
  pool_size: 10
observability:
  log:
    level: info
    format: json
  trace:
    enabled: true
    endpoint: "localhost:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
  oidc:
    discovery_url: "http://localhost:8180/realms/k1s0/.well-known/openid-configuration"
    client_id: "k1s0-bff"
    redirect_uri: "http://localhost:3000/callback"
    scopes:
      - "openid"
      - "profile"
    jwks_uri: "http://localhost:8180/realms/k1s0/protocol/openid-connect/certs"
    jwks_cache_ttl: "10m"
''');

      final cfg = loadConfig(path);
      expect(cfg.app.name, 'order-server');
      expect(cfg.app.tier, 'service');
      expect(cfg.grpc, isNotNull);
      expect(cfg.grpc!.port, 50051);
      expect(cfg.database, isNotNull);
      expect(cfg.database!.name, 'order_db');
      expect(cfg.kafka, isNotNull);
      expect(cfg.kafka!.securityProtocol, 'PLAINTEXT');
      expect(cfg.redis, isNotNull);
      expect(cfg.redis!.port, 6379);
      expect(cfg.auth.oidc, isNotNull);
      expect(cfg.auth.oidc!.clientId, 'k1s0-bff');

      expect(() => validateConfig(cfg), returnsNormally);
    });
  });
}
