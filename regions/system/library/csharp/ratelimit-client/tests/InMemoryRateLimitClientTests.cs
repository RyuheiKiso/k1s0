using K1s0.System.RateLimitClient;

namespace K1s0.System.RateLimitClient.Tests;

public class InMemoryRateLimitClientTests
{
    [Fact]
    public async Task Check_Allowed()
    {
        var client = new InMemoryRateLimitClient();
        var status = await client.CheckAsync("test-key", 1);

        Assert.True(status.Allowed);
        Assert.Equal(99u, status.Remaining);
        Assert.Null(status.RetryAfterSecs);
    }

    [Fact]
    public async Task Check_Denied_WhenOverLimit()
    {
        var client = new InMemoryRateLimitClient();
        client.SetPolicy("limited", new RateLimitPolicy("limited", 2, 60, "fixed_window"));

        await client.ConsumeAsync("limited", 2);
        var status = await client.CheckAsync("limited", 1);

        Assert.False(status.Allowed);
        Assert.Equal(0u, status.Remaining);
        Assert.Equal(60ul, status.RetryAfterSecs);
    }

    [Fact]
    public async Task Consume_Success()
    {
        var client = new InMemoryRateLimitClient();
        var result = await client.ConsumeAsync("test-key", 1);

        Assert.Equal(99u, result.Remaining);
        Assert.Equal(1u, client.GetUsedCount("test-key"));
    }

    [Fact]
    public async Task Consume_ThrowsWhenExceeded()
    {
        var client = new InMemoryRateLimitClient();
        client.SetPolicy("small", new RateLimitPolicy("small", 1, 60, "token_bucket"));

        await client.ConsumeAsync("small", 1);
        await Assert.ThrowsAsync<RateLimitException>(() => client.ConsumeAsync("small", 1));
    }

    [Fact]
    public async Task GetLimit_DefaultPolicy()
    {
        var client = new InMemoryRateLimitClient();
        var policy = await client.GetLimitAsync("unknown");

        Assert.Equal(100u, policy.Limit);
        Assert.Equal(3600ul, policy.WindowSecs);
        Assert.Equal("token_bucket", policy.Algorithm);
    }

    [Fact]
    public async Task GetLimit_CustomPolicy()
    {
        var client = new InMemoryRateLimitClient();
        client.SetPolicy("tenant:T1", new RateLimitPolicy("tenant:T1", 50, 1800, "sliding_window"));

        var policy = await client.GetLimitAsync("tenant:T1");

        Assert.Equal("tenant:T1", policy.Key);
        Assert.Equal(50u, policy.Limit);
        Assert.Equal("sliding_window", policy.Algorithm);
    }

    [Fact]
    public void RateLimitException_ContainsCodeAndRetry()
    {
        var ex = new RateLimitException("exceeded", "LIMIT_EXCEEDED", 30);
        Assert.Equal("LIMIT_EXCEEDED", ex.Code);
        Assert.Equal(30ul, ex.RetryAfterSecs);
    }

    [Fact]
    public async Task Check_MultipleCosts()
    {
        var client = new InMemoryRateLimitClient();
        client.SetPolicy("cost-key", new RateLimitPolicy("cost-key", 10, 60, "fixed_window"));

        var s1 = await client.CheckAsync("cost-key", 5);
        Assert.True(s1.Allowed);
        Assert.Equal(5u, s1.Remaining);

        var s2 = await client.CheckAsync("cost-key", 11);
        Assert.False(s2.Allowed);
    }
}
