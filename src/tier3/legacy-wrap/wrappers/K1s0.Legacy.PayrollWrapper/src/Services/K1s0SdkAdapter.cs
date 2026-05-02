// k1s0 BFF を HttpClient で叩く IK1s0SdkAdapter 実装。
//
// リリース時点 minimum: BFF REST POST /api/state/save と /api/audit/record を呼ぶ。
// リリース時点 で K1s0.Sdk.Grpc 経由の gRPC 直接呼出に切替可能（interface 越しなので置換可）。

using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;

namespace K1s0.Legacy.PayrollWrapper.Services;

public sealed class K1s0SdkAdapter : IK1s0SdkAdapter
{
    // HttpClient（DI で注入される）。
    private readonly HttpClient _http;
    // BFF endpoint URL（appsettings から DI で注入）。
    private readonly string _bffUrl;
    // テナント ID（appsettings から DI で注入）。
    private readonly string _tenantId;

    public K1s0SdkAdapter(HttpClient http, string bffUrl, string tenantId)
    {
        _http = http;
        _bffUrl = bffUrl.TrimEnd('/');
        _tenantId = tenantId;
    }

    public async Task SaveStateAsync(string store, string key, string jsonValue, CancellationToken ct = default)
    {
        // BFF stateSaveRequest と整合する JSON を組み立てる。
        var payload = JsonSerializer.Serialize(new { store, key, data = jsonValue });
        await PostAsync("/api/state/save", payload, ct).ConfigureAwait(false);
    }

    public async Task RecordAuditAsync(string actor, string action, string resource, string outcome, CancellationToken ct = default)
    {
        // BFF auditRecordRequest と整合する JSON を組み立てる。
        var payload = JsonSerializer.Serialize(new { actor, action, resource, outcome });
        await PostAsync("/api/audit/record", payload, ct).ConfigureAwait(false);
    }

    private async Task PostAsync(string path, string jsonPayload, CancellationToken ct)
    {
        using var req = new HttpRequestMessage(HttpMethod.Post, $"{_bffUrl}{path}")
        {
            Content = new StringContent(jsonPayload, Encoding.UTF8, "application/json"),
        };
        // テナント / 認証ヘッダ。Bearer は環境変数 / SecureStorage 経由の最小実装（採用初期 SecureStorage 結線、IMP-SEC-* 参照）。
        req.Headers.Add("X-Tenant-Id", _tenantId);
        req.Headers.Authorization = new AuthenticationHeaderValue("Bearer", "wrapper-dev-token");
        using var res = await _http.SendAsync(req, ct).ConfigureAwait(false);
        if (!res.IsSuccessStatusCode)
        {
            throw new HttpRequestException($"BFF {path} returned {(int)res.StatusCode}");
        }
    }
}
