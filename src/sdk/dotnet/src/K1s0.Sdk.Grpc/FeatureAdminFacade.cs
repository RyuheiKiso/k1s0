// 本ファイルは k1s0 .NET SDK の FeatureAdmin 動詞統一 facade。
using K1s0.Sdk.Generated.K1s0.Tier1.Feature.V1;

namespace K1s0.Sdk;

public sealed class FeatureAdminFacade
{
    private readonly K1s0Client _client;
    internal FeatureAdminFacade(K1s0Client client) { _client = client; }

    /// RegisterFlagAsync: Flag 定義の登録（permission 種別は approvalId 必須）。
    public async Task<long> RegisterFlagAsync(
        FlagDefinition flag, string changeReason, string approvalId = "", CancellationToken ct = default)
    {
        var resp = await _client.Raw.FeatureAdmin.RegisterFlagAsync(new RegisterFlagRequest
        {
            Flag = flag,
            ChangeReason = changeReason,
            ApprovalId = approvalId,
            Context = _client.TenantContext(),
        }, cancellationToken: ct);
        return resp.Version;
    }

    /// GetFlagAsync: Flag 定義の取得。version 省略で最新。
    public async Task<(FlagDefinition? Flag, long Version)> GetFlagAsync(
        string flagKey, long? version = null, CancellationToken ct = default)
    {
        var req = new GetFlagRequest { FlagKey = flagKey, Context = _client.TenantContext() };
        if (version.HasValue) req.Version = version.Value;
        var resp = await _client.Raw.FeatureAdmin.GetFlagAsync(req, cancellationToken: ct);
        return (resp.Flag, resp.Version);
    }

    /// ListFlagsAsync: Flag 定義の一覧。
    public async Task<IReadOnlyList<FlagDefinition>> ListFlagsAsync(
        FlagKind? kind = null, FlagState? state = null, CancellationToken ct = default)
    {
        var req = new ListFlagsRequest { Context = _client.TenantContext() };
        if (kind.HasValue) req.Kind = kind.Value;
        if (state.HasValue) req.State = state.Value;
        var resp = await _client.Raw.FeatureAdmin.ListFlagsAsync(req, cancellationToken: ct);
        return resp.Flags.ToList();
    }
}
