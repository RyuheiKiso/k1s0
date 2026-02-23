namespace K1s0.System.Resiliency;

internal sealed class Bulkhead : IDisposable
{
    private readonly SemaphoreSlim _semaphore;
    private readonly int _maxConcurrent;
    private readonly TimeSpan _maxWait;

    public Bulkhead(int maxConcurrent, TimeSpan maxWait)
    {
        _maxConcurrent = maxConcurrent;
        _maxWait = maxWait;
        _semaphore = new SemaphoreSlim(maxConcurrent, maxConcurrent);
    }

    public async Task AcquireAsync(CancellationToken ct)
    {
        var acquired = await _semaphore.WaitAsync(_maxWait, ct);
        if (!acquired)
        {
            throw new ResiliencyException(
                $"Bulkhead full, max concurrent calls: {_maxConcurrent}",
                ResiliencyErrorKind.BulkheadFull);
        }
    }

    public void Release() => _semaphore.Release();

    public void Dispose() => _semaphore.Dispose();
}
