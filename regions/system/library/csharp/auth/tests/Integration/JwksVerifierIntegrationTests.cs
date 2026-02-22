using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using Microsoft.IdentityModel.Tokens;
using WireMock.RequestBuilders;
using WireMock.ResponseBuilders;
using WireMock.Server;
using Xunit;

namespace K1s0.System.Auth.Tests.Integration;

[Trait("Category", "Integration")]
public class JwksVerifierIntegrationTests : IDisposable
{
    private readonly WireMockServer _server;
    private readonly RSA _rsa;
    private readonly RsaSecurityKey _securityKey;
    private readonly JsonWebKey _jwk;

    public JwksVerifierIntegrationTests()
    {
        _server = WireMockServer.Start();
        _rsa = RSA.Create(2048);
        _securityKey = new RsaSecurityKey(_rsa) { KeyId = "integration-test-key" };
        _jwk = JsonWebKeyConverter.ConvertFromRSASecurityKey(_securityKey);
    }

    [Fact]
    public async Task VerifyToken_WithWireMockJwks_Success()
    {
        // Arrange
        var jwksJson = global::System.Text.Json.JsonSerializer.Serialize(new
        {
            keys = new[]
            {
                new
                {
                    kty = _jwk.Kty,
                    kid = _jwk.Kid,
                    use = "sig",
                    alg = "RS256",
                    n = _jwk.N,
                    e = _jwk.E,
                },
            },
        });

        _server.Given(
            Request.Create().WithPath("/.well-known/jwks.json").UsingGet())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(jwksJson));

        var config = new AuthConfig
        {
            JwksUrl = $"{_server.Url}/.well-known/jwks.json",
            Issuer = "https://auth.example.com",
            Audience = "k1s0-api",
            CacheTtlSeconds = 300,
        };

        var httpClientFactory = new TestHttpClientFactory();
        var fetcher = new HttpJwksFetcher(httpClientFactory);
        var verifier = new JwksVerifier(fetcher, config);

        var token = CreateTestToken();

        // Act
        var claims = await verifier.VerifyTokenAsync(token);

        // Assert
        Assert.Equal("integration-test-user", claims.Sub);
        Assert.Equal("https://auth.example.com", claims.Iss);
        Assert.Equal("k1s0-api", claims.Aud);
    }

    [Fact]
    public async Task VerifyToken_JwksEndpointDown_ThrowsAuthException()
    {
        // Arrange
        _server.Given(
            Request.Create().WithPath("/.well-known/jwks.json").UsingGet())
            .RespondWith(
                Response.Create().WithStatusCode(500));

        var config = new AuthConfig
        {
            JwksUrl = $"{_server.Url}/.well-known/jwks.json",
            Issuer = "https://auth.example.com",
            Audience = "k1s0-api",
        };

        var httpClientFactory = new TestHttpClientFactory();
        var fetcher = new HttpJwksFetcher(httpClientFactory);
        var verifier = new JwksVerifier(fetcher, config);

        // Act & Assert
        await Assert.ThrowsAsync<AuthException>(() => verifier.VerifyTokenAsync("some-token"));
    }

    [Fact]
    public async Task VerifyToken_CacheTtlRespected_FetchesOnlyOnce()
    {
        // Arrange
        var jwksJson = global::System.Text.Json.JsonSerializer.Serialize(new
        {
            keys = new[]
            {
                new
                {
                    kty = _jwk.Kty,
                    kid = _jwk.Kid,
                    use = "sig",
                    alg = "RS256",
                    n = _jwk.N,
                    e = _jwk.E,
                },
            },
        });

        _server.Given(
            Request.Create().WithPath("/.well-known/jwks.json").UsingGet())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(jwksJson));

        var config = new AuthConfig
        {
            JwksUrl = $"{_server.Url}/.well-known/jwks.json",
            Issuer = "https://auth.example.com",
            Audience = "k1s0-api",
            CacheTtlSeconds = 300,
        };

        var httpClientFactory = new TestHttpClientFactory();
        var fetcher = new HttpJwksFetcher(httpClientFactory);
        var verifier = new JwksVerifier(fetcher, config);

        var token = CreateTestToken();

        // Act
        await verifier.VerifyTokenAsync(token);
        await verifier.VerifyTokenAsync(token);
        await verifier.VerifyTokenAsync(token);

        // Assert - JWKS endpoint should only be called once
        var requests = _server.LogEntries;
        Assert.Single(requests);
    }

    private string CreateTestToken()
    {
        var credentials = new SigningCredentials(_securityKey, SecurityAlgorithms.RsaSha256);
        var handler = new JwtSecurityTokenHandler();

        var tokenDescriptor = new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity(
            [
                new Claim(JwtRegisteredClaimNames.Sub, "integration-test-user"),
                new Claim("scope", "openid profile"),
            ]),
            Issuer = "https://auth.example.com",
            Audience = "k1s0-api",
            Expires = DateTime.UtcNow.AddMinutes(30),
            IssuedAt = DateTime.UtcNow,
            SigningCredentials = credentials,
        };

        var token = handler.CreateToken(tokenDescriptor);
        return handler.WriteToken(token);
    }

    public void Dispose()
    {
        _server.Stop();
        _server.Dispose();
        _rsa.Dispose();
    }

    private sealed class TestHttpClientFactory : IHttpClientFactory
    {
        public HttpClient CreateClient(string name) => new();
    }
}
