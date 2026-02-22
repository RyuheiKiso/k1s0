namespace K1s0.System.Auth;

public interface IJwksVerifier
{
    Task<TokenClaims> VerifyTokenAsync(string token, CancellationToken ct = default);
}
