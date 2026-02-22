namespace K1s0.System.Kafka;

public sealed class KafkaConfigBuilder
{
    private string[] _brokers = [];
    private string _securityProtocol = "PLAINTEXT";
    private string? _consumerGroup;
    private SaslConfig? _sasl;
    private TlsConfig? _tls;
    private int? _messageMaxBytes;
    private int? _requestTimeoutMs;
    private int? _sessionTimeoutMs;

    public KafkaConfigBuilder WithBrokers(params string[] brokers)
    {
        _brokers = brokers;
        return this;
    }

    public KafkaConfigBuilder WithSecurityProtocol(string protocol)
    {
        _securityProtocol = protocol;
        return this;
    }

    public KafkaConfigBuilder WithConsumerGroup(string consumerGroup)
    {
        _consumerGroup = consumerGroup;
        return this;
    }

    public KafkaConfigBuilder WithSasl(string mechanism, string username, string password)
    {
        _sasl = new SaslConfig
        {
            Mechanism = mechanism,
            Username = username,
            Password = password,
        };
        return this;
    }

    public KafkaConfigBuilder WithTls(string? caCertPath = null)
    {
        _tls = new TlsConfig { CaCertPath = caCertPath };
        return this;
    }

    public KafkaConfigBuilder WithMessageMaxBytes(int maxBytes)
    {
        _messageMaxBytes = maxBytes;
        return this;
    }

    public KafkaConfigBuilder WithRequestTimeoutMs(int timeoutMs)
    {
        _requestTimeoutMs = timeoutMs;
        return this;
    }

    public KafkaConfigBuilder WithSessionTimeoutMs(int timeoutMs)
    {
        _sessionTimeoutMs = timeoutMs;
        return this;
    }

    public KafkaConfig Build()
    {
        if (_brokers.Length == 0)
        {
            throw new KafkaException(KafkaException.ErrorCodes.Config, "At least one broker must be specified");
        }

        return new KafkaConfig
        {
            Brokers = _brokers,
            SecurityProtocol = _securityProtocol,
            ConsumerGroup = _consumerGroup,
            Sasl = _sasl,
            Tls = _tls,
            MessageMaxBytes = _messageMaxBytes,
            RequestTimeoutMs = _requestTimeoutMs,
            SessionTimeoutMs = _sessionTimeoutMs,
        };
    }
}
