namespace K1s0.System.TestHelper.Mocks;

/// <summary>モックサーバービルダー。</summary>
public class MockServerBuilder
{
    private readonly string _serverType;
    private readonly List<MockRoute> _routes = new();

    private MockServerBuilder(string serverType)
    {
        _serverType = serverType;
    }

    public static MockServerBuilder NotificationServer() => new("notification");

    public static MockServerBuilder RatelimitServer() => new("ratelimit");

    public static MockServerBuilder TenantServer() => new("tenant");

    public string ServerType => _serverType;

    public MockServerBuilder WithHealthOk()
    {
        _routes.Add(new MockRoute("GET", "/health", 200, "{\"status\":\"ok\"}"));
        return this;
    }

    public MockServerBuilder WithSuccessResponse(string path, string body)
    {
        _routes.Add(new MockRoute("POST", path, 200, body));
        return this;
    }

    public MockServerBuilder WithErrorResponse(string path, int status)
    {
        _routes.Add(new MockRoute("POST", path, status, "{\"error\":\"mock error\"}"));
        return this;
    }

    public MockServer Build() => new(_routes);
}
