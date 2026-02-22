namespace K1s0.System.ServiceAuth;

public sealed record ServiceClaims(
    string Sub,
    string Iss,
    string Scope,
    long Exp,
    long Iat);
