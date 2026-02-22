using YamlDotNet.Serialization;

namespace K1s0.System.Config;

public sealed record AppSection
{
    [YamlMember(Alias = "name")]
    public string Name { get; init; } = string.Empty;

    [YamlMember(Alias = "version")]
    public string Version { get; init; } = string.Empty;

    [YamlMember(Alias = "tier")]
    public string Tier { get; init; } = string.Empty;

    [YamlMember(Alias = "environment")]
    public string Environment { get; init; } = string.Empty;
}

public sealed record ServerSection
{
    [YamlMember(Alias = "host")]
    public string Host { get; init; } = string.Empty;

    [YamlMember(Alias = "port")]
    public int Port { get; init; }

    [YamlMember(Alias = "read_timeout")]
    public int? ReadTimeout { get; init; }

    [YamlMember(Alias = "write_timeout")]
    public int? WriteTimeout { get; init; }

    [YamlMember(Alias = "shutdown_timeout")]
    public int? ShutdownTimeout { get; init; }
}

public sealed record GrpcSection
{
    [YamlMember(Alias = "port")]
    public int Port { get; init; }

    [YamlMember(Alias = "max_recv_msg_size")]
    public int? MaxRecvMsgSize { get; init; }
}

public sealed record DatabaseSection
{
    [YamlMember(Alias = "host")]
    public string Host { get; init; } = string.Empty;

    [YamlMember(Alias = "port")]
    public int Port { get; init; }

    [YamlMember(Alias = "name")]
    public string Name { get; init; } = string.Empty;

    [YamlMember(Alias = "user")]
    public string User { get; init; } = string.Empty;

    [YamlMember(Alias = "password")]
    public string Password { get; init; } = string.Empty;

    [YamlMember(Alias = "ssl_mode")]
    public string? SslMode { get; init; }

    [YamlMember(Alias = "max_open_conns")]
    public int? MaxOpenConns { get; init; }

    [YamlMember(Alias = "max_idle_conns")]
    public int? MaxIdleConns { get; init; }

    [YamlMember(Alias = "conn_max_lifetime")]
    public string? ConnMaxLifetime { get; init; }
}

public sealed record SaslConfig
{
    [YamlMember(Alias = "mechanism")]
    public string Mechanism { get; init; } = string.Empty;

    [YamlMember(Alias = "username")]
    public string Username { get; init; } = string.Empty;

    [YamlMember(Alias = "password")]
    public string Password { get; init; } = string.Empty;
}

public sealed record TlsConfig
{
    [YamlMember(Alias = "ca_cert_path")]
    public string? CaCertPath { get; init; }
}

public sealed record TopicsConfig
{
    [YamlMember(Alias = "publish")]
    public string[] Publish { get; init; } = [];

    [YamlMember(Alias = "subscribe")]
    public string[] Subscribe { get; init; } = [];
}

public sealed record KafkaConfigSection
{
    [YamlMember(Alias = "brokers")]
    public string[] Brokers { get; init; } = [];

    [YamlMember(Alias = "consumer_group")]
    public string ConsumerGroup { get; init; } = string.Empty;

    [YamlMember(Alias = "security_protocol")]
    public string SecurityProtocol { get; init; } = string.Empty;

    [YamlMember(Alias = "sasl")]
    public SaslConfig? Sasl { get; init; }

    [YamlMember(Alias = "tls")]
    public TlsConfig? Tls { get; init; }

    [YamlMember(Alias = "topics")]
    public TopicsConfig? Topics { get; init; }
}

public sealed record RedisSection
{
    [YamlMember(Alias = "host")]
    public string Host { get; init; } = string.Empty;

    [YamlMember(Alias = "port")]
    public int Port { get; init; }

    [YamlMember(Alias = "password")]
    public string? Password { get; init; }

    [YamlMember(Alias = "db")]
    public int? Db { get; init; }

    [YamlMember(Alias = "pool_size")]
    public int? PoolSize { get; init; }
}

public sealed record LogConfig
{
    [YamlMember(Alias = "level")]
    public string Level { get; init; } = "info";

    [YamlMember(Alias = "format")]
    public string Format { get; init; } = "json";
}

public sealed record TraceConfig
{
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; init; }

    [YamlMember(Alias = "endpoint")]
    public string? Endpoint { get; init; }

    [YamlMember(Alias = "sample_rate")]
    public double? SampleRate { get; init; }
}

public sealed record MetricsConfig
{
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; init; }

    [YamlMember(Alias = "path")]
    public string? Path { get; init; }
}

public sealed record ObservabilitySection
{
    [YamlMember(Alias = "log")]
    public LogConfig Log { get; init; } = new();

    [YamlMember(Alias = "trace")]
    public TraceConfig? Trace { get; init; }

    [YamlMember(Alias = "metrics")]
    public MetricsConfig? Metrics { get; init; }
}

public sealed record JwtConfig
{
    [YamlMember(Alias = "issuer")]
    public string Issuer { get; init; } = string.Empty;

    [YamlMember(Alias = "audience")]
    public string Audience { get; init; } = string.Empty;

    [YamlMember(Alias = "public_key_path")]
    public string? PublicKeyPath { get; init; }
}

public sealed record OidcConfig
{
    [YamlMember(Alias = "discovery_url")]
    public string DiscoveryUrl { get; init; } = string.Empty;

    [YamlMember(Alias = "client_id")]
    public string ClientId { get; init; } = string.Empty;

    [YamlMember(Alias = "client_secret")]
    public string? ClientSecret { get; init; }

    [YamlMember(Alias = "redirect_uri")]
    public string RedirectUri { get; init; } = string.Empty;

    [YamlMember(Alias = "scopes")]
    public string[] Scopes { get; init; } = [];

    [YamlMember(Alias = "jwks_uri")]
    public string JwksUri { get; init; } = string.Empty;

    [YamlMember(Alias = "jwks_cache_ttl")]
    public string? JwksCacheTtl { get; init; }
}

public sealed record AuthSection
{
    [YamlMember(Alias = "jwt")]
    public JwtConfig Jwt { get; init; } = new();

    [YamlMember(Alias = "oidc")]
    public OidcConfig? Oidc { get; init; }
}

public sealed record AppConfig
{
    [YamlMember(Alias = "app")]
    public AppSection App { get; init; } = new();

    [YamlMember(Alias = "server")]
    public ServerSection Server { get; init; } = new();

    [YamlMember(Alias = "grpc")]
    public GrpcSection? Grpc { get; init; }

    [YamlMember(Alias = "database")]
    public DatabaseSection? Database { get; init; }

    [YamlMember(Alias = "kafka")]
    public KafkaConfigSection? Kafka { get; init; }

    [YamlMember(Alias = "redis")]
    public RedisSection? Redis { get; init; }

    [YamlMember(Alias = "redis_session")]
    public RedisSection? RedisSession { get; init; }

    [YamlMember(Alias = "observability")]
    public ObservabilitySection Observability { get; init; } = new();

    [YamlMember(Alias = "auth")]
    public AuthSection? Auth { get; init; }
}
