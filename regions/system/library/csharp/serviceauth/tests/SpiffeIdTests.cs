using K1s0.System.ServiceAuth;

namespace K1s0.System.ServiceAuth.Tests;

public class SpiffeIdTests
{
    [Fact]
    public void Parse_ValidUri_ReturnsSpiffeId()
    {
        var result = SpiffeId.Parse("spiffe://k1s0.internal/ns/system/sa/auth-service");

        Assert.Equal("k1s0.internal", result.TrustDomain);
        Assert.Equal("system", result.Namespace);
        Assert.Equal("auth-service", result.ServiceAccount);
    }

    [Fact]
    public void Parse_ValidUri_ToString_RoundTrips()
    {
        const string uri = "spiffe://example.com/ns/production/sa/web-server";
        var spiffe = SpiffeId.Parse(uri);

        Assert.Equal(uri, spiffe.ToString());
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    [InlineData("http://invalid/ns/system/sa/svc")]
    [InlineData("spiffe://")]
    [InlineData("spiffe://domain/wrong/format")]
    [InlineData("spiffe://domain/ns//sa/svc")]
    [InlineData("spiffe:///ns/system/sa/svc")]
    [InlineData("spiffe://domain/ns/system/sa/")]
    public void Parse_InvalidUri_ThrowsServiceAuthException(string uri)
    {
        var ex = Assert.Throws<ServiceAuthException>(() => SpiffeId.Parse(uri));
        Assert.Equal("InvalidSpiffeId", ex.Code);
    }

    [Fact]
    public void Parse_NullUri_ThrowsServiceAuthException()
    {
        var ex = Assert.Throws<ServiceAuthException>(() => SpiffeId.Parse(null!));
        Assert.Equal("InvalidSpiffeId", ex.Code);
    }
}
