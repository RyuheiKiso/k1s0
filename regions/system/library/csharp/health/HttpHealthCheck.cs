namespace K1s0.System.Health;

/// <summary>
/// HTTP GET リクエストでヘルスを確認する IHealthCheck 実装。
/// </summary>
public class HttpHealthCheck : IHealthCheck
{
    private readonly HttpClient _client;
    private readonly string _url;

    public string Name { get; }

    public HttpHealthCheck(string url, TimeSpan? timeout = null, string? name = null)
    {
        Name = name ?? "http";
        _url = url;
        _client = new HttpClient
        {
            Timeout = timeout ?? TimeSpan.FromSeconds(5),
        };
    }

    /// <summary>
    /// テスト用に HttpClient を注入できるコンストラクタ。
    /// </summary>
    internal HttpHealthCheck(string url, HttpClient client, string? name = null)
    {
        Name = name ?? "http";
        _url = url;
        _client = client;
    }

    public async Task<CheckResult> CheckAsync(CancellationToken ct = default)
    {
        try
        {
            var response = await _client.GetAsync(_url, ct).ConfigureAwait(false);

            return response.IsSuccessStatusCode
                ? new CheckResult(HealthStatus.Healthy)
                : new CheckResult(HealthStatus.Unhealthy,
                    $"HTTP {_url} returned status {(int)response.StatusCode}");
        }
        catch (TaskCanceledException) when (!ct.IsCancellationRequested)
        {
            return new CheckResult(HealthStatus.Unhealthy,
                $"HTTP check timeout: {_url}");
        }
        catch (Exception ex)
        {
            return new CheckResult(HealthStatus.Unhealthy,
                $"HTTP check failed: {ex.Message}");
        }
    }
}
