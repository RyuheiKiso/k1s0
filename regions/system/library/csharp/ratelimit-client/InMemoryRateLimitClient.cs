namespace K1s0.System.RateLimitClient;

public class InMemoryRateLimitClient : IRateLimitClient
{
    private readonly Dictionary<string, uint> _counters = new();
    private readonly Dictionary<string, RateLimitPolicy> _policies = new();

    private static readonly RateLimitPolicy DefaultPolicy = new("default", 100, 3600, "token_bucket");

    public void SetPolicy(string key, RateLimitPolicy policy) => _policies[key] = policy;

    private RateLimitPolicy GetPolicy(string key) =>
        _policies.TryGetValue(key, out var p) ? p : DefaultPolicy;

    public Task<RateLimitStatus> CheckAsync(string key, uint cost, CancellationToken ct = default)
    {
        var policy = GetPolicy(key);
        _counters.TryGetValue(key, out var used);
        var resetAt = DateTimeOffset.UtcNow.AddSeconds(policy.WindowSecs);

        if (used + cost > policy.Limit)
        {
            return Task.FromResult(new RateLimitStatus(false, 0, resetAt, policy.WindowSecs));
        }

        var remaining = policy.Limit - used - cost;
        return Task.FromResult(new RateLimitStatus(true, remaining, resetAt));
    }

    public Task<RateLimitResult> ConsumeAsync(string key, uint cost, CancellationToken ct = default)
    {
        var policy = GetPolicy(key);
        _counters.TryGetValue(key, out var used);

        if (used + cost > policy.Limit)
        {
            throw new RateLimitException(
                $"Rate limit exceeded for key: {key}",
                "LIMIT_EXCEEDED",
                policy.WindowSecs);
        }

        _counters[key] = used + cost;
        var remaining = policy.Limit - (used + cost);
        var resetAt = DateTimeOffset.UtcNow.AddSeconds(policy.WindowSecs);

        return Task.FromResult(new RateLimitResult(remaining, resetAt));
    }

    public Task<RateLimitPolicy> GetLimitAsync(string key, CancellationToken ct = default)
    {
        return Task.FromResult(GetPolicy(key));
    }

    public uint GetUsedCount(string key)
    {
        _counters.TryGetValue(key, out var count);
        return count;
    }
}
