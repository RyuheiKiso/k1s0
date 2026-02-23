using K1s0.System.Health;

namespace K1s0.System.Health.Tests;

public class HealthCheckerTests
{
    private class FakeCheck : IHealthCheck
    {
        public string Name { get; }
        private readonly CheckResult _result;

        public FakeCheck(string name, CheckResult result)
        {
            Name = name;
            _result = result;
        }

        public Task<CheckResult> CheckAsync(CancellationToken ct = default) =>
            Task.FromResult(_result);
    }

    [Fact]
    public async Task RunAll_NoChecks_ReturnsHealthy()
    {
        var checker = new HealthChecker();
        var resp = await checker.RunAllAsync();

        Assert.Equal(HealthStatus.Healthy, resp.Status);
        Assert.Empty(resp.Checks);
    }

    [Fact]
    public async Task RunAll_AllHealthy_ReturnsHealthy()
    {
        var checker = new HealthChecker();
        checker.Add(new FakeCheck("db", new CheckResult(HealthStatus.Healthy)));
        checker.Add(new FakeCheck("cache", new CheckResult(HealthStatus.Healthy)));

        var resp = await checker.RunAllAsync();

        Assert.Equal(HealthStatus.Healthy, resp.Status);
        Assert.Equal(2, resp.Checks.Count);
    }

    [Fact]
    public async Task RunAll_OneDegraded_ReturnsDegraded()
    {
        var checker = new HealthChecker();
        checker.Add(new FakeCheck("db", new CheckResult(HealthStatus.Healthy)));
        checker.Add(new FakeCheck("cache", new CheckResult(HealthStatus.Degraded, "slow")));

        var resp = await checker.RunAllAsync();

        Assert.Equal(HealthStatus.Degraded, resp.Status);
    }

    [Fact]
    public async Task RunAll_OneUnhealthy_ReturnsUnhealthy()
    {
        var checker = new HealthChecker();
        checker.Add(new FakeCheck("db", new CheckResult(HealthStatus.Unhealthy, "down")));
        checker.Add(new FakeCheck("cache", new CheckResult(HealthStatus.Healthy)));

        var resp = await checker.RunAllAsync();

        Assert.Equal(HealthStatus.Unhealthy, resp.Status);
        Assert.Equal("down", resp.Checks["db"].Message);
    }

    [Fact]
    public async Task RunAll_HasTimestamp()
    {
        var checker = new HealthChecker();
        var before = DateTimeOffset.UtcNow;
        var resp = await checker.RunAllAsync();
        var after = DateTimeOffset.UtcNow;

        Assert.InRange(resp.Timestamp, before, after);
    }
}
