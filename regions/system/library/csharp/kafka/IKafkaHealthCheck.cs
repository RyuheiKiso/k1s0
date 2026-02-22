namespace K1s0.System.Kafka;

public interface IKafkaHealthCheck
{
    Task<HealthCheckResult> CheckHealthAsync(CancellationToken ct = default);
}

public enum HealthCheckResult
{
    Healthy,
    Unhealthy,
}
