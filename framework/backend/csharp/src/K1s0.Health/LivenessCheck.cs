using Microsoft.Extensions.Diagnostics.HealthChecks;

namespace K1s0.Health;

/// <summary>
/// Basic liveness probe that always reports healthy.
/// Indicates the process is running and not deadlocked.
/// </summary>
public class LivenessCheck : IHealthCheck
{
    /// <inheritdoc />
    public Task<HealthCheckResult> CheckHealthAsync(
        HealthCheckContext context,
        CancellationToken cancellationToken = default)
    {
        return Task.FromResult(HealthCheckResult.Healthy("Service is alive."));
    }
}
