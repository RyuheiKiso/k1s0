// k1s0 BFF 呼出の実装（HttpClient ベース）。
//
// リリース時点 minimum: BFF の REST POST /api/state/get を呼ぶ。
// リリース時点 で K1s0.Sdk.Grpc 経由の gRPC 直接呼出に切替可能（IK1s0Service 越しなので置換可）。

using System.Text;
using System.Text.Json;

namespace K1s0.Native.Hub.Services;

public sealed class K1s0Service : IK1s0Service
{
    // BFF endpoint URL（環境変数 / 設定ファイルから注入する想定、リリース時点 はビルド時定数）。
    private static readonly string BffUrl = "https://api.k1s0.example.com";

    // HttpClient（HostEnvironment から推奨される DI で受け取るのが本来の形だが、
    // リリース時点 minimum では singleton 共有）。
    private readonly HttpClient _http;

    public K1s0Service()
    {
        _http = new HttpClient { Timeout = TimeSpan.FromSeconds(10) };
    }

    public async Task<string?> GetStateAsync(string store, string key, CancellationToken ct = default)
    {
        // BFF の REST endpoint に JSON ペイロードを POST する。
        var payload = JsonSerializer.Serialize(new { store, key });
        using var req = new HttpRequestMessage(HttpMethod.Post, $"{BffUrl}/api/state/get")
        {
            Content = new StringContent(payload, Encoding.UTF8, "application/json"),
        };
        // リリース時点 minimum: tenant ID は固定（リリース時点 で SecureStorage から取得）。
        req.Headers.Add("X-Tenant-Id", "tenant-dev");

        using var res = await _http.SendAsync(req, ct).ConfigureAwait(false);
        if (!res.IsSuccessStatusCode)
        {
            throw new HttpRequestException($"BFF returned {(int)res.StatusCode}");
        }
        var body = await res.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        // 失敗時は body の error 表現を解析しないが、リリース時点 で詳細解析を入れる。
        var doc = JsonDocument.Parse(body);
        if (!doc.RootElement.TryGetProperty("found", out var found) || !found.GetBoolean())
        {
            return null;
        }
        return doc.RootElement.TryGetProperty("data", out var data) ? data.GetString() : null;
    }
}
