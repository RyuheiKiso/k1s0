namespace K1s0.System.QuotaClient;

public interface IQuotaClient
{
    Task<QuotaStatus> CheckAsync(string quotaId, ulong amount, CancellationToken ct = default);

    Task<QuotaUsage> IncrementAsync(string quotaId, ulong amount, CancellationToken ct = default);

    Task<QuotaUsage> GetUsageAsync(string quotaId, CancellationToken ct = default);

    Task<QuotaPolicy> GetPolicyAsync(string quotaId, CancellationToken ct = default);
}
