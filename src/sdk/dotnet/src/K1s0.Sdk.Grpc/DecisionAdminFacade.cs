// 本ファイルは k1s0 .NET SDK の DecisionAdmin 動詞統一 facade。
using Google.Protobuf;
using K1s0.Sdk.Generated.K1s0.Tier1.Decision.V1;

namespace K1s0.Sdk;

public sealed class DecisionAdminFacade
{
    private readonly K1s0Client _client;
    internal DecisionAdminFacade(K1s0Client client) { _client = client; }

    /// RegisterRuleAsync: JDM 文書の登録。返り値は (RuleVersion, EffectiveAtMs)。
    public async Task<(string RuleVersion, long EffectiveAtMs)> RegisterRuleAsync(
        string ruleId, byte[] jdmDocument, byte[] sigstoreSignature, string commitHash, CancellationToken ct = default)
    {
        var resp = await _client.Raw.DecisionAdmin.RegisterRuleAsync(new RegisterRuleRequest
        {
            RuleId = ruleId,
            JdmDocument = ByteString.CopyFrom(jdmDocument),
            SigstoreSignature = ByteString.CopyFrom(sigstoreSignature),
            CommitHash = commitHash,
            Context = _client.TenantContext(),
        }, cancellationToken: ct);
        return (resp.RuleVersion, resp.EffectiveAtMs);
    }

    /// ListVersionsAsync: バージョン一覧。
    public async Task<IReadOnlyList<RuleVersionMeta>> ListVersionsAsync(string ruleId, CancellationToken ct = default)
    {
        var resp = await _client.Raw.DecisionAdmin.ListVersionsAsync(new ListVersionsRequest
        {
            RuleId = ruleId,
            Context = _client.TenantContext(),
        }, cancellationToken: ct);
        return resp.Versions.ToList();
    }

    /// GetRuleAsync: 特定バージョンの取得。
    public async Task<(byte[] JdmDocument, RuleVersionMeta? Meta)> GetRuleAsync(
        string ruleId, string ruleVersion, CancellationToken ct = default)
    {
        var resp = await _client.Raw.DecisionAdmin.GetRuleAsync(new GetRuleRequest
        {
            RuleId = ruleId,
            RuleVersion = ruleVersion,
            Context = _client.TenantContext(),
        }, cancellationToken: ct);
        return (resp.JdmDocument.ToByteArray(), resp.Meta);
    }
}
