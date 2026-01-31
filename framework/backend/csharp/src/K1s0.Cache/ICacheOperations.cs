namespace K1s0.Cache;

/// <summary>
/// Provides basic key-value cache operations.
/// </summary>
public interface ICacheOperations
{
    /// <summary>
    /// Retrieves the value associated with the specified key.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The cached value, or <c>null</c> if the key does not exist.</returns>
    Task<string?> GetAsync(string key, CancellationToken ct = default);

    /// <summary>
    /// Sets the value for the specified key with an optional time-to-live.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="value">The value to store.</param>
    /// <param name="ttl">Optional time-to-live. If <c>null</c>, the configured default TTL is used.</param>
    /// <param name="ct">A cancellation token.</param>
    Task SetAsync(string key, string value, TimeSpan? ttl = null, CancellationToken ct = default);

    /// <summary>
    /// Deletes the specified key from the cache.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns><c>true</c> if the key was removed; otherwise, <c>false</c>.</returns>
    Task<bool> DeleteAsync(string key, CancellationToken ct = default);

    /// <summary>
    /// Checks whether the specified key exists in the cache.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns><c>true</c> if the key exists; otherwise, <c>false</c>.</returns>
    Task<bool> ExistsAsync(string key, CancellationToken ct = default);

    /// <summary>
    /// Increments the integer value of a key by the specified amount.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="amount">The amount to increment by. Defaults to <c>1</c>.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The value after the increment.</returns>
    Task<long> IncrAsync(string key, long amount = 1, CancellationToken ct = default);

    /// <summary>
    /// Decrements the integer value of a key by the specified amount.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="amount">The amount to decrement by. Defaults to <c>1</c>.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The value after the decrement.</returns>
    Task<long> DecrAsync(string key, long amount = 1, CancellationToken ct = default);
}
