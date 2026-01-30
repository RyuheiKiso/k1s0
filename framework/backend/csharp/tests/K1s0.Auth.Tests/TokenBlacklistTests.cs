using FluentAssertions;

namespace K1s0.Auth.Tests;

public class TokenBlacklistTests : IDisposable
{
    private readonly InMemoryBlacklist _blacklist = new();

    [Fact]
    public async Task IsBlacklisted_AfterAdd_ReturnsTrue()
    {
        await _blacklist.AddAsync("jti-1", DateTime.UtcNow.AddHours(1));

        var result = await _blacklist.IsBlacklistedAsync("jti-1");

        result.Should().BeTrue();
    }

    [Fact]
    public async Task IsBlacklisted_UnknownToken_ReturnsFalse()
    {
        var result = await _blacklist.IsBlacklistedAsync("unknown");

        result.Should().BeFalse();
    }

    [Fact]
    public async Task IsBlacklisted_ExpiredEntry_ReturnsFalse()
    {
        await _blacklist.AddAsync("jti-expired", DateTime.UtcNow.AddSeconds(-1));

        var result = await _blacklist.IsBlacklistedAsync("jti-expired");

        result.Should().BeFalse();
    }

    public void Dispose()
    {
        _blacklist.Dispose();
        GC.SuppressFinalize(this);
    }
}
