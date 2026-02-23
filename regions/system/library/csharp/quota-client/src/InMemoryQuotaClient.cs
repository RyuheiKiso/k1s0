namespace K1s0.System.QuotaClient;

public class InMemoryQuotaClient : IQuotaClient
{
    private readonly Dictionary<string, UsageEntry> _usages = new();
    private readonly Dictionary<string, QuotaPolicy> _policies = new();
    private readonly object _lock = new();

    public void SetPolicy(string quotaId, QuotaPolicy policy)
    {
        lock (_lock)
        {
            _policies[quotaId] = policy;
        }
    }

    public Task<QuotaStatus> CheckAsync(string quotaId, ulong amount, CancellationToken ct = default)
    {
        lock (_lock)
        {
            var usage = GetOrCreateUsage(quotaId);
            var remaining = usage.Limit - usage.Used;
            return Task.FromResult(new QuotaStatus(
                Allowed: amount <= remaining,
                Remaining: remaining,
                Limit: usage.Limit,
                ResetAt: usage.ResetAt));
        }
    }

    public Task<QuotaUsage> IncrementAsync(string quotaId, ulong amount, CancellationToken ct = default)
    {
        lock (_lock)
        {
            var usage = GetOrCreateUsage(quotaId);
            usage.Used += amount;
            return Task.FromResult(new QuotaUsage(
                QuotaId: usage.QuotaId,
                Used: usage.Used,
                Limit: usage.Limit,
                Period: usage.Period,
                ResetAt: usage.ResetAt));
        }
    }

    public Task<QuotaUsage> GetUsageAsync(string quotaId, CancellationToken ct = default)
    {
        lock (_lock)
        {
            var usage = GetOrCreateUsage(quotaId);
            return Task.FromResult(new QuotaUsage(
                QuotaId: usage.QuotaId,
                Used: usage.Used,
                Limit: usage.Limit,
                Period: usage.Period,
                ResetAt: usage.ResetAt));
        }
    }

    public Task<QuotaPolicy> GetPolicyAsync(string quotaId, CancellationToken ct = default)
    {
        lock (_lock)
        {
            if (_policies.TryGetValue(quotaId, out var policy))
            {
                return Task.FromResult(policy);
            }

            return Task.FromResult(new QuotaPolicy(
                QuotaId: quotaId,
                Limit: 1000,
                Period: QuotaPeriod.Daily,
                ResetStrategy: "fixed"));
        }
    }

    private UsageEntry GetOrCreateUsage(string quotaId)
    {
        if (!_usages.TryGetValue(quotaId, out var entry))
        {
            _policies.TryGetValue(quotaId, out var policy);
            entry = new UsageEntry
            {
                QuotaId = quotaId,
                Used = 0,
                Limit = policy?.Limit ?? 1000,
                Period = policy?.Period ?? QuotaPeriod.Daily,
                ResetAt = DateTimeOffset.UtcNow.AddDays(1),
            };
            _usages[quotaId] = entry;
        }

        return entry;
    }

    private class UsageEntry
    {
        public required string QuotaId { get; init; }

        public ulong Used { get; set; }

        public required ulong Limit { get; init; }

        public required QuotaPeriod Period { get; init; }

        public required DateTimeOffset ResetAt { get; init; }
    }
}
