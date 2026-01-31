namespace K1s0.Cache;

/// <summary>
/// Performs health checks against the Redis server by issuing a PING command.
/// </summary>
public sealed class CacheHealthChecker
{
    private readonly ConnectionManager _connection;

    /// <summary>
    /// Initializes a new instance of the <see cref="CacheHealthChecker"/> class.
    /// </summary>
    /// <param name="connection">The Redis connection manager.</param>
    public CacheHealthChecker(ConnectionManager connection)
    {
        _connection = connection ?? throw new ArgumentNullException(nameof(connection));
    }

    /// <summary>
    /// Checks whether the Redis server is reachable by executing a PING command.
    /// </summary>
    /// <param name="ct">A cancellation token.</param>
    /// <returns><c>true</c> if the server responded to PING; otherwise, <c>false</c>.</returns>
    public async Task<bool> CheckAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        try
        {
            var db = _connection.GetDatabase();
            var ping = await db.PingAsync().ConfigureAwait(false);
            return ping.TotalMilliseconds >= 0;
        }
        catch
        {
            return false;
        }
    }
}
