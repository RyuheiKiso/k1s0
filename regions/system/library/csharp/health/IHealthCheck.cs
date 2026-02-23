namespace K1s0.System.Health;

public enum HealthStatus
{
    Healthy,
    Degraded,
    Unhealthy,
}

public record CheckResult(HealthStatus Status, string? Message = null);

public record HealthResponse(
    HealthStatus Status,
    IReadOnlyDictionary<string, CheckResult> Checks,
    DateTimeOffset Timestamp);

public interface IHealthCheck
{
    string Name { get; }

    Task<CheckResult> CheckAsync(CancellationToken ct = default);
}
