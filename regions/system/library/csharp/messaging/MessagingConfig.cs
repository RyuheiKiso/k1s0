namespace K1s0.System.Messaging;

public sealed record SaslConfig(string Mechanism, string Username, string Password);

public sealed record TlsConfig(string CaCertPath);

public sealed record MessagingConfig(
    string[] Brokers,
    string SecurityProtocol = "PLAINTEXT",
    SaslConfig? SaslConfig = null,
    TlsConfig? TlsConfig = null,
    ConsumerConfig? ConsumerConfig = null);
