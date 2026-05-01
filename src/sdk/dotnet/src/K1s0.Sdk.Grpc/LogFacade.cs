// 本ファイルは k1s0 .NET SDK の Log 動詞統一 facade。
using Google.Protobuf.WellKnownTypes;
using K1s0.Sdk.Generated.K1s0.Tier1.Log.V1;

namespace K1s0.Sdk;

public sealed class LogFacade
{
    private readonly K1s0Client _client;
    internal LogFacade(K1s0Client client) { _client = client; }

    /// SendAsync: 単一エントリ送信（generic）。
    public async Task SendAsync(Severity severity, string body, IDictionary<string, string>? attributes = null, CancellationToken ct = default)
    {
        var entry = new LogEntry { Timestamp = Timestamp.FromDateTime(DateTime.UtcNow), Severity = severity, Body = body };
        if (attributes != null) foreach (var kv in attributes) entry.Attributes.Add(kv.Key, kv.Value);
        await _client.Raw.Log.SendAsync(new SendLogRequest { Entry = entry, Context = _client.TenantContext() }, cancellationToken: ct);
    }

    public Task InfoAsync(string body, IDictionary<string, string>? attrs = null, CancellationToken ct = default) => SendAsync(Severity.Info, body, attrs, ct);
    public Task WarnAsync(string body, IDictionary<string, string>? attrs = null, CancellationToken ct = default) => SendAsync(Severity.Warn, body, attrs, ct);
    public Task ErrorAsync(string body, IDictionary<string, string>? attrs = null, CancellationToken ct = default) => SendAsync(Severity.Error, body, attrs, ct);
    public Task DebugAsync(string body, IDictionary<string, string>? attrs = null, CancellationToken ct = default) => SendAsync(Severity.Debug, body, attrs, ct);

    /// BulkSendAsync: LogEntry の一括送信（FR-T1-LOG-* 共通、Send の高スループット版）。
    /// 各 entry の Timestamp が null なら呼出時刻を自動設定する。
    /// 戻り値は (Accepted, Rejected) の tuple（rejected は PII / schema 違反による却下件数）。
    public async Task<(int Accepted, int Rejected)> BulkSendAsync(
        IEnumerable<LogEntryInput> entries,
        CancellationToken ct = default)
    {
        var req = new BulkSendLogRequest { Context = _client.TenantContext() };
        var nowTs = Timestamp.FromDateTime(DateTime.UtcNow);
        foreach (var e in entries)
        {
            var pe = new LogEntry
            {
                Timestamp = e.Timestamp.HasValue
                    ? Timestamp.FromDateTime(e.Timestamp.Value.ToUniversalTime())
                    : nowTs,
                Severity = e.Severity,
                Body = e.Body,
            };
            if (e.Attributes != null)
            {
                foreach (var kv in e.Attributes)
                {
                    pe.Attributes.Add(kv.Key, kv.Value);
                }
            }
            req.Entries.Add(pe);
        }
        var resp = await _client.Raw.Log.BulkSendAsync(req, cancellationToken: ct);
        return (resp.Accepted, resp.Rejected);
    }
}

/// BulkSendAsync の 1 件分の入力。
public sealed class LogEntryInput
{
    /// 重大度（OTel SeverityNumber）。
    public Severity Severity { get; init; }
    /// 発生時刻。null なら呼出時刻が自動設定される。
    public DateTime? Timestamp { get; init; }
    /// 本文。
    public string Body { get; init; } = string.Empty;
    /// 構造化属性（OTel attributes）。
    public IDictionary<string, string>? Attributes { get; init; }
}
