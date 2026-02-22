using Confluent.Kafka;

namespace K1s0.System.Kafka;

public sealed class KafkaHealthCheck : IKafkaHealthCheck
{
    private readonly KafkaConfig _config;

    public KafkaHealthCheck(KafkaConfig config)
    {
        _config = config;
    }

    public Task<HealthCheckResult> CheckHealthAsync(CancellationToken ct = default)
    {
        try
        {
            var adminConfig = new AdminClientConfig
            {
                BootstrapServers = _config.BootstrapServers,
            };

            if (_config.Sasl is not null)
            {
                adminConfig.SaslMechanism = Enum.Parse<SaslMechanism>(_config.Sasl.Mechanism, ignoreCase: true);
                adminConfig.SaslUsername = _config.Sasl.Username;
                adminConfig.SaslPassword = _config.Sasl.Password;
            }

            if (_config.UsesTls)
            {
                adminConfig.SecurityProtocol = Enum.Parse<SecurityProtocol>(_config.SecurityProtocol.Replace("_", string.Empty), ignoreCase: true);
                if (_config.Tls?.CaCertPath is not null)
                {
                    adminConfig.SslCaLocation = _config.Tls.CaCertPath;
                }
            }

            using var adminClient = new AdminClientBuilder(adminConfig).Build();
            var metadata = adminClient.GetMetadata(TimeSpan.FromSeconds(5));

            return Task.FromResult(metadata.Brokers.Count > 0
                ? HealthCheckResult.Healthy
                : HealthCheckResult.Unhealthy);
        }
        catch (Exception)
        {
            return Task.FromResult(HealthCheckResult.Unhealthy);
        }
    }
}
