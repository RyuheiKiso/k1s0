using System.Net;
using NSubstitute;
using Xunit;

namespace K1s0.System.Auth.Tests;

public class HttpJwksFetcherTests
{
    [Fact]
    public async Task FetchKeysAsync_SuccessfulResponse_ReturnsKeys()
    {
        // Arrange
        var jwksJson = """
        {
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "test-key",
                    "use": "sig",
                    "alg": "RS256",
                    "n": "0vx7agoebGcQSuuPiLJXZptN9nndrQmbXEps2aiAFbWhM78LhWx4cbbfAAtVT86zwu1RK7aPFFxuhDR1L6tSoc_BJECPebWKRXjBZCiFV4n3oknjhMstn64tZ_2W-5JsGY4Hc5n9yBXArwl93lqt7_RN5w6Cf0h4QyQ5v-65YGjQR0_FDW2QvzqY368QQMicAtaSqzs8KJZgnYb9c7d0zgdAZHzu6qMQvRL5hajrn1n91CbOpbISD08qNLyrdkt-bFTWhAI4vMQFh6WeZu0fM4lFd2NcRwr3XPksINHaQ-G_xBniIqbw0Ls1jF44-csFCur-kEgU8awapJzKnqDKgw",
                    "e": "AQAB"
                }
            ]
        }
        """;

        var handler = new FakeHttpMessageHandler(new HttpResponseMessage
        {
            StatusCode = HttpStatusCode.OK,
            Content = new StringContent(jwksJson),
        });

        var httpClientFactory = Substitute.For<IHttpClientFactory>();
        httpClientFactory.CreateClient(nameof(HttpJwksFetcher)).Returns(new HttpClient(handler));

        var fetcher = new HttpJwksFetcher(httpClientFactory);

        // Act
        var keys = await fetcher.FetchKeysAsync("https://auth.example.com/.well-known/jwks.json");

        // Assert
        Assert.Single(keys);
        Assert.Equal("test-key", keys[0].Kid);
    }

    [Fact]
    public async Task FetchKeysAsync_HttpError_ThrowsAuthException()
    {
        // Arrange
        var handler = new FakeHttpMessageHandler(new HttpResponseMessage
        {
            StatusCode = HttpStatusCode.InternalServerError,
        });

        var httpClientFactory = Substitute.For<IHttpClientFactory>();
        httpClientFactory.CreateClient(nameof(HttpJwksFetcher)).Returns(new HttpClient(handler));

        var fetcher = new HttpJwksFetcher(httpClientFactory);

        // Act & Assert
        var ex = await Assert.ThrowsAsync<AuthException>(
            () => fetcher.FetchKeysAsync("https://auth.example.com/.well-known/jwks.json"));
        Assert.Equal("JWKS_FETCH_FAILED", ex.Code);
    }

    [Fact]
    public async Task FetchKeysAsync_InvalidJson_ThrowsAuthException()
    {
        // Arrange
        var handler = new FakeHttpMessageHandler(new HttpResponseMessage
        {
            StatusCode = HttpStatusCode.OK,
            Content = new StringContent("not-json"),
        });

        var httpClientFactory = Substitute.For<IHttpClientFactory>();
        httpClientFactory.CreateClient(nameof(HttpJwksFetcher)).Returns(new HttpClient(handler));

        var fetcher = new HttpJwksFetcher(httpClientFactory);

        // Act & Assert
        var ex = await Assert.ThrowsAsync<AuthException>(
            () => fetcher.FetchKeysAsync("https://auth.example.com/.well-known/jwks.json"));
        Assert.Equal("JWKS_PARSE_FAILED", ex.Code);
    }

    [Fact]
    public async Task DisposeAsync_PreventsSubsequentCalls()
    {
        // Arrange
        var httpClientFactory = Substitute.For<IHttpClientFactory>();
        var fetcher = new HttpJwksFetcher(httpClientFactory);

        // Act
        await fetcher.DisposeAsync();

        // Assert
        await Assert.ThrowsAsync<ObjectDisposedException>(
            () => fetcher.FetchKeysAsync("https://auth.example.com/.well-known/jwks.json"));
    }

    private sealed class FakeHttpMessageHandler(HttpResponseMessage response) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken) => Task.FromResult(response);
    }
}
