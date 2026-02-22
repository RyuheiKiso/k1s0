namespace K1s0.System.Auth;

public sealed record AuthConfig
{
    public required string JwksUrl { get; init; }

    public required string Issuer { get; init; }

    public required string Audience { get; init; }

    public int CacheTtlSeconds { get; init; } = 300;
}
