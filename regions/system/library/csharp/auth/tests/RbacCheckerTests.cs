using Xunit;

namespace K1s0.System.Auth.Tests;

public class RbacCheckerTests
{
    [Fact]
    public void CheckPermission_AdminRole_ReturnsTrue()
    {
        var claims = new TokenClaims
        {
            Sub = "user1",
            Iss = "issuer",
            Aud = "audience",
            Roles = ["admin"],
        };

        Assert.True(RbacChecker.CheckPermission(claims, "orders", "read"));
    }

    [Fact]
    public void CheckPermission_MatchingScope_ReturnsTrue()
    {
        var claims = new TokenClaims
        {
            Sub = "user1",
            Iss = "issuer",
            Aud = "audience",
            Scope = "read write",
        };

        Assert.True(RbacChecker.CheckPermission(claims, "orders", "read"));
    }

    [Fact]
    public void CheckPermission_ResourceScopedAction_ReturnsTrue()
    {
        var claims = new TokenClaims
        {
            Sub = "user1",
            Iss = "issuer",
            Aud = "audience",
            Scope = "orders:read",
        };

        Assert.True(RbacChecker.CheckPermission(claims, "orders", "orders:read"));
    }

    [Fact]
    public void CheckPermission_ResourceAccessMatch_ReturnsTrue()
    {
        var claims = new TokenClaims
        {
            Sub = "user1",
            Iss = "issuer",
            Aud = "audience",
            ResourceAccess = new Dictionary<string, IReadOnlyList<string>>
            {
                ["orders"] = new List<string> { "read", "write" },
            },
        };

        Assert.True(RbacChecker.CheckPermission(claims, "orders", "read"));
    }

    [Fact]
    public void CheckPermission_ResourceAccessAdminRole_ReturnsTrue()
    {
        var claims = new TokenClaims
        {
            Sub = "user1",
            Iss = "issuer",
            Aud = "audience",
            ResourceAccess = new Dictionary<string, IReadOnlyList<string>>
            {
                ["orders"] = new List<string> { "admin" },
            },
        };

        Assert.True(RbacChecker.CheckPermission(claims, "orders", "anything"));
    }

    [Fact]
    public void CheckPermission_NoMatchingPermission_ReturnsFalse()
    {
        var claims = new TokenClaims
        {
            Sub = "user1",
            Iss = "issuer",
            Aud = "audience",
            Scope = "read",
        };

        Assert.False(RbacChecker.CheckPermission(claims, "orders", "delete"));
    }

    [Fact]
    public void CheckPermission_EmptyClaims_ReturnsFalse()
    {
        var claims = new TokenClaims
        {
            Sub = "user1",
            Iss = "issuer",
            Aud = "audience",
        };

        Assert.False(RbacChecker.CheckPermission(claims, "orders", "read"));
    }
}
