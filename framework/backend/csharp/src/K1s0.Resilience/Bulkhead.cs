using System.Collections.Concurrent;

namespace K1s0.Resilience;

/// <summary>
/// Provides named bulkhead partitions for isolating concurrent workloads.
/// Each named partition maintains its own concurrency limit via a semaphore.
/// </summary>
public static class Bulkhead
{
    private static readonly ConcurrentDictionary<string, SemaphoreSlim> Semaphores = new();

    /// <summary>
    /// Executes the specified action within the named bulkhead partition.
    /// </summary>
    /// <typeparam name="T">The return type of the action.</typeparam>
    /// <param name="config">The bulkhead configuration specifying the partition name and concurrency limit.</param>
    /// <param name="action">The asynchronous action to execute.</param>
    /// <returns>The result of the action.</returns>
    /// <exception cref="ConcurrencyLimitException">Thrown when the bulkhead partition's concurrency limit has been reached.</exception>
    public static async Task<T> ExecuteAsync<T>(BulkheadConfig config, Func<Task<T>> action)
    {
        ArgumentNullException.ThrowIfNull(config);
        ArgumentNullException.ThrowIfNull(action);

        var semaphore = Semaphores.GetOrAdd(config.Name, _ => new SemaphoreSlim(config.MaxConcurrent, config.MaxConcurrent));

        if (!semaphore.Wait(0))
        {
            throw new ConcurrencyLimitException($"Bulkhead '{config.Name}' concurrency limit reached.");
        }

        try
        {
            return await action().ConfigureAwait(false);
        }
        finally
        {
            semaphore.Release();
        }
    }

    /// <summary>
    /// Removes all registered bulkhead partitions. Intended for testing purposes.
    /// </summary>
    internal static void Reset()
    {
        foreach (var kvp in Semaphores)
        {
            kvp.Value.Dispose();
        }

        Semaphores.Clear();
    }
}
