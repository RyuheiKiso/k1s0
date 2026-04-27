// 本ファイルは k1s0 .NET SDK の Telemetry 動詞統一 facade。
using K1s0.Sdk.Generated.K1s0.Tier1.Telemetry.V1;

namespace K1s0.Sdk;

public sealed class TelemetryFacade
{
    private readonly K1s0Client _client;
    internal TelemetryFacade(K1s0Client client) { _client = client; }

    public async Task EmitMetricAsync(IEnumerable<Metric> metrics, CancellationToken ct = default)
    {
        var req = new EmitMetricRequest { Context = _client.TenantContext() };
        foreach (var m in metrics) req.Metrics.Add(m);
        await _client.Raw.Telemetry.EmitMetricAsync(req, cancellationToken: ct);
    }

    public async Task EmitSpanAsync(IEnumerable<Span> spans, CancellationToken ct = default)
    {
        var req = new EmitSpanRequest { Context = _client.TenantContext() };
        foreach (var s in spans) req.Spans.Add(s);
        await _client.Raw.Telemetry.EmitSpanAsync(req, cancellationToken: ct);
    }
}
