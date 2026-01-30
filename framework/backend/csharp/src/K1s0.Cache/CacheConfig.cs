namespace K1s0.Cache;

/// <summary>
/// Configuration settings for the Redis cache connection and behavior.
/// </summary>
/// <param name="Host">The Redis server hostname. Defaults to <c>"localhost"</c>.</param>
/// <param name="Port">The Redis server port. Defaults to <c>6379</c>.</param>
/// <param name="Db">The Redis database index. Defaults to <c>0</c>.</param>
/// <param name="Prefix">A key prefix applied to all cache operations. Defaults to an empty string.</param>
/// <param name="PoolSize">The connection pool size. Defaults to <c>10</c>.</param>
/// <param name="DefaultTtlSeconds">The default time-to-live for cache entries in seconds. Defaults to <c>3600</c>.</param>
/// <param name="ConnectTimeoutMs">The connection timeout in milliseconds. Defaults to <c>5000</c>.</param>
public record CacheConfig(
    string Host = "localhost",
    int Port = 6379,
    int Db = 0,
    string Prefix = "",
    int PoolSize = 10,
    int DefaultTtlSeconds = 3600,
    int ConnectTimeoutMs = 5000);
