package encryption_test

import (
	"testing"

	"github.com/k1s0-platform/system-library-go-encryption"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGenerateKey(t *testing.T) {
	key, err := encryption.GenerateKey()
	require.NoError(t, err)
	assert.Len(t, key, 32)
}

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

func TestDecrypt_WrongKey(t *testing.T) {
	key1, _ := encryption.GenerateKey()
	key2, _ := encryption.GenerateKey()

	ciphertext, err := encryption.Encrypt(key1, []byte("secret"))
	require.NoError(t, err)

	_, err = encryption.Decrypt(key2, ciphertext)
	assert.Error(t, err)
}

func TestDecrypt_TamperedData(t *testing.T) {
	key, _ := encryption.GenerateKey()
	ciphertext, _ := encryption.Encrypt(key, []byte("data"))

	// Tamper with the ciphertext
	tampered := ciphertext[:len(ciphertext)-2] + "xx"
	_, err := encryption.Decrypt(key, tampered)
	assert.Error(t, err)
}

func TestHashPassword_And_Verify(t *testing.T) {
	hash, err := encryption.HashPassword("mypassword")
	require.NoError(t, err)
	assert.NotEmpty(t, hash)

	err = encryption.VerifyPassword("mypassword", hash)
	assert.NoError(t, err)
}

func TestVerifyPassword_Wrong(t *testing.T) {
	hash, _ := encryption.HashPassword("correct")
	err := encryption.VerifyPassword("wrong", hash)
	assert.Error(t, err)
}

func TestEncrypt_EmptyPlaintext(t *testing.T) {
	key, _ := encryption.GenerateKey()
	ciphertext, err := encryption.Encrypt(key, []byte{})
	require.NoError(t, err)

	decrypted, err := encryption.Decrypt(key, ciphertext)
	require.NoError(t, err)
	assert.Empty(t, decrypted)
}
