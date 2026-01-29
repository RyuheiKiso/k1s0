using Microsoft.Extensions.Diagnostics.HealthChecks;

namespace K1s0.Health;

/// <summary>
/// Readiness probe that checks whether all registered dependency checks pass.
/// Use <see cref="AddDependencyCheck"/> to register custom checks.
/// </summary>
public class ReadinessCheck : IHealthCheck
{
    private readonly List<Func<CancellationToken, Task<bool>>> _checks = [];

    /// <summary>
    /// Adds a dependency check. The function should return true if the dependency is available.
    /// </summary>
    /// <param name="check">An async function that returns true when the dependency is ready.</param>
    public void AddDependencyCheck(Func<CancellationToken, Task<bool>> check)
    {
        ArgumentNullException.ThrowIfNull(check);
        _checks.Add(check);
    }

    /// <inheritdoc />
    public async Task<HealthCheckResult> CheckHealthAsync(
        HealthCheckContext context,
        CancellationToken cancellationToken = default)
    {
        if (_checks.Count == 0)
        {
            return HealthCheckResult.Healthy("No dependencies registered; assuming ready.");
        }

        foreach (var check in _checks)
        {
            try
            {
                bool ready = await check(cancellationToken).ConfigureAwait(false);
                if (!ready)
                {
                    return HealthCheckResult.Unhealthy("One or more dependencies are not ready.");
                }
            }
            catch (Exception ex)
            {
                return HealthCheckResult.Unhealthy("Dependency check failed.", ex);
            }
        }

        return HealthCheckResult.Healthy("All dependencies are ready.");
    }
}
