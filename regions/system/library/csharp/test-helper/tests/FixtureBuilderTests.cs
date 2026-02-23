using K1s0.System.TestHelper.Fixtures;
using Xunit;

namespace K1s0.System.TestHelper.Tests;

public class FixtureBuilderTests
{
    [Fact]
    public void Uuid_ReturnsValidFormat()
    {
        var id = FixtureBuilder.Uuid();
        Assert.Equal(36, id.Length);
        Assert.Contains("-", id);
    }

    [Fact]
    public void Email_ContainsDomain()
    {
        var email = FixtureBuilder.Email();
        Assert.Contains("@example.com", email);
    }

    [Fact]
    public void Name_HasPrefix()
    {
        var name = FixtureBuilder.Name();
        Assert.StartsWith("user-", name);
    }

    [Fact]
    public void Int_InRange()
    {
        for (var i = 0; i < 100; i++)
        {
            var val = FixtureBuilder.Int(10, 20);
            Assert.InRange(val, 10, 19);
        }
    }

    [Fact]
    public void Int_SameMinMax_ReturnsMin()
    {
        Assert.Equal(5, FixtureBuilder.Int(5, 5));
    }

    [Fact]
    public void TenantId_HasPrefix()
    {
        var tid = FixtureBuilder.TenantId();
        Assert.StartsWith("tenant-", tid);
    }

    [Fact]
    public void Uuid_IsUnique()
    {
        var a = FixtureBuilder.Uuid();
        var b = FixtureBuilder.Uuid();
        Assert.NotEqual(a, b);
    }
}
