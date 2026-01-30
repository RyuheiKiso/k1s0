namespace K1s0.Cache.Patterns;

/// <summary>
/// Implements the write-through pattern: writes go to the backing store first, then to the cache.
/// </summary>
public sealed class WriteThrough
{
    private readonly ICacheOperations _cache;
    private readonly TimeSpan? _defaultTtl;

    /// <summary>
    /// Initializes a new instance of the <see cref="WriteThrough"/> class.
    /// </summary>
    /// <param name="cache">The cache operations instance.</param>
    /// <param name="defaultTtl">Optional default TTL for cached entries.</param>
    public WriteThrough(ICacheOperations cache, TimeSpan? defaultTtl = null)
    {
        _cache = cache ?? throw new ArgumentNullException(nameof(cache));
        _defaultTtl = defaultTtl;
    }

    /// <summary>
    /// Writes the value to the backing store via the writer delegate, then stores it in the cache.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="value">The value to write.</param>
    /// <param name="writer">A delegate that persists the key-value pair to the backing store.</param>
    /// <param name="ttl">Optional TTL override.</param>
    /// <param name="ct">A cancellation token.</param>
    public async Task WriteAsync(
        string key,
        string value,
        Func<string, string, Task> writer,
        TimeSpan? ttl = null,
        CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(writer);

        // Write to backing store first.
        await writer(key, value).ConfigureAwait(false);

        // Then update the cache.
        await _cache.SetAsync(key, value, ttl ?? _defaultTtl, ct).ConfigureAwait(false);
    }
}
