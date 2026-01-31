using System.Collections.Concurrent;

namespace K1s0.Auth.Policy;

/// <summary>
/// Wraps an <see cref="IPolicyRepository"/> with time-based caching.
/// </summary>
public class CachedPolicyRepository : IPolicyRepository
{
    private readonly IPolicyRepository _inner;
    private readonly TimeSpan _ttl;
    private readonly ConcurrentDictionary<string, CacheEntry> _cache = new();

    /// <summary>
    /// Initializes a new instance of the <see cref="CachedPolicyRepository"/> class.
    /// </summary>
    /// <param name="inner">The underlying policy repository.</param>
    /// <param name="ttl">The cache time-to-live.</param>
    public CachedPolicyRepository(IPolicyRepository inner, TimeSpan ttl)
    {
        _inner = inner ?? throw new ArgumentNullException(nameof(inner));
        _ttl = ttl;
    }

    /// <inheritdoc />
    public async Task<IReadOnlyList<PolicyRule>> GetRulesAsync(string resource, CancellationToken ct = default)
    {
        if (_cache.TryGetValue(resource, out var entry) && entry.ExpiresAt > DateTime.UtcNow)
        {
            return entry.Rules;
        }

        var rules = await _inner.GetRulesAsync(resource, ct).ConfigureAwait(false);
        _cache[resource] = new CacheEntry(rules, DateTime.UtcNow.Add(_ttl));
        return rules;
    }

    private sealed record CacheEntry(IReadOnlyList<PolicyRule> Rules, DateTime ExpiresAt);
}
