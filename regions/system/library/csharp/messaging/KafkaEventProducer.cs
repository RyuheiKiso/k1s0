using Confluent.Kafka;

namespace K1s0.System.Messaging;

public sealed class KafkaEventProducer : IEventProducer
{
    private readonly IProducer<string, byte[]> _producer;

    public KafkaEventProducer(IProducer<string, byte[]> producer)
    {
        _producer = producer ?? throw new ArgumentNullException(nameof(producer));
    }

    public KafkaEventProducer(MessagingConfig config)
    {
        var producerConfig = new ProducerConfig
        {
            BootstrapServers = string.Join(",", config.Brokers),
            SecurityProtocol = ParseSecurityProtocol(config.SecurityProtocol),
            Acks = Acks.All,
            EnableIdempotence = true,
        };

        if (config.SaslConfig is not null)
        {
            producerConfig.SaslMechanism = ParseSaslMechanism(config.SaslConfig.Mechanism);
            producerConfig.SaslUsername = config.SaslConfig.Username;
            producerConfig.SaslPassword = config.SaslConfig.Password;
        }

        if (config.TlsConfig is not null)
        {
            producerConfig.SslCaLocation = config.TlsConfig.CaCertPath;
        }

        _producer = new ProducerBuilder<string, byte[]>(producerConfig).Build();
    }

    public async Task PublishAsync(EventEnvelope envelope, CancellationToken ct = default)
    {
        var message = BuildMessage(envelope);

        try
        {
            await _producer.ProduceAsync(envelope.Topic, message, ct).ConfigureAwait(false);
        }
        catch (ProduceException<string, byte[]> ex)
        {
            throw new MessagingException("Publish", $"Failed to publish to topic '{envelope.Topic}'.", ex);
        }
    }

    public async Task PublishBatchAsync(IReadOnlyList<EventEnvelope> envelopes, CancellationToken ct = default)
    {
        foreach (var envelope in envelopes)
        {
            await PublishAsync(envelope, ct).ConfigureAwait(false);
        }
    }

    public ValueTask DisposeAsync()
    {
        _producer.Flush(TimeSpan.FromSeconds(10));
        _producer.Dispose();
        return ValueTask.CompletedTask;
    }

    private static Message<string, byte[]> BuildMessage(EventEnvelope envelope)
    {
        var message = new Message<string, byte[]>
        {
            Key = envelope.Key ?? string.Empty,
            Value = envelope.Payload,
        };

        if (envelope.Headers is { Count: > 0 })
        {
            message.Headers = new Headers();
            foreach (var (key, value) in envelope.Headers)
            {
                message.Headers.Add(key, global::System.Text.Encoding.UTF8.GetBytes(value));
            }
        }

        return message;
    }

    private static SecurityProtocol ParseSecurityProtocol(string protocol)
    {
        return protocol.ToUpperInvariant() switch
        {
            "PLAINTEXT" => SecurityProtocol.Plaintext,
            "SSL" => SecurityProtocol.Ssl,
            "SASL_PLAINTEXT" => SecurityProtocol.SaslPlaintext,
            "SASL_SSL" => SecurityProtocol.SaslSsl,
            _ => throw new MessagingException("Config", $"Unknown security protocol: {protocol}"),
        };
    }

    private static SaslMechanism ParseSaslMechanism(string mechanism)
    {
        return mechanism.ToUpperInvariant() switch
        {
            "PLAIN" => SaslMechanism.Plain,
            "SCRAM-SHA-256" => SaslMechanism.ScramSha256,
            "SCRAM-SHA-512" => SaslMechanism.ScramSha512,
            "OAUTHBEARER" => SaslMechanism.OAuthBearer,
            _ => throw new MessagingException("Config", $"Unknown SASL mechanism: {mechanism}"),
        };
    }
}
