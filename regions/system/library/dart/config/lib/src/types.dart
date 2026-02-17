/// Config クラス定義。config設計.md のスキーマに準拠する。
library;

class Config {
  final AppConfig app;
  final ServerConfig server;
  final GrpcConfig? grpc;
  final DatabaseConfig? database;
  final KafkaConfig? kafka;
  final RedisConfig? redis;
  final RedisConfig? redisSession;
  final ObservabilityConfig observability;
  final AuthConfig auth;

  Config({
    required this.app,
    required this.server,
    this.grpc,
    this.database,
    this.kafka,
    this.redis,
    this.redisSession,
    required this.observability,
    required this.auth,
  });

  factory Config.fromYaml(Map<String, dynamic> yaml) {
    return Config(
      app: AppConfig.fromYaml(yaml['app'] as Map<String, dynamic>),
      server: ServerConfig.fromYaml(yaml['server'] as Map<String, dynamic>),
      grpc: yaml['grpc'] != null
          ? GrpcConfig.fromYaml(yaml['grpc'] as Map<String, dynamic>)
          : null,
      database: yaml['database'] != null
          ? DatabaseConfig.fromYaml(yaml['database'] as Map<String, dynamic>)
          : null,
      kafka: yaml['kafka'] != null
          ? KafkaConfig.fromYaml(yaml['kafka'] as Map<String, dynamic>)
          : null,
      redis: yaml['redis'] != null
          ? RedisConfig.fromYaml(yaml['redis'] as Map<String, dynamic>)
          : null,
      redisSession: yaml['redis_session'] != null
          ? RedisConfig.fromYaml(
              yaml['redis_session'] as Map<String, dynamic>)
          : null,
      observability: ObservabilityConfig.fromYaml(
          yaml['observability'] as Map<String, dynamic>),
      auth: AuthConfig.fromYaml(yaml['auth'] as Map<String, dynamic>),
    );
  }
}

class AppConfig {
  final String name;
  final String version;
  final String tier;
  final String environment;

  AppConfig({
    required this.name,
    required this.version,
    required this.tier,
    required this.environment,
  });

  factory AppConfig.fromYaml(Map<String, dynamic> yaml) {
    return AppConfig(
      name: yaml['name'] as String,
      version: yaml['version'].toString(),
      tier: yaml['tier'] as String,
      environment: yaml['environment'] as String,
    );
  }
}

class ServerConfig {
  final String host;
  final int port;
  final String? readTimeout;
  final String? writeTimeout;
  final String? shutdownTimeout;

  ServerConfig({
    required this.host,
    required this.port,
    this.readTimeout,
    this.writeTimeout,
    this.shutdownTimeout,
  });

  factory ServerConfig.fromYaml(Map<String, dynamic> yaml) {
    return ServerConfig(
      host: yaml['host'] as String,
      port: yaml['port'] as int,
      readTimeout: yaml['read_timeout'] as String?,
      writeTimeout: yaml['write_timeout'] as String?,
      shutdownTimeout: yaml['shutdown_timeout'] as String?,
    );
  }
}

class GrpcConfig {
  final int port;
  final int? maxRecvMsgSize;

  GrpcConfig({required this.port, this.maxRecvMsgSize});

  factory GrpcConfig.fromYaml(Map<String, dynamic> yaml) {
    return GrpcConfig(
      port: yaml['port'] as int,
      maxRecvMsgSize: yaml['max_recv_msg_size'] as int?,
    );
  }
}

class DatabaseConfig {
  final String host;
  final int port;
  final String name;
  final String user;
  String password;
  final String? sslMode;
  final int? maxOpenConns;
  final int? maxIdleConns;
  final String? connMaxLifetime;

  DatabaseConfig({
    required this.host,
    required this.port,
    required this.name,
    required this.user,
    required this.password,
    this.sslMode,
    this.maxOpenConns,
    this.maxIdleConns,
    this.connMaxLifetime,
  });

  factory DatabaseConfig.fromYaml(Map<String, dynamic> yaml) {
    return DatabaseConfig(
      host: yaml['host'] as String,
      port: yaml['port'] as int,
      name: yaml['name'] as String,
      user: yaml['user'] as String,
      password: yaml['password'] as String? ?? '',
      sslMode: yaml['ssl_mode'] as String?,
      maxOpenConns: yaml['max_open_conns'] as int?,
      maxIdleConns: yaml['max_idle_conns'] as int?,
      connMaxLifetime: yaml['conn_max_lifetime'] as String?,
    );
  }
}

class KafkaConfig {
  final List<String> brokers;
  final String consumerGroup;
  final String securityProtocol;
  final KafkaSaslConfig? sasl;
  final KafkaTlsConfig? tls;
  final KafkaTopics? topics;

  KafkaConfig({
    required this.brokers,
    required this.consumerGroup,
    required this.securityProtocol,
    this.sasl,
    this.tls,
    this.topics,
  });

  factory KafkaConfig.fromYaml(Map<String, dynamic> yaml) {
    return KafkaConfig(
      brokers: (yaml['brokers'] as List).cast<String>(),
      consumerGroup: yaml['consumer_group'] as String,
      securityProtocol: yaml['security_protocol'] as String,
      sasl: yaml['sasl'] != null
          ? KafkaSaslConfig.fromYaml(yaml['sasl'] as Map<String, dynamic>)
          : null,
      tls: yaml['tls'] != null
          ? KafkaTlsConfig.fromYaml(yaml['tls'] as Map<String, dynamic>)
          : null,
      topics: yaml['topics'] != null
          ? KafkaTopics.fromYaml(yaml['topics'] as Map<String, dynamic>)
          : null,
    );
  }
}

class KafkaSaslConfig {
  final String mechanism;
  String username;
  String password;

  KafkaSaslConfig({
    required this.mechanism,
    required this.username,
    required this.password,
  });

  factory KafkaSaslConfig.fromYaml(Map<String, dynamic> yaml) {
    return KafkaSaslConfig(
      mechanism: yaml['mechanism'] as String,
      username: yaml['username'] as String? ?? '',
      password: yaml['password'] as String? ?? '',
    );
  }
}

class KafkaTlsConfig {
  final String? caCertPath;

  KafkaTlsConfig({this.caCertPath});

  factory KafkaTlsConfig.fromYaml(Map<String, dynamic> yaml) {
    return KafkaTlsConfig(
      caCertPath: yaml['ca_cert_path'] as String?,
    );
  }
}

class KafkaTopics {
  final List<String> publish;
  final List<String> subscribe;

  KafkaTopics({required this.publish, required this.subscribe});

  factory KafkaTopics.fromYaml(Map<String, dynamic> yaml) {
    return KafkaTopics(
      publish: (yaml['publish'] as List?)?.cast<String>() ?? [],
      subscribe: (yaml['subscribe'] as List?)?.cast<String>() ?? [],
    );
  }
}

class RedisConfig {
  final String host;
  final int port;
  String? password;
  final int? db;
  final int? poolSize;

  RedisConfig({
    required this.host,
    required this.port,
    this.password,
    this.db,
    this.poolSize,
  });

  factory RedisConfig.fromYaml(Map<String, dynamic> yaml) {
    return RedisConfig(
      host: yaml['host'] as String,
      port: yaml['port'] as int,
      password: yaml['password'] as String?,
      db: yaml['db'] as int?,
      poolSize: yaml['pool_size'] as int?,
    );
  }
}

class ObservabilityConfig {
  final LogConfig log;
  final TraceConfig trace;
  final MetricsConfig metrics;

  ObservabilityConfig({
    required this.log,
    required this.trace,
    required this.metrics,
  });

  factory ObservabilityConfig.fromYaml(Map<String, dynamic> yaml) {
    return ObservabilityConfig(
      log: LogConfig.fromYaml(yaml['log'] as Map<String, dynamic>),
      trace: TraceConfig.fromYaml(yaml['trace'] as Map<String, dynamic>),
      metrics: MetricsConfig.fromYaml(yaml['metrics'] as Map<String, dynamic>),
    );
  }
}

class LogConfig {
  final String level;
  final String format;

  LogConfig({required this.level, required this.format});

  factory LogConfig.fromYaml(Map<String, dynamic> yaml) {
    return LogConfig(
      level: yaml['level'] as String,
      format: yaml['format'] as String,
    );
  }
}

class TraceConfig {
  final bool enabled;
  final String? endpoint;
  final double? sampleRate;

  TraceConfig({required this.enabled, this.endpoint, this.sampleRate});

  factory TraceConfig.fromYaml(Map<String, dynamic> yaml) {
    return TraceConfig(
      enabled: yaml['enabled'] as bool,
      endpoint: yaml['endpoint'] as String?,
      sampleRate: (yaml['sample_rate'] as num?)?.toDouble(),
    );
  }
}

class MetricsConfig {
  final bool enabled;
  final String? path;

  MetricsConfig({required this.enabled, this.path});

  factory MetricsConfig.fromYaml(Map<String, dynamic> yaml) {
    return MetricsConfig(
      enabled: yaml['enabled'] as bool,
      path: yaml['path'] as String?,
    );
  }
}

class AuthConfig {
  final JwtConfig jwt;
  final OidcConfig? oidc;

  AuthConfig({required this.jwt, this.oidc});

  factory AuthConfig.fromYaml(Map<String, dynamic> yaml) {
    return AuthConfig(
      jwt: JwtConfig.fromYaml(yaml['jwt'] as Map<String, dynamic>),
      oidc: yaml['oidc'] != null
          ? OidcConfig.fromYaml(yaml['oidc'] as Map<String, dynamic>)
          : null,
    );
  }
}

class JwtConfig {
  final String issuer;
  final String audience;
  final String? publicKeyPath;

  JwtConfig({required this.issuer, required this.audience, this.publicKeyPath});

  factory JwtConfig.fromYaml(Map<String, dynamic> yaml) {
    return JwtConfig(
      issuer: yaml['issuer'] as String,
      audience: yaml['audience'] as String,
      publicKeyPath: yaml['public_key_path'] as String?,
    );
  }
}

class OidcConfig {
  final String discoveryUrl;
  final String clientId;
  String? clientSecret;
  final String redirectUri;
  final List<String> scopes;
  final String jwksUri;
  final String? jwksCacheTtl;

  OidcConfig({
    required this.discoveryUrl,
    required this.clientId,
    this.clientSecret,
    required this.redirectUri,
    required this.scopes,
    required this.jwksUri,
    this.jwksCacheTtl,
  });

  factory OidcConfig.fromYaml(Map<String, dynamic> yaml) {
    return OidcConfig(
      discoveryUrl: yaml['discovery_url'] as String,
      clientId: yaml['client_id'] as String,
      clientSecret: yaml['client_secret'] as String?,
      redirectUri: yaml['redirect_uri'] as String,
      scopes: (yaml['scopes'] as List).cast<String>(),
      jwksUri: yaml['jwks_uri'] as String,
      jwksCacheTtl: yaml['jwks_cache_ttl'] as String?,
    );
  }
}
