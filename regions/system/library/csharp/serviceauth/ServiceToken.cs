namespace K1s0.System.ServiceAuth;

public sealed record ServiceToken(
    string AccessToken,
    DateTimeOffset ExpiresAt,
    string TokenType = "Bearer")
{
    public bool IsExpired => DateTimeOffset.UtcNow >= ExpiresAt;

    public bool IsNearExpiry => DateTimeOffset.UtcNow >= ExpiresAt.AddSeconds(-60);
}
