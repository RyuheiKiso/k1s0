using K1s0.System.QuotaClient;

namespace K1s0.System.QuotaClient.Tests;

public class InMemoryQuotaClientTests
{
    private readonly InMemoryQuotaClient _client = new();

    [Fact]
    public async Task Check_Allowed()
    {
        var status = await _client.CheckAsync("q1", 100);
        Assert.True(status.Allowed);
        Assert.Equal(1000UL, status.Remaining);
        Assert.Equal(1000UL, status.Limit);
    }

    [Fact]
    public async Task Check_Exceeded()
    {
        await _client.IncrementAsync("q1", 900);
        var status = await _client.CheckAsync("q1", 200);
        Assert.False(status.Allowed);
        Assert.Equal(100UL, status.Remaining);
    }

    [Fact]
    public async Task Increment_Accumulates()
    {
        await _client.IncrementAsync("q1", 300);
        var usage = await _client.IncrementAsync("q1", 200);
        Assert.Equal(500UL, usage.Used);
        Assert.Equal(1000UL, usage.Limit);
    }

    [Fact]
    public async Task GetUsage_ReturnsCurrentUsage()
    {
        await _client.IncrementAsync("q1", 100);
        var usage = await _client.GetUsageAsync("q1");
        Assert.Equal(100UL, usage.Used);
        Assert.Equal("q1", usage.QuotaId);
    }

    [Fact]
    public async Task GetPolicy_ReturnsDefault()
    {
        var policy = await _client.GetPolicyAsync("q1");
        Assert.Equal("q1", policy.QuotaId);
        Assert.Equal(1000UL, policy.Limit);
        Assert.Equal(QuotaPeriod.Daily, policy.Period);
        Assert.Equal("fixed", policy.ResetStrategy);
    }

    [Fact]
    public async Task GetPolicy_ReturnsCustom()
    {
        _client.SetPolicy("q1", new QuotaPolicy("q1", 5000, QuotaPeriod.Monthly, "sliding"));
        var policy = await _client.GetPolicyAsync("q1");
        Assert.Equal(5000UL, policy.Limit);
        Assert.Equal(QuotaPeriod.Monthly, policy.Period);
    }
}
