using System.Security.Cryptography;
using System.Text;

using Konscious.Security.Cryptography;

namespace K1s0.System.Encryption;

public static class PasswordHasher
{
    private const int SaltSize = 16;
    private const int HashSize = 32;
    private const int MemorySize = 19456; // KiB
    private const int Iterations = 2;
    private const int DegreeOfParallelism = 1;

    /// <summary>
    /// Hash a password with Argon2id.
    /// Returns a PHC-format string: $argon2id$v=19$m=19456,t=2,p=1$&lt;salt_base64&gt;$&lt;hash_base64&gt;
    /// </summary>
    public static string Hash(string password)
    {
        var salt = RandomNumberGenerator.GetBytes(SaltSize);
        var hash = ComputeArgon2Id(password, salt, HashSize);

        var saltB64 = ToBase64NoPadding(salt);
        var hashB64 = ToBase64NoPadding(hash);

        return $"$argon2id$v=19$m={MemorySize},t={Iterations},p={DegreeOfParallelism}${saltB64}${hashB64}";
    }

    /// <summary>
    /// Verify a password against an Argon2id hash string.
    /// </summary>
    public static bool Verify(string password, string hashString)
    {
        try
        {
            var parts = hashString.Split('$');
            // Format: ["", "argon2id", "v=19", "m=19456,t=2,p=1", "<salt>", "<hash>"]
            if (parts.Length != 6 || parts[1] != "argon2id")
                return false;

            var paramParts = parts[3].Split(',');
            var memory = int.Parse(paramParts[0][2..]);    // m=...
            var timeCost = int.Parse(paramParts[1][2..]);   // t=...
            var parallelism = int.Parse(paramParts[2][2..]); // p=...

            var salt = FromBase64NoPadding(parts[4]);
            var expectedHash = FromBase64NoPadding(parts[5]);

            var computedHash = ComputeArgon2Id(password, salt, expectedHash.Length, memory, timeCost, parallelism);

            return CryptographicOperations.FixedTimeEquals(expectedHash, computedHash);
        }
        catch
        {
            return false;
        }
    }

    private static byte[] ComputeArgon2Id(
        string password,
        byte[] salt,
        int hashSize,
        int memory = MemorySize,
        int iterations = Iterations,
        int parallelism = DegreeOfParallelism)
    {
        var argon2 = new Argon2id(Encoding.UTF8.GetBytes(password))
        {
            Salt = salt,
            MemorySize = memory,
            Iterations = iterations,
            DegreeOfParallelism = parallelism,
        };

        return argon2.GetBytes(hashSize);
    }

    private static string ToBase64NoPadding(byte[] data) =>
        Convert.ToBase64String(data).TrimEnd('=');

    private static byte[] FromBase64NoPadding(string s)
    {
        var padded = s.Length % 4 switch
        {
            2 => s + "==",
            3 => s + "=",
            _ => s,
        };
        return Convert.FromBase64String(padded);
    }
}
