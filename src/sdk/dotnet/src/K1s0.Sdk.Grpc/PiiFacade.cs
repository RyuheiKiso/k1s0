// 本ファイルは k1s0 .NET SDK の Pii 動詞統一 facade。
using K1s0.Sdk.Generated.K1s0.Tier1.Pii.V1;

namespace K1s0.Sdk;

public sealed class PiiFacade
{
    private readonly K1s0Client _client;
    internal PiiFacade(K1s0Client client) { _client = client; }

    /// ClassifyAsync: PII 種別の検出。
    public async Task<(IReadOnlyList<PiiFinding> Findings, bool ContainsPii)> ClassifyAsync(string text, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Pii.ClassifyAsync(new ClassifyRequest { Text = text, Context = _client.TenantContext() }, cancellationToken: ct);
        return (resp.Findings.ToList(), resp.ContainsPii);
    }

    /// MaskAsync: マスキング。
    public async Task<(string MaskedText, IReadOnlyList<PiiFinding> Findings)> MaskAsync(string text, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Pii.MaskAsync(new MaskRequest { Text = text, Context = _client.TenantContext() }, cancellationToken: ct);
        return (resp.MaskedText, resp.Findings.ToList());
    }
}
