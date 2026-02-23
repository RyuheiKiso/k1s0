namespace K1s0.System.AuditClient;

public interface IAuditClient
{
    Task RecordAsync(AuditEvent @event, CancellationToken ct = default);

    Task<IReadOnlyList<AuditEvent>> FlushAsync(CancellationToken ct = default);
}
