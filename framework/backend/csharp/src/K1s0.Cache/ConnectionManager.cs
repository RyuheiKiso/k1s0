using StackExchange.Redis;

namespace K1s0.Cache;

/// <summary>
/// Manages the lifecycle of a <see cref="ConnectionMultiplexer"/> for Redis connections.
/// </summary>
public class ConnectionManager : IAsyncDisposable
{
    private readonly CacheConfig _config;
    private readonly SemaphoreSlim _lock = new(1, 1);
    private ConnectionMultiplexer? _connection;
    private bool _disposed;

    /// <summary>
    /// Initializes a new instance of the <see cref="ConnectionManager"/> class.
    /// </summary>
    /// <param name="config">The cache configuration.</param>
    public ConnectionManager(CacheConfig config)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
    }

    /// <summary>
    /// Gets the Redis <see cref="IDatabase"/> instance for executing commands.
    /// </summary>
    /// <returns>An <see cref="IDatabase"/> connected to the configured Redis server.</returns>
    /// <exception cref="CacheConnectionException">Thrown when the connection cannot be established.</exception>
    public virtual IDatabase GetDatabase()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);

        if (_connection is { IsConnected: true })
        {
            return _connection.GetDatabase(_config.Db);
        }

        _lock.Wait();
        try
        {
            if (_connection is { IsConnected: true })
            {
                return _connection.GetDatabase(_config.Db);
            }

            try
            {
                var options = new ConfigurationOptions
                {
                    EndPoints = { { _config.Host, _config.Port } },
                    DefaultDatabase = _config.Db,
                    ConnectTimeout = _config.ConnectTimeoutMs,
                    AbortOnConnectFail = false,
                };

                _connection = ConnectionMultiplexer.Connect(options);
            }
            catch (RedisConnectionException ex)
            {
                throw new CacheConnectionException(
                    $"Failed to connect to Redis at {_config.Host}:{_config.Port}", ex);
            }

            return _connection.GetDatabase(_config.Db);
        }
        finally
        {
            _lock.Release();
        }
    }

    /// <summary>
    /// Gets the underlying <see cref="ConnectionMultiplexer"/> if connected.
    /// </summary>
    /// <returns>The connection multiplexer, or <c>null</c> if not yet connected.</returns>
    internal ConnectionMultiplexer? GetMultiplexer() => _connection;

    /// <inheritdoc/>
    public async ValueTask DisposeAsync()
    {
        if (_disposed)
        {
            return;
        }

        _disposed = true;

        if (_connection is not null)
        {
            await _connection.CloseAsync().ConfigureAwait(false);
            _connection.Dispose();
        }

        _lock.Dispose();
    }
}
