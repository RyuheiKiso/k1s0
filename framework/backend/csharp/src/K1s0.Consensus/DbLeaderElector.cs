using System.Runtime.CompilerServices;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Npgsql;

namespace K1s0.Consensus;

/// <summary>
/// PostgreSQL-backed leader election implementation using advisory locks and lease rows.
/// The table must be created beforehand with columns:
/// <c>lease_key TEXT PRIMARY KEY, holder_id TEXT NOT NULL, fence_token BIGINT NOT NULL DEFAULT 0, expires_at TIMESTAMPTZ NOT NULL</c>.
/// </summary>
public sealed class DbLeaderElector : ILeaderElector
{
    private readonly LeaderConfig _config;
    private readonly ILogger<DbLeaderElector> _logger;
    private readonly string _connectionString;

    /// <summary>
    /// Creates a new <see cref="DbLeaderElector"/>.
    /// </summary>
    /// <param name="options">Leader election configuration.</param>
    /// <param name="logger">Logger instance.</param>
    public DbLeaderElector(IOptions<ConsensusConfig> options, ILogger<DbLeaderElector> logger)
    {
        _config = options.Value.Leader;
        _logger = logger;
        _connectionString = ReadConnectionString(_config.ConnectionStringFile);
    }

    /// <inheritdoc />
    public async Task<LeaderLease?> TryAcquireAsync(string leaseKey, string holderId, CancellationToken cancellationToken = default)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var expiresAt = DateTimeOffset.UtcNow.Add(_config.LeaseDuration);
        var table = _config.TableName;

        // Upsert: insert if no row, or update if expired
        var sql = $@"
            INSERT INTO {table} (lease_key, holder_id, fence_token, expires_at)
            VALUES (@key, @holder, 1, @expires)
            ON CONFLICT (lease_key) DO UPDATE
            SET holder_id = @holder,
                fence_token = {table}.fence_token + 1,
                expires_at = @expires
            WHERE {table}.expires_at < NOW()
               OR {table}.holder_id = @holder
            RETURNING fence_token";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("key", leaseKey);
        cmd.Parameters.AddWithValue("holder", holderId);
        cmd.Parameters.AddWithValue("expires", expiresAt);

        var result = await cmd.ExecuteScalarAsync(cancellationToken).ConfigureAwait(false);
        if (result is null or DBNull)
        {
            _logger.LogDebug("Failed to acquire lease {LeaseKey}: held by another node", leaseKey);
            return null;
        }

        var fenceToken = Convert.ToUInt64(result);
        _logger.LogInformation("Acquired lease {LeaseKey} with fence token {FenceToken}", leaseKey, fenceToken);
        Metrics.LeaderMetrics.ElectionsTotal.Inc();
        return new LeaderLease(leaseKey, holderId, fenceToken, expiresAt);
    }

    /// <inheritdoc />
    public async Task<LeaderLease> RenewAsync(LeaderLease lease, CancellationToken cancellationToken = default)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var expiresAt = DateTimeOffset.UtcNow.Add(_config.LeaseDuration);
        var table = _config.TableName;

        var sql = $@"
            UPDATE {table}
            SET fence_token = fence_token + 1,
                expires_at = @expires
            WHERE lease_key = @key
              AND holder_id = @holder
              AND expires_at > NOW()
            RETURNING fence_token";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("key", lease.LeaseKey);
        cmd.Parameters.AddWithValue("holder", lease.HolderId);
        cmd.Parameters.AddWithValue("expires", expiresAt);

        var result = await cmd.ExecuteScalarAsync(cancellationToken).ConfigureAwait(false);
        if (result is null or DBNull)
        {
            _logger.LogWarning("Failed to renew lease {LeaseKey}: lease expired or not held", lease.LeaseKey);
            Metrics.LeaderMetrics.LeaderLostTotal.Inc();
            throw new LeaseExpiredException(lease.LeaseKey);
        }

        var fenceToken = Convert.ToUInt64(result);
        _logger.LogDebug("Renewed lease {LeaseKey} with fence token {FenceToken}", lease.LeaseKey, fenceToken);
        Metrics.LeaderMetrics.RenewalsTotal.Inc();
        return lease with { FenceToken = fenceToken, ExpiresAt = expiresAt };
    }

    /// <inheritdoc />
    public async Task ReleaseAsync(LeaderLease lease, CancellationToken cancellationToken = default)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $"DELETE FROM {_config.TableName} WHERE lease_key = @key AND holder_id = @holder";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("key", lease.LeaseKey);
        cmd.Parameters.AddWithValue("holder", lease.HolderId);

        await cmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);
        _logger.LogInformation("Released lease {LeaseKey}", lease.LeaseKey);
    }

    /// <inheritdoc />
    public async Task<LeaderLease?> CurrentLeaderAsync(string leaseKey, CancellationToken cancellationToken = default)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $"SELECT holder_id, fence_token, expires_at FROM {_config.TableName} WHERE lease_key = @key AND expires_at > NOW()";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("key", leaseKey);

        await using var reader = await cmd.ExecuteReaderAsync(cancellationToken).ConfigureAwait(false);
        if (!await reader.ReadAsync(cancellationToken).ConfigureAwait(false))
        {
            return null;
        }

        return new LeaderLease(
            leaseKey,
            reader.GetString(0),
            (ulong)reader.GetInt64(1),
            reader.GetFieldValue<DateTimeOffset>(2));
    }

    /// <inheritdoc />
    public async IAsyncEnumerable<LeaderEvent> WatchAsync(
        string leaseKey,
        string holderId,
        [EnumeratorCancellation] CancellationToken cancellationToken = default)
    {
        LeaderLease? currentLease = null;

        // Try initial acquisition
        currentLease = await TryAcquireAsync(leaseKey, holderId, cancellationToken).ConfigureAwait(false);
        if (currentLease is not null)
        {
            yield return LeaderEvent.Create(LeaderEventType.Elected, currentLease);
        }

        using var timer = new PeriodicTimer(_config.RenewInterval);

        while (await timer.WaitForNextTickAsync(cancellationToken).ConfigureAwait(false))
        {
            if (currentLease is not null)
            {
                // We are leader, try to renew
                LeaderEvent? renewEvent = null;
                try
                {
                    currentLease = await RenewAsync(currentLease, cancellationToken).ConfigureAwait(false);
                    renewEvent = LeaderEvent.Create(LeaderEventType.Renewed, currentLease);
                }
                catch (LeaseExpiredException)
                {
                    currentLease = null;
                    renewEvent = LeaderEvent.Create(LeaderEventType.Lost);
                }

                if (renewEvent is not null)
                {
                    yield return renewEvent;
                }
            }
            else
            {
                // We are not leader, try to acquire
                currentLease = await TryAcquireAsync(leaseKey, holderId, cancellationToken).ConfigureAwait(false);
                if (currentLease is not null)
                {
                    yield return LeaderEvent.Create(LeaderEventType.Elected, currentLease);
                }
                else
                {
                    // Check who the current leader is
                    var leader = await CurrentLeaderAsync(leaseKey, cancellationToken).ConfigureAwait(false);
                    if (leader is not null)
                    {
                        yield return LeaderEvent.Create(LeaderEventType.Changed, leader);
                    }
                }
            }
        }
    }

    private static string ReadConnectionString(string filePath)
    {
        if (string.IsNullOrWhiteSpace(filePath))
        {
            throw new ConsensusException(
                "consensus.config.missing_connection_string",
                "Leader election connection_string_file is not configured.");
        }

        return File.ReadAllText(filePath).Trim();
    }
}
