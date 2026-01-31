using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Npgsql;

namespace K1s0.Consensus;

/// <summary>
/// PostgreSQL-backed distributed lock implementation.
/// The table must be created beforehand with columns:
/// <c>lock_key TEXT PRIMARY KEY, holder_id TEXT NOT NULL, expires_at TIMESTAMPTZ NOT NULL</c>.
/// </summary>
public sealed class DbDistributedLock : IDistributedLock
{
    private readonly LockConfig _config;
    private readonly ILogger<DbDistributedLock> _logger;
    private readonly string _connectionString;

    /// <summary>
    /// Creates a new <see cref="DbDistributedLock"/>.
    /// </summary>
    /// <param name="options">Consensus configuration.</param>
    /// <param name="logger">Logger instance.</param>
    public DbDistributedLock(IOptions<ConsensusConfig> options, ILogger<DbDistributedLock> logger)
    {
        _config = options.Value.Lock;
        _logger = logger;
        _connectionString = ReadConnectionString(_config.ConnectionStringFile);
    }

    /// <inheritdoc />
    public async Task<LockGuard?> TryLockAsync(string lockKey, string holderId, TimeSpan? expiration = null, CancellationToken cancellationToken = default)
    {
        var exp = expiration ?? _config.DefaultExpiration;
        var expiresAt = DateTimeOffset.UtcNow.Add(exp);

        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $@"
            INSERT INTO {_config.TableName} (lock_key, holder_id, expires_at)
            VALUES (@key, @holder, @expires)
            ON CONFLICT (lock_key) DO UPDATE
            SET holder_id = @holder, expires_at = @expires
            WHERE {_config.TableName}.expires_at < NOW()
               OR {_config.TableName}.holder_id = @holder
            RETURNING lock_key";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("key", lockKey);
        cmd.Parameters.AddWithValue("holder", holderId);
        cmd.Parameters.AddWithValue("expires", expiresAt);

        var result = await cmd.ExecuteScalarAsync(cancellationToken).ConfigureAwait(false);
        if (result is null or DBNull)
        {
            _logger.LogDebug("Failed to acquire lock {LockKey}", lockKey);
            return null;
        }

        _logger.LogDebug("Acquired lock {LockKey}", lockKey);
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
        var expiresAt = DateTimeOffset.UtcNow.Add(ext);

        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $"UPDATE {_config.TableName} SET expires_at = @expires WHERE lock_key = @key AND holder_id = @holder AND expires_at > NOW()";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("key", lockKey);
        cmd.Parameters.AddWithValue("holder", holderId);
        cmd.Parameters.AddWithValue("expires", expiresAt);

        var rows = await cmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);
        return rows > 0;
    }

    /// <inheritdoc />
    public async Task<bool> UnlockAsync(string lockKey, string holderId, CancellationToken cancellationToken = default)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $"DELETE FROM {_config.TableName} WHERE lock_key = @key AND holder_id = @holder";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("key", lockKey);
        cmd.Parameters.AddWithValue("holder", holderId);

        var rows = await cmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);
        if (rows > 0)
        {
            _logger.LogDebug("Released lock {LockKey}", lockKey);
            Metrics.LockMetrics.ReleasedTotal.Inc();
        }

        return rows > 0;
    }

    private static string ReadConnectionString(string filePath)
    {
        if (string.IsNullOrWhiteSpace(filePath))
        {
            throw new ConsensusException(
                "consensus.config.missing_connection_string",
                "Lock connection_string_file is not configured.");
        }

        return File.ReadAllText(filePath).Trim();
    }
}
