using K1s0.System.QuotaClient;

namespace K1s0.System.QuotaClient.Tests;

public class CachedQuotaClientTests
{
    [Fact]
    public async Task CachesPolicy()
    {
        var inner = new InMemoryQuotaClient();
        var cached = new CachedQuotaClient(inner, TimeSpan.FromMinutes(1));

        var p1 = await cached.GetPolicyAsync("q1");
        inner.SetPolicy("q1", new QuotaPolicy("q1", 9999, QuotaPeriod.Hourly, "fixed"));
        var p2 = await cached.GetPolicyAsync("q1");
        Assert.Equal(p1.Limit, p2.Limit);
    }

    [Fact]
    public async Task DelegatesCheck()
    {
        var inner = new InMemoryQuotaClient();
        var cached = new CachedQuotaClient(inner, TimeSpan.FromMinutes(1));
        var status = await cached.CheckAsync("q1", 100);
        Assert.True(status.Allowed);
    }

    [Fact]
    public async Task DelegatesIncrement()
    {
        var inner = new InMemoryQuotaClient();
        var cached = new CachedQuotaClient(inner, TimeSpan.FromMinutes(1));
        var usage = await cached.IncrementAsync("q1", 100);
        Assert.Equal(100UL, usage.Used);
    }

    [Fact]
    public async Task DelegatesGetUsage()
    {
        var inner = new InMemoryQuotaClient();
        var cached = new CachedQuotaClient(inner, TimeSpan.FromMinutes(1));
        await cached.IncrementAsync("q1", 50);
        var usage = await cached.GetUsageAsync("q1");
        Assert.Equal(50UL, usage.Used);
    }
}
