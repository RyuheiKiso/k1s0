using K1s0.System.TestHelper.Mocks;
using Xunit;

namespace K1s0.System.TestHelper.Tests;

public class MockServerTests
{
    [Fact]
    public void NotificationServer_WithHealthOk_ReturnsOk()
    {
        var server = MockServerBuilder.NotificationServer()
            .WithHealthOk()
            .WithSuccessResponse("/send", "{\"id\":\"1\",\"status\":\"sent\"}")
            .Build();

        var health = server.Handle("GET", "/health");
        Assert.NotNull(health);
        Assert.Equal(200, health.Value.Status);
        Assert.Contains("ok", health.Value.Body);

        var send = server.Handle("POST", "/send");
        Assert.NotNull(send);
        Assert.Equal(200, send.Value.Status);

        Assert.Equal(2, server.RequestCount);
    }

    [Fact]
    public void RatelimitServer_UnknownRoute_ReturnsNull()
    {
        var server = MockServerBuilder.RatelimitServer().WithHealthOk().Build();
        Assert.Null(server.Handle("GET", "/nonexistent"));
    }

    [Fact]
    public void TenantServer_ErrorResponse()
    {
        var server = MockServerBuilder.TenantServer()
            .WithErrorResponse("/create", 500)
            .Build();
        var result = server.Handle("POST", "/create");
        Assert.NotNull(result);
        Assert.Equal(500, result.Value.Status);
        Assert.Contains("error", result.Value.Body);
    }
}
