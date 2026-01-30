namespace K1s0.Cache.Patterns;

/// <summary>
/// Implements the cache-aside (lazy-loading) pattern.
/// On cache miss, the provided loader function is called and the result is stored in the cache.
/// </summary>
public sealed class CacheAside
{
    private readonly ICacheOperations _cache;
    private readonly TimeSpan? _defaultTtl;

    /// <summary>
    /// Initializes a new instance of the <see cref="CacheAside"/> class.
    /// </summary>
    /// <param name="cache">The cache operations instance.</param>
    /// <param name="defaultTtl">Optional default TTL for cached entries.</param>
    public CacheAside(ICacheOperations cache, TimeSpan? defaultTtl = null)
    {
        _cache = cache ?? throw new ArgumentNullException(nameof(cache));
        _defaultTtl = defaultTtl;
    }

    /// <summary>
    /// Gets the value from cache if available; otherwise loads it using the provided function and caches the result.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="loader">A function that loads the value when the cache does not contain it.</param>
    /// <param name="ttl">Optional TTL override for this specific call.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The cached or freshly loaded value.</returns>
    public async Task<string> GetOrLoadAsync(
        string key,
        Func<Task<string>> loader,
        TimeSpan? ttl = null,
        CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(loader);

        string? cached = await _cache.GetAsync(key, ct).ConfigureAwait(false);
        if (cached is not null)
        {
            return cached;
        }

        string value = await loader().ConfigureAwait(false);
        await _cache.SetAsync(key, value, ttl ?? _defaultTtl, ct).ConfigureAwait(false);
        return value;
    }
}
