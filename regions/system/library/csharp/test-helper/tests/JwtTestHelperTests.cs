using K1s0.System.TestHelper.Jwt;
using Xunit;

namespace K1s0.System.TestHelper.Tests;

public class JwtTestHelperTests
{
    private readonly JwtTestHelper _helper = new("test-secret-key-long-enough-for-hmac");

    [Fact]
    public void CreateAdminToken_ReturnsValidToken()
    {
        var token = _helper.CreateAdminToken();
        Assert.NotEmpty(token);
        Assert.Contains(".", token);
    }

    [Fact]
    public void CreateAdminToken_DecodesCorrectly()
    {
        var token = _helper.CreateAdminToken();
        var claims = _helper.DecodeClaims(token);
        Assert.NotNull(claims);
        Assert.Equal("admin", claims.Sub);
        Assert.Contains("admin", claims.Roles);
    }

    [Fact]
    public void CreateUserToken_DecodesCorrectly()
    {
        var token = _helper.CreateUserToken("user-123", "user", "reader");
        var claims = _helper.DecodeClaims(token);
        Assert.NotNull(claims);
        Assert.Equal("user-123", claims.Sub);
        Assert.Contains("user", claims.Roles);
        Assert.Contains("reader", claims.Roles);
    }

    [Fact]
    public void CreateToken_WithTenant_DecodesCorrectly()
    {
        var token = _helper.CreateToken(new TestClaims("svc", new[] { "service" }, TenantId: "t-1"));
        var claims = _helper.DecodeClaims(token);
        Assert.NotNull(claims);
        Assert.Equal("t-1", claims.TenantId);
    }

    [Fact]
    public void DecodeClaims_InvalidToken_ReturnsNull()
    {
        var claims = _helper.DecodeClaims("invalid-token");
        Assert.Null(claims);
    }
}
