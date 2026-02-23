using Microsoft.Extensions.Caching.Memory;

namespace K1s0.System.QuotaClient;

public class CachedQuotaClient : IQuotaClient
{
    private readonly IQuotaClient _inner;
    private readonly MemoryCache _policyCache = new(new MemoryCacheOptions());
    private readonly TimeSpan _policyTtl;

    public CachedQuotaClient(IQuotaClient inner, TimeSpan policyTtl)
    {
        _inner = inner;
        _policyTtl = policyTtl;
    }

    public Task<QuotaStatus> CheckAsync(string quotaId, ulong amount, CancellationToken ct = default)
        => _inner.CheckAsync(quotaId, amount, ct);

    public Task<QuotaUsage> IncrementAsync(string quotaId, ulong amount, CancellationToken ct = default)
        => _inner.IncrementAsync(quotaId, amount, ct);

    public Task<QuotaUsage> GetUsageAsync(string quotaId, CancellationToken ct = default)
        => _inner.GetUsageAsync(quotaId, ct);

    public async Task<QuotaPolicy> GetPolicyAsync(string quotaId, CancellationToken ct = default)
    {
        if (_policyCache.TryGetValue(quotaId, out QuotaPolicy? cached) && cached is not null)
        {
            return cached;
        }

        var policy = await _inner.GetPolicyAsync(quotaId, ct);
        _policyCache.Set(quotaId, policy, _policyTtl);
        return policy;
    }
}
