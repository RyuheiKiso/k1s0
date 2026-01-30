namespace K1s0.Cache.Patterns;

/// <summary>
/// Implements the TTL-refresh pattern: each read extends the expiration of the cached entry.
/// </summary>
public sealed class TtlRefresh
{
    private readonly ICacheOperations _cache;
    private readonly TimeSpan _refreshTtl;

    /// <summary>
    /// Initializes a new instance of the <see cref="TtlRefresh"/> class.
    /// </summary>
    /// <param name="cache">The cache operations instance.</param>
    /// <param name="refreshTtl">The TTL to set on each read.</param>
    public TtlRefresh(ICacheOperations cache, TimeSpan refreshTtl)
    {
        _cache = cache ?? throw new ArgumentNullException(nameof(cache));
        _refreshTtl = refreshTtl;
    }

    /// <summary>
    /// Gets the value for the specified key and refreshes its TTL if it exists.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The cached value, or <c>null</c> if the key does not exist.</returns>
    public async Task<string?> GetAndRefreshAsync(string key, CancellationToken ct = default)
    {
        string? value = await _cache.GetAsync(key, ct).ConfigureAwait(false);
        if (value is not null)
        {
            // Re-set the value with the refresh TTL to extend expiration.
            await _cache.SetAsync(key, value, _refreshTtl, ct).ConfigureAwait(false);
        }

        return value;
    }
}
