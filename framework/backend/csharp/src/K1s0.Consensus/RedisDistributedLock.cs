using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using StackExchange.Redis;

namespace K1s0.Consensus;

/// <summary>
/// Redis-backed distributed lock implementation using Lua scripts for atomicity.
/// </summary>
public sealed class RedisDistributedLock : IDistributedLock
{
    private readonly LockConfig _config;
    private readonly ILogger<RedisDistributedLock> _logger;
    private readonly IConnectionMultiplexer _redis;

    private static readonly LuaScript AcquireScript = LuaScript.Prepare(
        """
        if redis.call('EXISTS', @key) == 0 then
            redis.call('SET', @key, @holder, 'PX', @ttl)
            return 1
        elseif redis.call('GET', @key) == @holder then
            redis.call('PEXPIRE', @key, @ttl)
            return 1
        else
            return 0
        end
        """);

    private static readonly LuaScript ReleaseScript = LuaScript.Prepare(
        """
        if redis.call('GET', @key) == @holder then
            return redis.call('DEL', @key)
        else
            return 0
        end
        """);

    private static readonly LuaScript ExtendScript = LuaScript.Prepare(
        """
        if redis.call('GET', @key) == @holder then
            return redis.call('PEXPIRE', @key, @ttl)
        else
            return 0
        end
        """);

    /// <summary>
    /// Creates a new <see cref="RedisDistributedLock"/>.
    /// </summary>
    /// <param name="options">Consensus configuration.</param>
    /// <param name="redis">Redis connection multiplexer.</param>
    /// <param name="logger">Logger instance.</param>
    public RedisDistributedLock(
        IOptions<ConsensusConfig> options,
        IConnectionMultiplexer redis,
        ILogger<RedisDistributedLock> logger)
    {
        _config = options.Value.Lock;
        _redis = redis;
        _logger = logger;
    }

    /// <inheritdoc />
    public async Task<LockGuard?> TryLockAsync(string lockKey, string holderId, TimeSpan? expiration = null, CancellationToken cancellationToken = default)
    {
        var exp = expiration ?? _config.DefaultExpiration;
        var db = _redis.GetDatabase();

        var result = (int)await db.ScriptEvaluateAsync(
            AcquireScript,
            new { key = (RedisKey)lockKey, holder = (RedisValue)holderId, ttl = (long)exp.TotalMilliseconds }).ConfigureAwait(false);

        if (result == 0)
        {
            _logger.LogDebug("Failed to acquire Redis lock {LockKey}", lockKey);
            return null;
        }

        _logger.LogDebug("Acquired Redis lock {LockKey}", lockKey);
        Metrics.LockMetrics.AcquiredTotal.Inc();
        return new LockGuard(this, lockKey, holderId);
    }

    /// <inheritdoc />
    public async Task<LockGuard> LockAsync(string lockKey, string holderId, TimeSpan? expiration = null, TimeSpan? waitTimeout = null, CancellationToken cancellationToken = default)
    {
        var timeout = waitTimeout ?? _config.DefaultWaitTimeout;
        using var cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        cts.CancelAfter(timeout);

        while (!cts.Token.IsCancellationRequested)
        {
            var guard = await TryLockAsync(lockKey, holderId, expiration, cts.Token).ConfigureAwait(false);
            if (guard is not null)
            {
                return guard;
            }

            await Task.Delay(_config.RetryInterval, cts.Token).ConfigureAwait(false);
        }

        Metrics.LockMetrics.TimeoutsTotal.Inc();
        throw new LockAcquisitionException(lockKey);
    }

    /// <inheritdoc />
    public async Task<bool> ExtendAsync(string lockKey, string holderId, TimeSpan? extension = null, CancellationToken cancellationToken = default)
    {
        var ext = extension ?? _config.DefaultExpiration;
        var db = _redis.GetDatabase();

        var result = (int)await db.ScriptEvaluateAsync(
            ExtendScript,
            new { key = (RedisKey)lockKey, holder = (RedisValue)holderId, ttl = (long)ext.TotalMilliseconds }).ConfigureAwait(false);

        return result == 1;
    }

    /// <inheritdoc />
    public async Task<bool> UnlockAsync(string lockKey, string holderId, CancellationToken cancellationToken = default)
    {
        var db = _redis.GetDatabase();

        var result = (int)await db.ScriptEvaluateAsync(
            ReleaseScript,
            new { key = (RedisKey)lockKey, holder = (RedisValue)holderId }).ConfigureAwait(false);

        if (result > 0)
        {
            _logger.LogDebug("Released Redis lock {LockKey}", lockKey);
            Metrics.LockMetrics.ReleasedTotal.Inc();
        }

        return result > 0;
    }
}
