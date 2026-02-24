using System.Net;
using K1s0.System.Health;

namespace K1s0.System.Health.Tests;

public class HttpHealthCheckTests
{
    private class FakeHandler : HttpMessageHandler
    {
        private readonly HttpStatusCode _statusCode;

        public FakeHandler(HttpStatusCode statusCode) => _statusCode = statusCode;

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request, CancellationToken ct) =>
            Task.FromResult(new HttpResponseMessage(_statusCode));
    }

    [Fact]
    public void DefaultName_IsHttp()
    {
        var check = new HttpHealthCheck("http://example.com/healthz");
        Assert.Equal("http", check.Name);
    }

    [Fact]
    public void CustomName_IsSet()
    {
        var check = new HttpHealthCheck("http://example.com/healthz", name: "upstream");
        Assert.Equal("upstream", check.Name);
    }

    [Fact]
    public async Task CheckAsync_2xx_ReturnsHealthy()
    {
        var client = new HttpClient(new FakeHandler(HttpStatusCode.OK));
        var check = new HttpHealthCheck("http://example.com/healthz", client, "test");

        var result = await check.CheckAsync();

        Assert.Equal(HealthStatus.Healthy, result.Status);
        Assert.Null(result.Message);
    }

    [Fact]
    public async Task CheckAsync_5xx_ReturnsUnhealthy()
    {
        var client = new HttpClient(new FakeHandler(HttpStatusCode.ServiceUnavailable));
        var check = new HttpHealthCheck("http://example.com/healthz", client, "test");

        var result = await check.CheckAsync();

        Assert.Equal(HealthStatus.Unhealthy, result.Status);
        Assert.Contains("status 503", result.Message);
    }

    [Fact]
    public async Task CheckAsync_IntegrationWithChecker()
    {
        var client = new HttpClient(new FakeHandler(HttpStatusCode.OK));
        var check = new HttpHealthCheck("http://example.com/healthz", client, "upstream");

        var checker = new HealthChecker();
        checker.Add(check);

        var resp = await checker.RunAllAsync();

        Assert.Equal(HealthStatus.Healthy, resp.Status);
        Assert.Equal(HealthStatus.Healthy, resp.Checks["upstream"].Status);
    }
}
