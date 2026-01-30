using System.Collections.Concurrent;

namespace K1s0.Auth;

/// <summary>
/// In-memory implementation of <see cref="ITokenBlacklist"/> with automatic expiry cleanup.
/// </summary>
public class InMemoryBlacklist : ITokenBlacklist, IDisposable
{
    private readonly ConcurrentDictionary<string, DateTime> _entries = new();
    private readonly Timer _cleanupTimer;

    /// <summary>
    /// Initializes a new instance of the <see cref="InMemoryBlacklist"/> class.
    /// </summary>
    /// <param name="cleanupInterval">The interval between cleanup runs. Defaults to 5 minutes.</param>
    public InMemoryBlacklist(TimeSpan? cleanupInterval = null)
    {
        var interval = cleanupInterval ?? TimeSpan.FromMinutes(5);
        _cleanupTimer = new Timer(_ => Cleanup(), null, interval, interval);
    }

    /// <inheritdoc />
    public Task<bool> IsBlacklistedAsync(string jti, CancellationToken ct = default)
    {
        if (_entries.TryGetValue(jti, out var expiry))
        {
            if (DateTime.UtcNow < expiry)
            {
                return Task.FromResult(true);
            }

            _entries.TryRemove(jti, out _);
        }

        return Task.FromResult(false);
    }

    /// <inheritdoc />
    public Task AddAsync(string jti, DateTime expiry, CancellationToken ct = default)
    {
        _entries[jti] = expiry;
        return Task.CompletedTask;
    }

    private void Cleanup()
    {
        var now = DateTime.UtcNow;
        foreach (var (jti, expiry) in _entries)
        {
            if (now >= expiry)
            {
                _entries.TryRemove(jti, out _);
            }
        }
    }

    /// <summary>
    /// Disposes the cleanup timer.
    /// </summary>
    public void Dispose()
    {
        _cleanupTimer.Dispose();
        GC.SuppressFinalize(this);
    }
}
