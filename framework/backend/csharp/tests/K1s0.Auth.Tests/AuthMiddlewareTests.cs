using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using System.Text.Json;
using FluentAssertions;
using K1s0.Auth.Jwt;
using K1s0.Auth.Middleware;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Logging;
using Microsoft.IdentityModel.Tokens;
using Moq;

namespace K1s0.Auth.Tests;

public class AuthMiddlewareTests : IDisposable
{
    private readonly RSA _rsa;
    private readonly RsaSecurityKey _securityKey;
    private readonly JwtVerifier _verifier;
    private readonly Mock<ILogger<AuthMiddleware>> _loggerMock = new();

    public AuthMiddlewareTests()
    {
        _rsa = RSA.Create(2048);
        _securityKey = new RsaSecurityKey(_rsa) { KeyId = "test-key-1" };

        var jwk = JsonWebKeyConverter.ConvertFromRSASecurityKey(_securityKey);
        jwk.Use = "sig";
        jwk.Alg = "RS256";
        var jwksJson = JsonSerializer.Serialize(new { keys = new[] { jwk } });

        var handler = new MockHttpMessageHandler(jwksJson);
        var httpClient = new HttpClient(handler);

        var config = new JwtVerifierConfig(
            Issuer: "https://test.example.com",
            JwksUri: "https://test.example.com/.well-known/jwks.json",
            Audience: "test-audience");

        _verifier = new JwtVerifier(config, httpClient);
    }

    [Fact]
    public async Task InvokeAsync_ValidToken_SetsClaims()
    {
        var token = CreateToken(DateTime.UtcNow.AddHours(1));
        var context = new DefaultHttpContext();
        context.Request.Headers.Authorization = $"Bearer {token}";
        var nextCalled = false;

        var middleware = new AuthMiddleware(_ =>
        {
            nextCalled = true;
            return Task.CompletedTask;
        }, _verifier, _loggerMock.Object);

        await middleware.InvokeAsync(context);

        nextCalled.Should().BeTrue();
        context.Items["Claims"].Should().BeOfType<Claims>();
    }

    [Fact]
    public async Task InvokeAsync_MissingToken_Returns401()
    {
        var context = new DefaultHttpContext();
        var nextCalled = false;

        var middleware = new AuthMiddleware(_ =>
        {
            nextCalled = true;
            return Task.CompletedTask;
        }, _verifier, _loggerMock.Object);

        await middleware.InvokeAsync(context);

        nextCalled.Should().BeFalse();
        context.Response.StatusCode.Should().Be(401);
    }

    [Fact]
    public async Task InvokeAsync_HealthzPath_SkipsAuth()
    {
        var context = new DefaultHttpContext();
        context.Request.Path = "/healthz";
        var nextCalled = false;

        var middleware = new AuthMiddleware(_ =>
        {
            nextCalled = true;
            return Task.CompletedTask;
        }, _verifier, _loggerMock.Object);

        await middleware.InvokeAsync(context);

        nextCalled.Should().BeTrue();
    }

    [Fact]
    public async Task InvokeAsync_ReadyzPath_SkipsAuth()
    {
        var context = new DefaultHttpContext();
        context.Request.Path = "/readyz";
        var nextCalled = false;

        var middleware = new AuthMiddleware(_ =>
        {
            nextCalled = true;
            return Task.CompletedTask;
        }, _verifier, _loggerMock.Object);

        await middleware.InvokeAsync(context);

        nextCalled.Should().BeTrue();
    }

    private string CreateToken(DateTime expires)
    {
        var descriptor = new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity(new[] { new Claim("sub", "user-1") }),
            Expires = expires,
            Issuer = "https://test.example.com",
            Audience = "test-audience",
            SigningCredentials = new SigningCredentials(_securityKey, SecurityAlgorithms.RsaSha256),
        };

        var handler = new JwtSecurityTokenHandler();
        return handler.WriteToken(handler.CreateToken(descriptor));
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
