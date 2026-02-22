package oauth

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
)

// PKCE holds code_verifier and code_challenge for RFC 7636.
type PKCE struct {
	CodeVerifier  string
	CodeChallenge string
}

// NewPKCE generates a new PKCE pair using S256 method.
func NewPKCE() (*PKCE, error) {
	verifier, err := generateCodeVerifier()
	if err != nil {
		return nil, err
	}

	challenge := computeCodeChallenge(verifier)

	return &PKCE{
		CodeVerifier:  verifier,
		CodeChallenge: challenge,
	}, nil
}

// generateCodeVerifier generates a cryptographically random 32-byte code verifier
// encoded as URL-safe base64 without padding (43 characters).
func generateCodeVerifier() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return base64.RawURLEncoding.EncodeToString(b), nil
}

// computeCodeChallenge computes the S256 code challenge from a verifier.
func computeCodeChallenge(verifier string) string {
	h := sha256.Sum256([]byte(verifier))
	return base64.RawURLEncoding.EncodeToString(h[:])
}
