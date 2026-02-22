using Confluent.Kafka;

namespace K1s0.System.Messaging;

public sealed class KafkaEventConsumer : IEventConsumer
{
    private readonly IConsumer<string, byte[]> _consumer;

    public KafkaEventConsumer(IConsumer<string, byte[]> consumer)
    {
        _consumer = consumer ?? throw new ArgumentNullException(nameof(consumer));
    }

    public KafkaEventConsumer(MessagingConfig config, params string[] topics)
    {
        if (config.ConsumerConfig is null)
        {
            throw new MessagingException("Config", "ConsumerConfig is required for KafkaEventConsumer.");
        }

        var consumerConfig = new Confluent.Kafka.ConsumerConfig
        {
            BootstrapServers = string.Join(",", config.Brokers),
            GroupId = config.ConsumerConfig.GroupId,
            AutoOffsetReset = ParseAutoOffsetReset(config.ConsumerConfig.AutoOffsetReset),
            EnableAutoCommit = config.ConsumerConfig.EnableAutoCommit,
            SecurityProtocol = ParseSecurityProtocol(config.SecurityProtocol),
        };

        if (config.ConsumerConfig.MaxPollIntervalMs.HasValue)
        {
            consumerConfig.MaxPollIntervalMs = config.ConsumerConfig.MaxPollIntervalMs.Value;
        }

        if (config.ConsumerConfig.SessionTimeoutMs.HasValue)
        {
            consumerConfig.SessionTimeoutMs = config.ConsumerConfig.SessionTimeoutMs.Value;
        }

        if (config.SaslConfig is not null)
        {
            consumerConfig.SaslMechanism = ParseSaslMechanism(config.SaslConfig.Mechanism);
            consumerConfig.SaslUsername = config.SaslConfig.Username;
            consumerConfig.SaslPassword = config.SaslConfig.Password;
        }

        if (config.TlsConfig is not null)
        {
            consumerConfig.SslCaLocation = config.TlsConfig.CaCertPath;
        }

        _consumer = new ConsumerBuilder<string, byte[]>(consumerConfig).Build();
        _consumer.Subscribe(topics);
    }

    public Task<ConsumedMessage> ReceiveAsync(CancellationToken ct = default)
    {
        try
        {
            var result = _consumer.Consume(ct);

            var headers = new Dictionary<string, string>();
            if (result.Message.Headers is not null)
            {
                foreach (var header in result.Message.Headers)
                {
                    headers[header.Key] = global::System.Text.Encoding.UTF8.GetString(header.GetValueBytes());
                }
            }

            return Task.FromResult(new ConsumedMessage(
                Topic: result.Topic,
                Partition: result.Partition.Value,
                Offset: result.Offset.Value,
                Key: result.Message.Key,
                Payload: result.Message.Value,
                Headers: headers));
        }
        catch (ConsumeException ex)
        {
            throw new MessagingException("Consume", "Failed to consume message.", ex);
        }
    }

    public Task CommitAsync(ConsumedMessage message, CancellationToken ct = default)
    {
        try
        {
            _consumer.Commit([new TopicPartitionOffset(
                message.Topic,
                new Partition(message.Partition),
                new Offset(message.Offset + 1))]);

            return Task.CompletedTask;
        }
        catch (KafkaException ex)
        {
            throw new MessagingException("Consume", "Failed to commit offset.", ex);
        }
    }

    public ValueTask DisposeAsync()
    {
        _consumer.Close();
        _consumer.Dispose();
        return ValueTask.CompletedTask;
    }

    private static AutoOffsetReset ParseAutoOffsetReset(string value)
    {
        return value.ToLowerInvariant() switch
        {
            "earliest" => AutoOffsetReset.Earliest,
            "latest" => AutoOffsetReset.Latest,
            "error" => AutoOffsetReset.Error,
            _ => throw new MessagingException("Config", $"Unknown auto offset reset: {value}"),
        };
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
