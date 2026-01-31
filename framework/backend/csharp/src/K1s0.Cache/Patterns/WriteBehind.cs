using System.Collections.Concurrent;

namespace K1s0.Cache.Patterns;

/// <summary>
/// Implements the write-behind (write-back) pattern: writes are buffered in memory
/// and flushed to the backing store in the background.
/// </summary>
public sealed class WriteBehind : IAsyncDisposable
{
    private readonly ICacheOperations _cache;
    private readonly Func<IReadOnlyList<KeyValuePair<string, string>>, Task> _flushHandler;
    private readonly TimeSpan? _defaultTtl;
    private readonly ConcurrentQueue<KeyValuePair<string, string>> _buffer = new();
    private readonly TimeSpan _flushInterval;
    private CancellationTokenSource? _cts;
    private Task? _backgroundTask;

    private long _totalWrites;
    private long _totalFlushes;
    private long _totalFailures;

    /// <summary>
    /// Initializes a new instance of the <see cref="WriteBehind"/> class.
    /// </summary>
    /// <param name="cache">The cache operations instance.</param>
    /// <param name="flushHandler">A delegate that persists a batch of key-value pairs to the backing store.</param>
    /// <param name="flushInterval">The interval between background flush cycles.</param>
    /// <param name="defaultTtl">Optional default TTL for cached entries.</param>
    public WriteBehind(
        ICacheOperations cache,
        Func<IReadOnlyList<KeyValuePair<string, string>>, Task> flushHandler,
        TimeSpan flushInterval,
        TimeSpan? defaultTtl = null)
    {
        _cache = cache ?? throw new ArgumentNullException(nameof(cache));
        _flushHandler = flushHandler ?? throw new ArgumentNullException(nameof(flushHandler));
        _flushInterval = flushInterval;
        _defaultTtl = defaultTtl;
    }

    /// <summary>
    /// Gets the total number of write operations enqueued.
    /// </summary>
    public long TotalWrites => Interlocked.Read(ref _totalWrites);

    /// <summary>
    /// Gets the total number of successful flush operations.
    /// </summary>
    public long TotalFlushes => Interlocked.Read(ref _totalFlushes);

    /// <summary>
    /// Gets the total number of failed flush operations.
    /// </summary>
    public long TotalFailures => Interlocked.Read(ref _totalFailures);

    /// <summary>
    /// Writes a value to the cache immediately and enqueues it for background persistence.
    /// </summary>
    /// <param name="key">The cache key.</param>
    /// <param name="value">The value to write.</param>
    /// <param name="ttl">Optional TTL override.</param>
    /// <param name="ct">A cancellation token.</param>
    public async Task WriteAsync(string key, string value, TimeSpan? ttl = null, CancellationToken ct = default)
    {
        // Write to cache immediately.
        await _cache.SetAsync(key, value, ttl ?? _defaultTtl, ct).ConfigureAwait(false);

        // Buffer for background flush.
        _buffer.Enqueue(new KeyValuePair<string, string>(key, value));
        Interlocked.Increment(ref _totalWrites);
    }

    /// <summary>
    /// Flushes all buffered writes to the backing store immediately.
    /// </summary>
    public async Task FlushAsync()
    {
        var batch = new List<KeyValuePair<string, string>>();
        while (_buffer.TryDequeue(out var item))
        {
            batch.Add(item);
        }

        if (batch.Count == 0)
        {
            return;
        }

        try
        {
            await _flushHandler(batch).ConfigureAwait(false);
            Interlocked.Increment(ref _totalFlushes);
        }
        catch
        {
            Interlocked.Increment(ref _totalFailures);

            // Re-enqueue items so they are not lost.
            foreach (var item in batch)
            {
                _buffer.Enqueue(item);
            }

            throw;
        }
    }

    /// <summary>
    /// Starts the background flush loop.
    /// </summary>
    public void StartAsync()
    {
        if (_cts is not null)
        {
            return;
        }

        _cts = new CancellationTokenSource();
        _backgroundTask = RunFlushLoopAsync(_cts.Token);
    }

    /// <summary>
    /// Stops the background flush loop and performs a final flush.
    /// </summary>
    public async Task StopAsync()
    {
        if (_cts is null)
        {
            return;
        }

        await _cts.CancelAsync().ConfigureAwait(false);

        if (_backgroundTask is not null)
        {
            try
            {
                await _backgroundTask.ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                // Expected on cancellation.
            }
        }

        // Final flush of remaining items.
        await FlushAsync().ConfigureAwait(false);

        _cts.Dispose();
        _cts = null;
        _backgroundTask = null;
    }

    /// <inheritdoc/>
    public async ValueTask DisposeAsync()
    {
        await StopAsync().ConfigureAwait(false);
    }

    private async Task RunFlushLoopAsync(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            try
            {
                await Task.Delay(_flushInterval, ct).ConfigureAwait(false);
                await FlushAsync().ConfigureAwait(false);
            }
            catch (OperationCanceledException) when (ct.IsCancellationRequested)
            {
                break;
            }
            catch
            {
                // Errors are tracked via TotalFailures; continue the loop.
            }
        }
    }
}
