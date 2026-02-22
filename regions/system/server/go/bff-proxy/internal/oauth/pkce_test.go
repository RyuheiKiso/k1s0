package oauth

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewPKCE(t *testing.T) {
	pkce, err := NewPKCE()
	require.NoError(t, err)

	// Verifier should be 43 characters (32 bytes base64url without padding).
	assert.Len(t, pkce.CodeVerifier, 43)

	// Challenge should be 43 characters (SHA256 hash base64url without padding).
	assert.Len(t, pkce.CodeChallenge, 43)

	// Verifier and challenge should be different.
	assert.NotEqual(t, pkce.CodeVerifier, pkce.CodeChallenge)
}

func TestNewPKCE_Uniqueness(t *testing.T) {
	pkce1, err := NewPKCE()
	require.NoError(t, err)

	pkce2, err := NewPKCE()
	require.NoError(t, err)

	assert.NotEqual(t, pkce1.CodeVerifier, pkce2.CodeVerifier)
	assert.NotEqual(t, pkce1.CodeChallenge, pkce2.CodeChallenge)
}

func TestComputeCodeChallenge_Deterministic(t *testing.T) {
	verifier := "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
	challenge1 := computeCodeChallenge(verifier)
	challenge2 := computeCodeChallenge(verifier)

	assert.Equal(t, challenge1, challenge2)
}
