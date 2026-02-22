namespace K1s0.System.Kafka;

public sealed record KafkaConfig
{
    public required string[] Brokers { get; init; }

    public string SecurityProtocol { get; init; } = "PLAINTEXT";

    public string? ConsumerGroup { get; init; }

    public SaslConfig? Sasl { get; init; }

    public TlsConfig? Tls { get; init; }

    public int? MessageMaxBytes { get; init; }

    public int? RequestTimeoutMs { get; init; }

    public int? SessionTimeoutMs { get; init; }

    public string BootstrapServers => string.Join(",", Brokers);

    public bool UsesTls => SecurityProtocol is "SSL" or "SASL_SSL";
}

public sealed record SaslConfig
{
    public required string Mechanism { get; init; }

    public required string Username { get; init; }

    public required string Password { get; init; }
}

public sealed record TlsConfig
{
    public string? CaCertPath { get; init; }
}
