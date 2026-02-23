namespace K1s0.System.TestHelper.Jwt;

/// <summary>テスト用 JWT クレーム。</summary>
public record TestClaims(
    string Sub,
    IReadOnlyList<string> Roles,
    string? TenantId = null,
    TimeSpan? Expiry = null);
