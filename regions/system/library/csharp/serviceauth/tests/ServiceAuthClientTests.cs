using System.Net;
using Xunit;

namespace K1s0.System.ServiceAuth.Tests;

public class ServiceAuthClientTests
{
    private static readonly ServiceAuthConfig TestConfig = new(
        TokenUrl: "http://auth.example.com/token",
        ClientId: "test-client",
        ClientSecret: "test-secret");

    [Fact]
    public async Task GetTokenAsync_Success_ReturnsToken()
    {
        var responseJson = """{"access_token":"eyJ.test.token","expires_in":3600,"token_type":"Bearer"}""";
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.OK, responseJson);
        using var httpClient = new HttpClient(handler);
        await using var client = new ServiceAuthClient(httpClient, TestConfig);

        var token = await client.GetTokenAsync();

        Assert.Equal("eyJ.test.token", token.AccessToken);
        Assert.Equal("Bearer", token.TokenType);
        Assert.False(token.IsExpired);
    }

    [Fact]
    public async Task GetTokenAsync_Unauthorized_ThrowsServiceAuthException()
    {
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.Unauthorized, "unauthorized");
        using var httpClient = new HttpClient(handler);
        await using var client = new ServiceAuthClient(httpClient, TestConfig);

        var ex = await Assert.ThrowsAsync<ServiceAuthException>(
            () => client.GetTokenAsync());

        Assert.Equal("TokenFetch", ex.Code);
    }

    [Fact]
    public async Task GetCachedTokenAsync_ReturnsSameTokenOnSecondCall()
    {
        var responseJson = """{"access_token":"cached.token","expires_in":3600,"token_type":"Bearer"}""";
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.OK, responseJson);
        using var httpClient = new HttpClient(handler);
        await using var client = new ServiceAuthClient(httpClient, TestConfig);

        var token1 = await client.GetCachedTokenAsync();
        var token2 = await client.GetCachedTokenAsync();

        Assert.Equal(token1.AccessToken, token2.AccessToken);
        Assert.Equal(1, handler.CallCount);
    }

    [Fact]
    public async Task VerifyTokenAsync_NoJwksUri_ThrowsServiceAuthException()
    {
        var responseJson = """{"access_token":"dummy","expires_in":3600,"token_type":"Bearer"}""";
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.OK, responseJson);
        using var httpClient = new HttpClient(handler);
        await using var client = new ServiceAuthClient(httpClient, TestConfig);

        var ex = await Assert.ThrowsAsync<ServiceAuthException>(
            () => client.VerifyTokenAsync("dummy-token"));

        Assert.Equal("InvalidToken", ex.Code);
        Assert.Contains("JWKS URI", ex.Message);
    }

    private sealed class FakeHttpMessageHandler(HttpStatusCode statusCode, string responseBody) : HttpMessageHandler
    {
        public int CallCount { get; private set; }

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request, CancellationToken cancellationToken)
        {
            CallCount++;
            var response = new HttpResponseMessage(statusCode)
            {
                Content = new StringContent(responseBody),
            };
            return Task.FromResult(response);
        }
    }
}
