// 本ファイルは k1s0 .NET SDK の Secrets 動詞統一 facade。
// `client.Secrets.GetAsync(...)` 形式で SecretsService への呼出を提供する。

using K1s0.Sdk.Generated.K1s0.Tier1.Secrets.V1;

namespace K1s0.Sdk;

// Rotate オプション。
public sealed class RotateOptions
{
    // 旧バージョンの猶予時間（秒、既定 3600）。
    public int GracePeriodSec { get; init; } = 3600;

    // 動的シークレットの発行ポリシー名。
    public string? Policy { get; init; }

    // 冪等性キー。
    public string IdempotencyKey { get; init; } = string.Empty;
}

// SecretsFacade は SecretsService の動詞統一 facade。
public sealed class SecretsFacade
{
    private readonly K1s0Client _client;

    internal SecretsFacade(K1s0Client client)
    {
        _client = client;
    }

    // GetAsync はシークレット名で値（key=value マップ）と version を取得する。
    public async Task<(IDictionary<string, string> Values, int Version)> GetAsync(
        string name,
        CancellationToken ct = default)
    {
        var req = new GetSecretRequest
        {
            Name = name,
            Context = _client.TenantContext(),
        };
        // RPC 呼出。
        var resp = await _client.RawSecrets().GetAsync(req, cancellationToken: ct);
        // (Values, Version) を返却する。
        return (resp.Values, resp.Version);
    }

    // RotateAsync はシークレットのローテーション。新バージョンと旧バージョンを返す。
    public async Task<(int NewVersion, int PreviousVersion)> RotateAsync(
        string name,
        RotateOptions? opts = null,
        CancellationToken ct = default)
    {
        opts ??= new RotateOptions();
        var req = new RotateSecretRequest
        {
            Name = name,
            Context = _client.TenantContext(),
            GracePeriodSec = opts.GracePeriodSec,
            IdempotencyKey = opts.IdempotencyKey,
        };
        // policy はオプショナル（proto3 optional）。
        if (opts.Policy != null)
        {
            req.Policy = opts.Policy;
        }
        // RPC 呼出。
        var resp = await _client.RawSecrets().RotateAsync(req, cancellationToken: ct);
        // (NewVersion, PreviousVersion) を返却する。
        return (resp.NewVersion, resp.PreviousVersion);
    }

    /// GetDynamicAsync は動的 Secret 発行（FR-T1-SECRETS-002）。
    /// engine="postgres" / "mysql" / "kafka" 等の OpenBao Database Engine 種別を指定する。
    /// ttlSec=0 で既定 1 時間（3600）、上限 24 時間（86400）に clamp される。
    public async Task<DynamicSecret> GetDynamicAsync(
        string engine,
        string role,
        int ttlSec = 0,
        CancellationToken ct = default)
    {
        var req = new GetDynamicSecretRequest
        {
            Engine = engine,
            Role = role,
            TtlSec = ttlSec,
            Context = _client.TenantContext(),
        };
        var resp = await _client.RawSecrets().GetDynamicAsync(req, cancellationToken: ct);
        return new DynamicSecret
        {
            Values = resp.Values,
            LeaseId = resp.LeaseId,
            TtlSec = resp.TtlSec,
            IssuedAtMs = resp.IssuedAtMs,
        };
    }
}

/// 動的 Secret 発行（FR-T1-SECRETS-002）の応答を SDK 利用者向けに整理した型。
public sealed class DynamicSecret
{
    /// credential 一式（"username" / "password" など、engine 別の field）。
    public IDictionary<string, string> Values { get; init; } = new Dictionary<string, string>();
    /// OpenBao の lease ID（renewal / revoke 用）。
    public string LeaseId { get; init; } = string.Empty;
    /// 実際に付与された TTL 秒（要求値から ceiling までクランプされる）。
    public int TtlSec { get; init; }
    /// 発効時刻（Unix epoch ミリ秒）。
    public long IssuedAtMs { get; init; }
}
