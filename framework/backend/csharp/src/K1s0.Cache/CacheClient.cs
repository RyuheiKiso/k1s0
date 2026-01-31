using StackExchange.Redis;

namespace K1s0.Cache;

/// <summary>
/// Default implementation of all cache operation interfaces backed by StackExchange.Redis.
/// </summary>
public sealed class CacheClient : ICacheOperations, IHashOperations, IListOperations, ISetOperations
{
    private readonly ConnectionManager _connection;
    private readonly CacheConfig _config;

    /// <summary>
    /// Initializes a new instance of the <see cref="CacheClient"/> class.
    /// </summary>
    /// <param name="connection">The Redis connection manager.</param>
    /// <param name="config">The cache configuration.</param>
    public CacheClient(ConnectionManager connection, CacheConfig config)
    {
        _connection = connection ?? throw new ArgumentNullException(nameof(connection));
        _config = config ?? throw new ArgumentNullException(nameof(config));
    }

    /// <summary>
    /// Returns the key with the configured prefix applied.
    /// </summary>
    /// <param name="key">The original key.</param>
    /// <returns>The prefixed key.</returns>
    internal string PrefixedKey(string key) =>
        string.IsNullOrEmpty(_config.Prefix) ? key : $"{_config.Prefix}:{key}";

    private IDatabase Db => _connection.GetDatabase();

    private TimeSpan DefaultTtl => TimeSpan.FromSeconds(_config.DefaultTtlSeconds);

    // ICacheOperations

    /// <inheritdoc/>
    public async Task<string?> GetAsync(string key, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            RedisValue value = await Db.StringGetAsync(PrefixedKey(key)).ConfigureAwait(false);
            return value.IsNullOrEmpty ? null : value.ToString();
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to GET key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task SetAsync(string key, string value, TimeSpan? ttl = null, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            await Db.StringSetAsync(PrefixedKey(key), value, ttl ?? DefaultTtl).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to SET key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<bool> DeleteAsync(string key, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.KeyDeleteAsync(PrefixedKey(key)).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to DELETE key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<bool> ExistsAsync(string key, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.KeyExistsAsync(PrefixedKey(key)).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to check EXISTS for key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<long> IncrAsync(string key, long amount = 1, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.StringIncrementAsync(PrefixedKey(key), amount).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to INCR key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<long> DecrAsync(string key, long amount = 1, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.StringDecrementAsync(PrefixedKey(key), amount).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to DECR key '{key}'", ex);
        }
    }

    // IHashOperations

    /// <inheritdoc/>
    public async Task<string?> HGetAsync(string key, string field, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            RedisValue value = await Db.HashGetAsync(PrefixedKey(key), field).ConfigureAwait(false);
            return value.IsNullOrEmpty ? null : value.ToString();
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to HGET field '{field}' from key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task HSetAsync(string key, string field, string value, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            await Db.HashSetAsync(PrefixedKey(key), field, value).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to HSET field '{field}' on key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<bool> HDeleteAsync(string key, string field, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.HashDeleteAsync(PrefixedKey(key), field).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to HDEL field '{field}' from key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<Dictionary<string, string>> HGetAllAsync(string key, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            HashEntry[] entries = await Db.HashGetAllAsync(PrefixedKey(key)).ConfigureAwait(false);
            var result = new Dictionary<string, string>(entries.Length);
            foreach (var entry in entries)
            {
                result[entry.Name.ToString()] = entry.Value.ToString();
            }

            return result;
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to HGETALL for key '{key}'", ex);
        }
    }

    // IListOperations

    /// <inheritdoc/>
    public async Task<long> LPushAsync(string key, string value, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.ListLeftPushAsync(PrefixedKey(key), value).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to LPUSH on key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<long> RPushAsync(string key, string value, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.ListRightPushAsync(PrefixedKey(key), value).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to RPUSH on key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<string?> LPopAsync(string key, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            RedisValue value = await Db.ListLeftPopAsync(PrefixedKey(key)).ConfigureAwait(false);
            return value.IsNullOrEmpty ? null : value.ToString();
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to LPOP on key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<string?> RPopAsync(string key, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            RedisValue value = await Db.ListRightPopAsync(PrefixedKey(key)).ConfigureAwait(false);
            return value.IsNullOrEmpty ? null : value.ToString();
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to RPOP on key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<List<string>> LRangeAsync(string key, long start, long stop, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            RedisValue[] values = await Db.ListRangeAsync(PrefixedKey(key), start, stop).ConfigureAwait(false);
            var result = new List<string>(values.Length);
            foreach (var v in values)
            {
                result.Add(v.ToString());
            }

            return result;
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to LRANGE on key '{key}'", ex);
        }
    }

    // ISetOperations

    /// <inheritdoc/>
    public async Task<bool> SAddAsync(string key, string value, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.SetAddAsync(PrefixedKey(key), value).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to SADD on key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<bool> SRemoveAsync(string key, string value, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.SetRemoveAsync(PrefixedKey(key), value).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to SREM on key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<HashSet<string>> SMembersAsync(string key, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            RedisValue[] members = await Db.SetMembersAsync(PrefixedKey(key)).ConfigureAwait(false);
            var result = new HashSet<string>(members.Length);
            foreach (var m in members)
            {
                result.Add(m.ToString());
            }

            return result;
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to SMEMBERS on key '{key}'", ex);
        }
    }

    /// <inheritdoc/>
    public async Task<bool> SIsMemberAsync(string key, string value, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            return await Db.SetContainsAsync(PrefixedKey(key), value).ConfigureAwait(false);
        }
        catch (RedisException ex)
        {
            throw new CacheOperationException($"Failed to SISMEMBER on key '{key}'", ex);
        }
    }
}
