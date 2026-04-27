// 本ファイルは k1s0 .NET SDK の Feature 動詞統一 facade（評価部のみ）。
using K1s0.Sdk.Generated.K1s0.Tier1.Feature.V1;

namespace K1s0.Sdk;

public sealed class FeatureFacade
{
    private readonly K1s0Client _client;
    internal FeatureFacade(K1s0Client client) { _client = client; }

    private EvaluateRequest MakeReq(string flagKey, IDictionary<string, string>? evalCtx)
    {
        var req = new EvaluateRequest { FlagKey = flagKey, Context = _client.TenantContext() };
        if (evalCtx != null) foreach (var kv in evalCtx) req.EvaluationContext.Add(kv.Key, kv.Value);
        return req;
    }

    public async Task<(bool Value, FlagMetadata? Metadata)> EvaluateBooleanAsync(string flagKey, IDictionary<string, string>? evalCtx = null, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Feature.EvaluateBooleanAsync(MakeReq(flagKey, evalCtx), cancellationToken: ct);
        return (resp.Value, resp.Metadata);
    }

    public async Task<(string Value, FlagMetadata? Metadata)> EvaluateStringAsync(string flagKey, IDictionary<string, string>? evalCtx = null, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Feature.EvaluateStringAsync(MakeReq(flagKey, evalCtx), cancellationToken: ct);
        return (resp.Value, resp.Metadata);
    }

    public async Task<(double Value, FlagMetadata? Metadata)> EvaluateNumberAsync(string flagKey, IDictionary<string, string>? evalCtx = null, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Feature.EvaluateNumberAsync(MakeReq(flagKey, evalCtx), cancellationToken: ct);
        return (resp.Value, resp.Metadata);
    }

    public async Task<(byte[] ValueJson, FlagMetadata? Metadata)> EvaluateObjectAsync(string flagKey, IDictionary<string, string>? evalCtx = null, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Feature.EvaluateObjectAsync(MakeReq(flagKey, evalCtx), cancellationToken: ct);
        return (resp.ValueJson.ToByteArray(), resp.Metadata);
    }
}
