namespace K1s0.System.Health;

public class HealthChecker
{
    private readonly List<IHealthCheck> _checks = new();

    public void Add(IHealthCheck check) => _checks.Add(check);

    public async Task<HealthResponse> RunAllAsync(CancellationToken ct = default)
    {
        var results = new Dictionary<string, CheckResult>();
        var worstStatus = HealthStatus.Healthy;

        foreach (var check in _checks)
        {
            var result = await check.CheckAsync(ct).ConfigureAwait(false);
            results[check.Name] = result;
            if (result.Status > worstStatus)
            {
                worstStatus = result.Status;
            }
        }

        return new HealthResponse(worstStatus, results, DateTimeOffset.UtcNow);
    }
}
