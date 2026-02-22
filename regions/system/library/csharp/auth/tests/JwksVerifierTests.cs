using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using Microsoft.IdentityModel.Tokens;
using NSubstitute;
using Xunit;

namespace K1s0.System.Auth.Tests;

public class JwksVerifierTests
{
    private readonly IJwksFetcher _mockFetcher;
    private readonly AuthConfig _config;
    private readonly RSA _rsa;
    private readonly RsaSecurityKey _securityKey;
    private readonly JsonWebKey _jwk;

    public JwksVerifierTests()
    {
        _mockFetcher = Substitute.For<IJwksFetcher>();
        _config = new AuthConfig
        {
            JwksUrl = "https://auth.example.com/.well-known/jwks.json",
            Issuer = "https://auth.example.com",
            Audience = "k1s0-api",
            CacheTtlSeconds = 300,
        };

        _rsa = RSA.Create(2048);
        _securityKey = new RsaSecurityKey(_rsa) { KeyId = "test-key-id" };
        _jwk = JsonWebKeyConverter.ConvertFromRSASecurityKey(_securityKey);
    }

    [Fact]
    public async Task VerifyTokenAsync_ValidToken_ReturnsClaims()
    {
        // Arrange
        _mockFetcher.FetchKeysAsync(_config.JwksUrl, Arg.Any<CancellationToken>())
            .Returns([_jwk]);

        var token = CreateTestToken();
        var verifier = new JwksVerifier(_mockFetcher, _config);

        // Act
        var claims = await verifier.VerifyTokenAsync(token);

        // Assert
        Assert.Equal("test-user-id", claims.Sub);
        Assert.Equal("https://auth.example.com", claims.Iss);
        Assert.Equal("k1s0-api", claims.Aud);
    }

    [Fact]
    public async Task VerifyTokenAsync_ExpiredToken_ThrowsAuthException()
    {
        // Arrange
        _mockFetcher.FetchKeysAsync(_config.JwksUrl, Arg.Any<CancellationToken>())
            .Returns([_jwk]);

        var token = CreateTestToken(expiredMinutesAgo: 10);
        var verifier = new JwksVerifier(_mockFetcher, _config);

        // Act & Assert
        var ex = await Assert.ThrowsAsync<AuthException>(() => verifier.VerifyTokenAsync(token));
        Assert.Equal("TOKEN_EXPIRED", ex.Code);
    }

    [Fact]
    public async Task VerifyTokenAsync_InvalidToken_ThrowsAuthException()
    {
        // Arrange
        _mockFetcher.FetchKeysAsync(_config.JwksUrl, Arg.Any<CancellationToken>())
            .Returns([_jwk]);

        var verifier = new JwksVerifier(_mockFetcher, _config);

        // Act & Assert
        var ex = await Assert.ThrowsAsync<AuthException>(() => verifier.VerifyTokenAsync("invalid-token"));
        Assert.Equal("INVALID_TOKEN", ex.Code);
    }

    [Fact]
    public async Task VerifyTokenAsync_CachesKeys()
    {
        // Arrange
        _mockFetcher.FetchKeysAsync(_config.JwksUrl, Arg.Any<CancellationToken>())
            .Returns([_jwk]);

        var token = CreateTestToken();
        var verifier = new JwksVerifier(_mockFetcher, _config);

        // Act - verify twice
        await verifier.VerifyTokenAsync(token);
        await verifier.VerifyTokenAsync(token);

        // Assert - fetcher should only be called once due to caching
        await _mockFetcher.Received(1).FetchKeysAsync(_config.JwksUrl, Arg.Any<CancellationToken>());
    }

    [Fact]
    public async Task VerifyTokenAsync_WrongIssuer_ThrowsAuthException()
    {
        // Arrange
        _mockFetcher.FetchKeysAsync(_config.JwksUrl, Arg.Any<CancellationToken>())
            .Returns([_jwk]);

        var token = CreateTestToken(issuer: "https://wrong-issuer.com");
        var verifier = new JwksVerifier(_mockFetcher, _config);

        // Act & Assert
        var ex = await Assert.ThrowsAsync<AuthException>(() => verifier.VerifyTokenAsync(token));
        Assert.Equal("INVALID_TOKEN", ex.Code);
    }

    private string CreateTestToken(int expiredMinutesAgo = -30, string? issuer = null)
    {
        var credentials = new SigningCredentials(_securityKey, SecurityAlgorithms.RsaSha256);
        var handler = new JwtSecurityTokenHandler();

        var expires = DateTime.UtcNow.AddMinutes(-expiredMinutesAgo);
        var issuedAt = expires.AddHours(-1);

        var tokenDescriptor = new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity(
            [
                new Claim(JwtRegisteredClaimNames.Sub, "test-user-id"),
                new Claim("scope", "openid profile"),
            ]),
            Issuer = issuer ?? "https://auth.example.com",
            Audience = "k1s0-api",
            NotBefore = issuedAt,
            IssuedAt = issuedAt,
            Expires = expires,
            SigningCredentials = credentials,
        };

        var token = handler.CreateToken(tokenDescriptor);
        return handler.WriteToken(token);
    }
}
