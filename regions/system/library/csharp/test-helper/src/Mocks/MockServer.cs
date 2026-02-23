namespace K1s0.System.TestHelper.Mocks;

/// <summary>モックサーバー (インメモリ)。</summary>
public class MockServer
{
    private readonly List<MockRoute> _routes;
    private readonly List<(string Method, string Path)> _requests = new();

    public MockServer(IEnumerable<MockRoute> routes)
    {
        _routes = routes.ToList();
    }

    /// <summary>登録済みルートからレスポンスを取得する。</summary>
    public (int Status, string Body)? Handle(string method, string path)
    {
        _requests.Add((method, path));
        var route = _routes.FirstOrDefault(r => r.Method == method && r.Path == path);
        return route is null ? null : (route.Status, route.Body);
    }

    /// <summary>記録されたリクエスト数を返す。</summary>
    public int RequestCount => _requests.Count;

    /// <summary>記録されたリクエストを返す。</summary>
    public IReadOnlyList<(string Method, string Path)> RecordedRequests =>
        _requests.AsReadOnly();
}
