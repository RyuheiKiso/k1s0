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

    public async Task<IReadOnlyList<AuditEvent>> QueryAsync(AuditFilter filter, CancellationToken ct = default)
    {
        await _sem.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            IEnumerable<AuditEvent> result = _buffer;

            if (filter.TenantId is not null)
            {
                result = result.Where(e => e.TenantId == filter.TenantId);
            }

            if (filter.ActorId is not null)
            {
                result = result.Where(e => e.ActorId == filter.ActorId);
            }

            if (filter.Action is not null)
            {
                result = result.Where(e => e.Action == filter.Action);
            }

            if (filter.ResourceType is not null)
            {
                result = result.Where(e => e.ResourceType == filter.ResourceType);
            }

            if (filter.From.HasValue)
            {
                result = result.Where(e => e.Timestamp >= filter.From.Value);
            }

            if (filter.To.HasValue)
            {
                result = result.Where(e => e.Timestamp <= filter.To.Value);
            }

            return result.ToList().AsReadOnly();
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
