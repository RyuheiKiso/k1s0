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
}
