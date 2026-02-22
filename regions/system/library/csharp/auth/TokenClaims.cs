namespace K1s0.System.Auth;

public sealed record TokenClaims
{
    public required string Sub { get; init; }

    public required string Iss { get; init; }

    public required string Aud { get; init; }

    public string? Scope { get; init; }

    public long Exp { get; init; }

    public long Iat { get; init; }

    public IReadOnlyList<string> Roles { get; init; } = [];

    public IReadOnlyDictionary<string, IReadOnlyList<string>> ResourceAccess { get; init; } =
        new Dictionary<string, IReadOnlyList<string>>();
}
