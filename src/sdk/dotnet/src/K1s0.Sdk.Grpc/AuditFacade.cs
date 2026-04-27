// 本ファイルは k1s0 .NET SDK の Audit 動詞統一 facade。
using Google.Protobuf.WellKnownTypes;
using K1s0.Sdk.Generated.K1s0.Tier1.Audit.V1;

namespace K1s0.Sdk;

public sealed class AuditFacade
{
    private readonly K1s0Client _client;
    internal AuditFacade(K1s0Client client) { _client = client; }

    /// RecordAsync: 監査イベント記録。auditId を返す。
    public async Task<string> RecordAsync(
        string actor, string action, string resource, string outcome,
        IDictionary<string, string>? attributes = null, CancellationToken ct = default)
    {
        var ev = new AuditEvent
        {
            Timestamp = Timestamp.FromDateTime(DateTime.UtcNow),
            Actor = actor,
            Action = action,
            Resource = resource,
            Outcome = outcome,
        };
        if (attributes != null) foreach (var kv in attributes) ev.Attributes.Add(kv.Key, kv.Value);
        var resp = await _client.Raw.Audit.RecordAsync(new RecordAuditRequest { Event = ev, Context = _client.TenantContext() }, cancellationToken: ct);
        return resp.AuditId;
    }

    /// QueryAsync: 監査イベント検索。
    public async Task<IReadOnlyList<AuditEvent>> QueryAsync(
        DateTime from, DateTime to, IDictionary<string, string>? filters = null, int limit = 100, CancellationToken ct = default)
    {
        var req = new QueryAuditRequest
        {
            From = Timestamp.FromDateTime(from.ToUniversalTime()),
            To = Timestamp.FromDateTime(to.ToUniversalTime()),
            Limit = limit,
            Context = _client.TenantContext(),
        };
        if (filters != null) foreach (var kv in filters) req.Filters.Add(kv.Key, kv.Value);
        var resp = await _client.Raw.Audit.QueryAsync(req, cancellationToken: ct);
        return resp.Events.ToList();
    }
}
