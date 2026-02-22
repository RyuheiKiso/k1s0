namespace K1s0.System.ServiceAuth;

public sealed record ServiceAuthConfig(
    string TokenUrl,
    string ClientId,
    string ClientSecret,
    string? JwksUri = null,
    string[]? Scopes = null);
