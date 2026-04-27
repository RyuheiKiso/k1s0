// 本ファイルは k1s0 .NET SDK の Decision 動詞統一 facade（評価部のみ）。
using Google.Protobuf;
using K1s0.Sdk.Generated.K1s0.Tier1.Decision.V1;

namespace K1s0.Sdk;

public sealed class DecisionFacade
{
    private readonly K1s0Client _client;
    internal DecisionFacade(K1s0Client client) { _client = client; }

    /// EvaluateAsync: ルール評価（同期）。返り値は (outputJson, traceJson, elapsedUs)。
    public async Task<(byte[] OutputJson, byte[] TraceJson, long ElapsedUs)> EvaluateAsync(
        string ruleId, string ruleVersion, byte[] inputJson, bool includeTrace = false, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Decision.EvaluateAsync(new EvaluateRequest
        {
            RuleId = ruleId,
            RuleVersion = ruleVersion,
            InputJson = ByteString.CopyFrom(inputJson),
            IncludeTrace = includeTrace,
            Context = _client.TenantContext(),
        }, cancellationToken: ct);
        return (resp.OutputJson.ToByteArray(), resp.TraceJson.ToByteArray(), resp.ElapsedUs);
    }

    /// BatchEvaluateAsync: バッチ評価。
    public async Task<IReadOnlyList<byte[]>> BatchEvaluateAsync(
        string ruleId, string ruleVersion, IEnumerable<byte[]> inputs, CancellationToken ct = default)
    {
        var req = new BatchEvaluateRequest { RuleId = ruleId, RuleVersion = ruleVersion, Context = _client.TenantContext() };
        foreach (var i in inputs) req.InputsJson.Add(ByteString.CopyFrom(i));
        var resp = await _client.Raw.Decision.BatchEvaluateAsync(req, cancellationToken: ct);
        return resp.OutputsJson.Select(b => b.ToByteArray()).ToList();
    }
}
