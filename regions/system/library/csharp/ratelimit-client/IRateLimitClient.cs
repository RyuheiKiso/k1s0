namespace K1s0.System.RateLimitClient;

public interface IRateLimitClient
{
    Task<RateLimitStatus> CheckAsync(string key, uint cost, CancellationToken ct = default);

    Task<RateLimitResult> ConsumeAsync(string key, uint cost, CancellationToken ct = default);

    Task<RateLimitPolicy> GetLimitAsync(string key, CancellationToken ct = default);
}
