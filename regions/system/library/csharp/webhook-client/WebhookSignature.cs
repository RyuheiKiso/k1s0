using System.Security.Cryptography;
using System.Text;

namespace K1s0.System.WebhookClient;

public static class WebhookSignature
{
    public static string Generate(string secret, byte[] body)
    {
        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(secret));
        var hash = hmac.ComputeHash(body);
        return Convert.ToHexStringLower(hash);
    }

    public static bool Verify(string secret, byte[] body, string signature)
    {
        var expected = Generate(secret, body);
        return CryptographicOperations.FixedTimeEquals(
            Encoding.UTF8.GetBytes(expected),
            Encoding.UTF8.GetBytes(signature));
    }
}
