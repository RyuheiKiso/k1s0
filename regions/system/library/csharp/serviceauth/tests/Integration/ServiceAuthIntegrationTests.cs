using System.Text.Json;
using K1s0.System.ServiceAuth;
using WireMock.RequestBuilders;
using WireMock.ResponseBuilders;
using WireMock.Server;

namespace K1s0.System.ServiceAuth.Tests.Integration;

[Trait("Category", "Integration")]
public class ServiceAuthIntegrationTests : IDisposable
{
    private readonly WireMockServer _server;

    public ServiceAuthIntegrationTests()
    {
        _server = WireMockServer.Start();
    }

    public void Dispose()
    {
        _server.Dispose();
    }

    [Fact]
    public async Task GetTokenAsync_WithWireMock_ReturnsValidToken()
    {
        var tokenResponse = JsonSerializer.Serialize(new
        {
            access_token = "integration-test-token",
            expires_in = 3600,
            token_type = "Bearer",
        });

        _server.Given(
                Request.Create().WithPath("/token").UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(tokenResponse));

        var config = new ServiceAuthConfig(
            TokenUrl: $"{_server.Url}/token",
            ClientId: "test-client",
            ClientSecret: "test-secret");

        using var httpClient = new HttpClient();
        await using var client = new ServiceAuthClient(httpClient, config);

        var token = await client.GetTokenAsync();

        Assert.Equal("integration-test-token", token.AccessToken);
        Assert.Equal("Bearer", token.TokenType);
        Assert.False(token.IsExpired);
    }

    [Fact]
    public async Task GetTokenAsync_WithWireMock_Unauthorized_Throws()
    {
        _server.Given(
                Request.Create().WithPath("/token").UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(401)
                    .WithBody("Unauthorized"));

        var config = new ServiceAuthConfig(
            TokenUrl: $"{_server.Url}/token",
            ClientId: "bad-client",
            ClientSecret: "bad-secret");

        using var httpClient = new HttpClient();
        await using var client = new ServiceAuthClient(httpClient, config);

        var ex = await Assert.ThrowsAsync<ServiceAuthException>(() => client.GetTokenAsync());
        Assert.Equal("TokenFetch", ex.Code);
    }

    [Fact]
    public async Task GetCachedTokenAsync_WithWireMock_CachesToken()
    {
        var tokenResponse = JsonSerializer.Serialize(new
        {
            access_token = "cached-integration-token",
            expires_in = 3600,
            token_type = "Bearer",
        });

        _server.Given(
                Request.Create().WithPath("/token").UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(tokenResponse));

        var config = new ServiceAuthConfig(
            TokenUrl: $"{_server.Url}/token",
            ClientId: "test-client",
            ClientSecret: "test-secret");

        using var httpClient = new HttpClient();
        await using var client = new ServiceAuthClient(httpClient, config);

        var token1 = await client.GetCachedTokenAsync();
        var token2 = await client.GetCachedTokenAsync();

        Assert.Equal(token1.AccessToken, token2.AccessToken);

        // WireMock should only have received one request due to caching.
        Assert.Single(_server.LogEntries);
    }
}
