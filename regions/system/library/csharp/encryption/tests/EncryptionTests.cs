using K1s0.System.Encryption;

namespace K1s0.System.Encryption.Tests;

public class AesEncryptionTests
{
    [Fact]
    public void GenerateKey_Returns32Bytes()
    {
        var key = AesEncryption.GenerateKey();
        Assert.Equal(32, key.Length);
    }

    [Fact]
    public void EncryptDecrypt_Roundtrip()
    {
        var key = AesEncryption.GenerateKey();
        var plaintext = "Hello, World!";

        var encrypted = AesEncryption.Encrypt(key, plaintext);
        var decrypted = AesEncryption.Decrypt(key, encrypted);

        Assert.Equal(plaintext, decrypted);
    }

    [Fact]
    public void Encrypt_ProducesDifferentCiphertext_EachTime()
    {
        var key = AesEncryption.GenerateKey();
        var plaintext = "same input";

        var a = AesEncryption.Encrypt(key, plaintext);
        var b = AesEncryption.Encrypt(key, plaintext);

        Assert.NotEqual(a, b);
    }

    [Fact]
    public void Decrypt_WrongKey_Throws()
    {
        var key1 = AesEncryption.GenerateKey();
        var key2 = AesEncryption.GenerateKey();
        var encrypted = AesEncryption.Encrypt(key1, "secret");

        Assert.ThrowsAny<Exception>(() => AesEncryption.Decrypt(key2, encrypted));
    }
}

public class PasswordHasherTests
{
    [Fact]
    public void Hash_ProducesNonEmptyString()
    {
        var hash = PasswordHasher.Hash("password123");
        Assert.False(string.IsNullOrEmpty(hash));
    }

    [Fact]
    public void Verify_CorrectPassword_ReturnsTrue()
    {
        var hash = PasswordHasher.Hash("mypassword");
        Assert.True(PasswordHasher.Verify("mypassword", hash));
    }

    [Fact]
    public void Verify_WrongPassword_ReturnsFalse()
    {
        var hash = PasswordHasher.Hash("mypassword");
        Assert.False(PasswordHasher.Verify("wrongpassword", hash));
    }

    [Fact]
    public void Hash_DifferentSalts_ProduceDifferentHashes()
    {
        var a = PasswordHasher.Hash("same");
        var b = PasswordHasher.Hash("same");
        Assert.NotEqual(a, b);
    }
}
