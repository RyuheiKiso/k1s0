using System.Security.Cryptography;
using System.Text;

namespace K1s0.System.Encryption;

public static class AesEncryption
{
    private const int KeySize = 32;
    private const int NonceSize = 12;
    private const int TagSize = 16;

    public static byte[] GenerateKey()
    {
        return RandomNumberGenerator.GetBytes(KeySize);
    }

    public static string Encrypt(byte[] key, string plaintext)
    {
        var nonce = RandomNumberGenerator.GetBytes(NonceSize);
        var plaintextBytes = Encoding.UTF8.GetBytes(plaintext);
        var ciphertext = new byte[plaintextBytes.Length];
        var tag = new byte[TagSize];

        using var aes = new AesGcm(key, TagSize);
        aes.Encrypt(nonce, plaintextBytes, ciphertext, tag);

        var result = new byte[NonceSize + ciphertext.Length + TagSize];
        nonce.CopyTo(result, 0);
        ciphertext.CopyTo(result, NonceSize);
        tag.CopyTo(result, NonceSize + ciphertext.Length);

        return Convert.ToBase64String(result);
    }

    public static string Decrypt(byte[] key, string ciphertextBase64)
    {
        var data = Convert.FromBase64String(ciphertextBase64);
        var nonce = data[..NonceSize];
        var ciphertext = data[NonceSize..^TagSize];
        var tag = data[^TagSize..];
        var plaintext = new byte[ciphertext.Length];

        using var aes = new AesGcm(key, TagSize);
        aes.Decrypt(nonce, ciphertext, tag, plaintext);

        return Encoding.UTF8.GetString(plaintext);
    }
}
