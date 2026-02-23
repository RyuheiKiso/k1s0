namespace K1s0.System.TenantClient;

public enum TenantStatus
{
    Active,
    Suspended,
    Deleted,
}

public record Tenant(
    string Id,
    string Name,
    TenantStatus Status,
    string Plan,
    IReadOnlyDictionary<string, string> Settings,
    DateTimeOffset CreatedAt);

public record TenantFilter(TenantStatus? Status = null, string? Plan = null);

public class TenantSettings
{
    private readonly IReadOnlyDictionary<string, string> _values;

    public TenantSettings(IReadOnlyDictionary<string, string> values)
    {
        _values = values;
    }

    public IReadOnlyDictionary<string, string> Values => _values;

    public string? Get(string key)
    {
        return _values.TryGetValue(key, out var value) ? value : null;
    }
}

public record TenantClientConfig(
    string ServerUrl,
    TimeSpan? CacheTtl = null,
    int CacheMaxCapacity = 1000);
