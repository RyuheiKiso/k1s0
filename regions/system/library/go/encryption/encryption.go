package encryption

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"errors"
	"fmt"
	"io"

	"golang.org/x/crypto/argon2"
)

// GenerateKey は32バイトのランダムキーを生成する。
func GenerateKey() ([]byte, error) {
	key := make([]byte, 32)
	if _, err := io.ReadFull(rand.Reader, key); err != nil {
		return nil, err
	}
	return key, nil
}

// Encrypt はAES-256-GCMで暗号化し、Base64エンコードした文字列を返す。
func Encrypt(key, plaintext []byte) (string, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return "", err
	}
	aead, err := cipher.NewGCM(block)
	if err != nil {
		return "", err
	}
	nonce := make([]byte, aead.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return "", err
	}
	ciphertext := aead.Seal(nonce, nonce, plaintext, nil)
	return base64.StdEncoding.EncodeToString(ciphertext), nil
}

// Decrypt はBase64デコード後、AES-256-GCMで復号する。
func Decrypt(key []byte, ciphertext string) ([]byte, error) {
	data, err := base64.StdEncoding.DecodeString(ciphertext)
	if err != nil {
		return nil, err
	}
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	aead, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}
	nonceSize := aead.NonceSize()
	if len(data) < nonceSize {
		return nil, errors.New("暗号文が短すぎます")
	}
	nonce, sealed := data[:nonceSize], data[nonceSize:]
	return aead.Open(nil, nonce, sealed, nil)
}

// Argon2id parameters
const (
	argon2Memory     = 19456
	argon2Iterations = 2
	argon2Parallelism = 1
	argon2KeyLen     = 32
	argon2SaltLen    = 16
)

// HashPassword はArgon2idでパスワードをハッシュ化する。
// 出力形式: $argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>
func HashPassword(password string) (string, error) {
	salt := make([]byte, argon2SaltLen)
	if _, err := io.ReadFull(rand.Reader, salt); err != nil {
		return "", err
	}

	hash := argon2.IDKey([]byte(password), salt, argon2Iterations, argon2Memory, argon2Parallelism, argon2KeyLen)

	saltB64 := base64.RawStdEncoding.EncodeToString(salt)
	hashB64 := base64.RawStdEncoding.EncodeToString(hash)

	return fmt.Sprintf("$argon2id$v=%d$m=%d,t=%d,p=%d$%s$%s",
		argon2.Version, argon2Memory, argon2Iterations, argon2Parallelism, saltB64, hashB64), nil
}

// VerifyPassword はパスワードとArgon2idハッシュを検証する。
func VerifyPassword(password, encodedHash string) error {
	parts := splitArgon2Hash(encodedHash)
	if parts == nil {
		return errors.New("invalid argon2id hash format")
	}

	salt, err := base64.RawStdEncoding.DecodeString(parts.salt)
	if err != nil {
		return fmt.Errorf("failed to decode salt: %w", err)
	}

	expectedHash, err := base64.RawStdEncoding.DecodeString(parts.hash)
	if err != nil {
		return fmt.Errorf("failed to decode hash: %w", err)
	}

	computed := argon2.IDKey([]byte(password), salt, parts.iterations, parts.memory, parts.parallelism, uint32(len(expectedHash)))

	if subtle.ConstantTimeCompare(computed, expectedHash) != 1 {
		return errors.New("password does not match")
	}
	return nil
}

type argon2Params struct {
	memory      uint32
	iterations  uint32
	parallelism uint8
	salt        string
	hash        string
}

func splitArgon2Hash(encoded string) *argon2Params {
	var version int
	var memory uint32
	var iterations uint32
	var parallelism uint8
	var salt, hash string

	// Format: $argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>
	n, err := fmt.Sscanf(encoded, "$argon2id$v=%d$m=%d,t=%d,p=%d$%s",
		&version, &memory, &iterations, &parallelism, &salt)
	if err != nil || n != 5 {
		return nil
	}

	// salt$hash が一つの文字列として読まれるので分割する
	for i := len(salt) - 1; i >= 0; i-- {
		if salt[i] == '$' {
			hash = salt[i+1:]
			salt = salt[:i]
			break
		}
	}

	if hash == "" {
		return nil
	}

	return &argon2Params{
		memory:      memory,
		iterations:  iterations,
		parallelism: parallelism,
		salt:        salt,
		hash:        hash,
	}
}
