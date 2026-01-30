namespace K1s0.Auth.Jwt;

/// <summary>
/// Configuration for JWT token verification.
/// </summary>
/// <param name="Issuer">The expected token issuer (iss claim).</param>
/// <param name="JwksUri">The URI to fetch JSON Web Key Sets for signature verification.</param>
/// <param name="Audience">The expected token audience (aud claim).</param>
/// <param name="ClockSkewSeconds">Allowed clock skew in seconds for expiration checks. Defaults to 30.</param>
/// <param name="Algorithms">Allowed signing algorithms. Defaults to RS256.</param>
public record JwtVerifierConfig(
    string Issuer,
    string JwksUri,
    string Audience,
    int ClockSkewSeconds = 30,
    string[]? Algorithms = null)
{
    /// <summary>
    /// Gets the allowed signing algorithms, defaulting to RS256 if not specified.
    /// </summary>
    public string[] Algorithms { get; init; } = Algorithms ?? ["RS256"];
}
