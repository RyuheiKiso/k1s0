namespace K1s0.System.AuditClient;

public class BufferedAuditClient : IAuditClient
{
    private readonly List<AuditEvent> _buffer = new();
    private readonly SemaphoreSlim _sem = new(1, 1);

    public async Task RecordAsync(AuditEvent @event, CancellationToken ct = default)
    {
        await _sem.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            _buffer.Add(@event);
        }
        finally
        {
            _sem.Release();
        }
    }

    public async Task<IReadOnlyList<AuditEvent>> FlushAsync(CancellationToken ct = default)
    {
        await _sem.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            var events = _buffer.ToList().AsReadOnly();
            _buffer.Clear();
            return events;
        }
        finally
        {
            _sem.Release();
        }
    }
}
