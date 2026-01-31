using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using System.Text.Json;
using FluentAssertions;
using K1s0.Auth.Jwt;
using Microsoft.IdentityModel.Tokens;

namespace K1s0.Auth.Tests;

public class JwtVerifierTests : IDisposable
{
    private readonly RSA _rsa;
    private readonly RsaSecurityKey _securityKey;
    private readonly JsonWebKey _jwk;
    private readonly JwtVerifierConfig _config;

    public JwtVerifierTests()
    {
        _rsa = RSA.Create(2048);
        _securityKey = new RsaSecurityKey(_rsa) { KeyId = "test-key-1" };
        _jwk = JsonWebKeyConverter.ConvertFromRSASecurityKey(_securityKey);
        _jwk.Use = "sig";
        _jwk.Alg = "RS256";

        _config = new JwtVerifierConfig(
            Issuer: "https://test-issuer.example.com",
            JwksUri: "https://test-issuer.example.com/.well-known/jwks.json",
            Audience: "test-audience");
    }

    [Fact]
    public async Task VerifyAsync_ValidToken_ReturnsClaims()
    {
        var token = CreateToken(DateTime.UtcNow.AddHours(1));
        var jwksJson = CreateJwksJson();
        using var httpClient = CreateMockHttpClient(jwksJson);
        var verifier = new JwtVerifier(_config, httpClient);

        var claims = await verifier.VerifyAsync(token);

        claims.Sub.Should().Be("user-123");
    }

    [Fact]
    public async Task VerifyAsync_ExpiredToken_ThrowsTokenExpiredException()
    {
        var token = CreateToken(DateTime.UtcNow.AddHours(-1));
        var jwksJson = CreateJwksJson();
        using var httpClient = CreateMockHttpClient(jwksJson);
        var verifier = new JwtVerifier(_config, httpClient);

        var act = () => verifier.VerifyAsync(token);

        await act.Should().ThrowAsync<TokenExpiredException>();
    }

    [Fact]
    public async Task VerifyAsync_InvalidSignature_ThrowsTokenInvalidException()
    {
        using var otherRsa = RSA.Create(2048);
        var otherKey = new RsaSecurityKey(otherRsa) { KeyId = "other-key" };
        var token = CreateTokenWithKey(DateTime.UtcNow.AddHours(1), otherKey);
        var jwksJson = CreateJwksJson();
        using var httpClient = CreateMockHttpClient(jwksJson);
        var verifier = new JwtVerifier(_config, httpClient);

        var act = () => verifier.VerifyAsync(token);

        await act.Should().ThrowAsync<TokenInvalidException>();
    }

    [Fact]
    public async Task VerifyAsync_TokenWithRoles_ParsesRoles()
    {
        var additionalClaims = new[] { new Claim("roles", "[\"admin\",\"user\"]") };
        var token = CreateToken(DateTime.UtcNow.AddHours(1), additionalClaims);
        var jwksJson = CreateJwksJson();
        using var httpClient = CreateMockHttpClient(jwksJson);
        var verifier = new JwtVerifier(_config, httpClient);

        var claims = await verifier.VerifyAsync(token);

        claims.Roles.Should().Contain("admin");
        claims.Roles.Should().Contain("user");
    }

    private string CreateToken(DateTime expires, IEnumerable<Claim>? additionalClaims = null)
    {
        return CreateTokenWithKey(expires, _securityKey, additionalClaims);
    }

    private static string CreateTokenWithKey(DateTime expires, SecurityKey key, IEnumerable<Claim>? additionalClaims = null)
    {
        var claims = new List<Claim> { new("sub", "user-123") };
        if (additionalClaims is not null)
        {
            claims.AddRange(additionalClaims);
        }

        var descriptor = new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity(claims),
            NotBefore = expires.AddHours(-2),
            IssuedAt = expires.AddHours(-2),
            Expires = expires,
            Issuer = "https://test-issuer.example.com",
            Audience = "test-audience",
            SigningCredentials = new SigningCredentials(key, SecurityAlgorithms.RsaSha256),
        };

        var handler = new JwtSecurityTokenHandler();
        var token = handler.CreateToken(descriptor);
        return handler.WriteToken(token);
    }

    private string CreateJwksJson()
    {
        var jwks = new { keys = new[] { _jwk } };
        return JsonSerializer.Serialize(jwks);
    }

    private static HttpClient CreateMockHttpClient(string jwksResponse)
    {
        var handler = new MockHttpMessageHandler(jwksResponse);
        return new HttpClient(handler);
    }

    public void Dispose()
    {
        _rsa.Dispose();
        GC.SuppressFinalize(this);
    }

    private sealed class MockHttpMessageHandler : HttpMessageHandler
    {
        private readonly string _response;

        public MockHttpMessageHandler(string response) => _response = response;

        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            return Task.FromResult(new HttpResponseMessage(System.Net.HttpStatusCode.OK)
            {
                Content = new StringContent(_response, System.Text.Encoding.UTF8, "application/json"),
            });
        }
    }
}
