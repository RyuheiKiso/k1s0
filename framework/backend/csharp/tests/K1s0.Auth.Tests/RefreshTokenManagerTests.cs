using System.Security.Cryptography;
using FluentAssertions;

namespace K1s0.Auth.Tests;

public class RefreshTokenManagerTests
{
    private readonly RefreshTokenManager _manager;

    public RefreshTokenManagerTests()
    {
        var key = RandomNumberGenerator.GetBytes(32);
        _manager = new RefreshTokenManager(key);
    }

    [Fact]
    public void Issue_ReturnsNonEmptyToken()
    {
        var token = _manager.Issue("user-1");

        token.Should().NotBeNullOrEmpty();
        token.Should().Contain(".");
    }

    [Fact]
    public void Verify_ValidToken_ReturnsSubject()
    {
        var token = _manager.Issue("user-1");

        var sub = _manager.Verify(token);

        sub.Should().Be("user-1");
    }

    [Fact]
    public void Verify_InvalidToken_ReturnsNull()
    {
        var sub = _manager.Verify("invalid.token");

        sub.Should().BeNull();
    }

    [Fact]
    public async Task RevokeAsync_PreventsVerification()
    {
        var token = _manager.Issue("user-1");

        await _manager.RevokeAsync(token);

        _manager.Verify(token).Should().BeNull();
    }

    [Fact]
    public void Verify_ExpiredToken_ReturnsNull()
    {
        var key = RandomNumberGenerator.GetBytes(32);
        var manager = new RefreshTokenManager(key, TimeSpan.FromMilliseconds(1));
        var token = manager.Issue("user-1");

        Thread.Sleep(10);

        manager.Verify(token).Should().BeNull();
    }
}
