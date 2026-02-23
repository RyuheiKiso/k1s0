namespace K1s0.System.Cache;

public interface ICacheClient
{
    Task<string?> GetAsync(string key);

    Task SetAsync(string key, string value, TimeSpan? ttl = null);

    Task<bool> DeleteAsync(string key);

    Task<bool> ExistsAsync(string key);

    Task<bool> SetNxAsync(string key, string value, TimeSpan ttl);
}
