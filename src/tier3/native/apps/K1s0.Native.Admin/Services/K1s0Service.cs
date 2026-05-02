// k1s0 BFF 呼出の実装（Admin 用、HttpClient ベース）。
//
// リリース時点 minimum: BFF の REST POST /api/audit/query を呼ぶ。
// リリース時点 で K1s0.Sdk.Grpc 経由の gRPC 直接呼出に切替可能（IK1s0Service 越しなので置換可）。
//
// admin-bff には GraphQL は無く REST のみ提供される（auth role=admin 必須）。
// 本実装は admin-bff の POST /api/audit/query を期待する。

using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;

namespace K1s0.Native.Admin.Services;

public sealed class K1s0Service : IK1s0Service
{
    // BFF endpoint URL（環境変数 / 設定ファイルから注入する想定、リリース時点 はビルド時定数）。
    private static readonly string BffUrl = "https://admin.k1s0.example.com";

    // HttpClient（HostEnvironment から推奨される DI で受け取るのが本来の形だが、
    // リリース時点 minimum では singleton 共有）。
    private readonly HttpClient _http;

    public K1s0Service()
    {
        _http = new HttpClient { Timeout = TimeSpan.FromSeconds(10) };
    }

    public async Task<IReadOnlyList<AuditEvent>> QueryAuditAsync(int hours, int limit, CancellationToken ct = default)
    {
        // 範囲: 直近 hours 時間。RFC3339 文字列化する（BFF の auditQueryRequest が要求する形式）。
        var to = DateTimeOffset.UtcNow;
        var from = to.AddHours(-hours);
        var payload = JsonSerializer.Serialize(new
        {
            from = from.ToString("yyyy-MM-ddTHH:mm:ssZ"),
            to = to.ToString("yyyy-MM-ddTHH:mm:ssZ"),
            limit,
        });
        using var req = new HttpRequestMessage(HttpMethod.Post, $"{BffUrl}/api/audit/query")
        {
            Content = new StringContent(payload, Encoding.UTF8, "application/json"),
        };
        // リリース時点 minimum: tenant ID は固定（リリース時点 で SecureStorage から取得）。
        req.Headers.Add("X-Tenant-Id", "tenant-admin");
        // 認証は role=admin が必須。Bearer は SecureStorage 結線前の最小実装（採用初期で SecureStorage 化、IMP-SEC-* 参照）。
        req.Headers.Authorization = new AuthenticationHeaderValue("Bearer", "admin-dev-token");

        using var res = await _http.SendAsync(req, ct).ConfigureAwait(false);
        if (!res.IsSuccessStatusCode)
        {
            throw new HttpRequestException($"BFF returned {(int)res.StatusCode}");
        }
        var body = await res.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        // リリース時点 minimum: events 配列を順次パースする。
        using var doc = JsonDocument.Parse(body);
        if (!doc.RootElement.TryGetProperty("events", out var events) || events.ValueKind != JsonValueKind.Array)
        {
            return Array.Empty<AuditEvent>();
        }
        var result = new List<AuditEvent>(events.GetArrayLength());
        foreach (var e in events.EnumerateArray())
        {
            result.Add(new AuditEvent(
                OccurredAtMillis: e.TryGetProperty("occurred_at_millis", out var ts) && ts.ValueKind == JsonValueKind.Number ? ts.GetInt64() : 0,
                Actor: e.TryGetProperty("actor", out var actor) ? actor.GetString() ?? string.Empty : string.Empty,
                Action: e.TryGetProperty("action", out var action) ? action.GetString() ?? string.Empty : string.Empty,
                Resource: e.TryGetProperty("resource", out var resource) ? resource.GetString() ?? string.Empty : string.Empty,
                Outcome: e.TryGetProperty("outcome", out var outcome) ? outcome.GetString() ?? string.Empty : string.Empty));
        }
        return result;
    }
}
