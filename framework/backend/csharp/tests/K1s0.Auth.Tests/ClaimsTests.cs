using FluentAssertions;
using K1s0.Auth.Jwt;

namespace K1s0.Auth.Tests;

public class ClaimsTests
{
    private static Claims CreateClaims(
        IReadOnlyList<string>? roles = null,
        IReadOnlyList<string>? permissions = null) =>
        new(
            Sub: "user-1",
            Roles: roles ?? Array.Empty<string>(),
            Permissions: permissions ?? Array.Empty<string>(),
            Groups: Array.Empty<string>(),
            TenantId: null,
            Custom: new Dictionary<string, object>());

    [Theory]
    [InlineData("admin", true)]
    [InlineData("Admin", true)]
    [InlineData("unknown", false)]
    public void HasRole_ReturnsExpected(string role, bool expected)
    {
        var claims = CreateClaims(roles: new[] { "admin", "user" });
        claims.HasRole(role).Should().Be(expected);
    }

    [Theory]
    [InlineData("read:users", true)]
    [InlineData("Read:Users", true)]
    [InlineData("delete:users", false)]
    public void HasPermission_ReturnsExpected(string permission, bool expected)
    {
        var claims = CreateClaims(permissions: new[] { "read:users", "write:users" });
        claims.HasPermission(permission).Should().Be(expected);
    }

    [Fact]
    public void HasAnyRole_WithMatchingRole_ReturnsTrue()
    {
        var claims = CreateClaims(roles: new[] { "viewer" });
        claims.HasAnyRole(new[] { "admin", "viewer" }).Should().BeTrue();
    }

    [Fact]
    public void HasAnyRole_WithNoMatchingRole_ReturnsFalse()
    {
        var claims = CreateClaims(roles: new[] { "viewer" });
        claims.HasAnyRole(new[] { "admin", "editor" }).Should().BeFalse();
    }
}
