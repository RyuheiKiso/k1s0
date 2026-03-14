package encryption_test

import (
	"testing"

	"github.com/k1s0-platform/system-library-go-encryption"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// GenerateKeyが32バイトの暗号化キーを正常に生成することを確認する。
func TestGenerateKey(t *testing.T) {
	key, err := encryption.GenerateKey()
	require.NoError(t, err)
	assert.Len(t, key, 32)
}

// EncryptとDecryptが同一キーで暗号化・復号化のラウンドトリップを正常に行うことを確認する。
func TestEncryptDecrypt_RoundTrip(t *testing.T) {
	key, err := encryption.GenerateKey()
	require.NoError(t, err)

	plaintext := []byte("Hello, World!")
	ciphertext, err := encryption.Encrypt(key, plaintext)
	require.NoError(t, err)
	assert.NotEmpty(t, ciphertext)

	decrypted, err := encryption.Decrypt(key, ciphertext)
	require.NoError(t, err)
	assert.Equal(t, plaintext, decrypted)
}

// Decryptが異なるキーで暗号化されたデータの復号化に失敗することを確認する。
func TestDecrypt_WrongKey(t *testing.T) {
	key1, _ := encryption.GenerateKey()
	key2, _ := encryption.GenerateKey()

	ciphertext, err := encryption.Encrypt(key1, []byte("secret"))
	require.NoError(t, err)

	_, err = encryption.Decrypt(key2, ciphertext)
	assert.Error(t, err)
}

// Decryptが改ざんされた暗号文の復号化に失敗することを確認する。
func TestDecrypt_TamperedData(t *testing.T) {
	key, _ := encryption.GenerateKey()
	ciphertext, _ := encryption.Encrypt(key, []byte("data"))

	// Tamper with the ciphertext
	tampered := ciphertext[:len(ciphertext)-2] + "xx"
	_, err := encryption.Decrypt(key, tampered)
	assert.Error(t, err)
}

// HashPasswordがパスワードをハッシュ化し、VerifyPasswordが正しいパスワードを検証できることを確認する。
func TestHashPassword_And_Verify(t *testing.T) {
	hash, err := encryption.HashPassword("mypassword")
	require.NoError(t, err)
	assert.NotEmpty(t, hash)

	err = encryption.VerifyPassword("mypassword", hash)
	assert.NoError(t, err)
}

// VerifyPasswordが誤ったパスワードの検証に失敗することを確認する。
func TestVerifyPassword_Wrong(t *testing.T) {
	hash, _ := encryption.HashPassword("correct")
	err := encryption.VerifyPassword("wrong", hash)
	assert.Error(t, err)
}

// Encryptが空の平文を正常に暗号化し、復号化後も空データを返すことを確認する。
func TestEncrypt_EmptyPlaintext(t *testing.T) {
	key, _ := encryption.GenerateKey()
	ciphertext, err := encryption.Encrypt(key, []byte{})
	require.NoError(t, err)

	decrypted, err := encryption.Decrypt(key, ciphertext)
	require.NoError(t, err)
	assert.Empty(t, decrypted)
}

// RSA-OAEPで暗号化・復号化のラウンドトリップが正常に行われることを確認する。
func TestRSARoundtrip(t *testing.T) {
	pub, priv, err := encryption.GenerateRSAKeyPair()
	if err != nil {
		t.Fatal(err)
	}
	plaintext := []byte("hello RSA-OAEP")
	ciphertext, err := encryption.RSAEncrypt(pub, plaintext)
	if err != nil {
		t.Fatal(err)
	}
	decrypted, err := encryption.RSADecrypt(priv, ciphertext)
	if err != nil {
		t.Fatal(err)
	}
	if string(decrypted) != string(plaintext) {
		t.Errorf("got %s, want %s", decrypted, plaintext)
	}
}

// RSA復号化で異なる秘密鍵を使用した場合にエラーが返ることを確認する。
func TestRSAWrongKeyFails(t *testing.T) {
	pub, _, err := encryption.GenerateRSAKeyPair()
	if err != nil {
		t.Fatal(err)
	}
	_, priv2, err := encryption.GenerateRSAKeyPair()
	if err != nil {
		t.Fatal(err)
	}
	ciphertext, err := encryption.RSAEncrypt(pub, []byte("secret"))
	if err != nil {
		t.Fatal(err)
	}
	_, err = encryption.RSADecrypt(priv2, ciphertext)
	if err == nil {
		t.Error("expected error with wrong key")
	}
}

// RSAEncryptが無効なPEM文字列を受け取った場合にエラーを返すことを確認する。
func TestRSAEncryptInvalidPEM(t *testing.T) {
	_, err := encryption.RSAEncrypt("not-valid-pem", []byte("data"))
	if err == nil {
		t.Error("expected error with invalid PEM")
	}
}
