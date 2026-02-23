using System.Security.Cryptography;

namespace K1s0.System.Encryption;

public static class PasswordHasher
{
    private const int SaltSize = 16;
    private const int HashSize = 32;
    private const int Iterations = 100_000;

    public static string Hash(string password)
    {
        var salt = RandomNumberGenerator.GetBytes(SaltSize);
        var hash = Rfc2898DeriveBytes.Pbkdf2(
            password,
            salt,
            Iterations,
            HashAlgorithmName.SHA256,
            HashSize);

        var result = new byte[SaltSize + HashSize];
        salt.CopyTo(result, 0);
        hash.CopyTo(result, SaltSize);

        return Convert.ToBase64String(result);
    }

    public static bool Verify(string password, string hashString)
    {
        var data = Convert.FromBase64String(hashString);
        var salt = data[..SaltSize];
        var expectedHash = data[SaltSize..];

        var actualHash = Rfc2898DeriveBytes.Pbkdf2(
            password,
            salt,
            Iterations,
            HashAlgorithmName.SHA256,
            HashSize);

        return CryptographicOperations.FixedTimeEquals(expectedHash, actualHash);
    }
}
