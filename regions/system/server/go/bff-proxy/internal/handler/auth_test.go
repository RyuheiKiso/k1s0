package handler

import (
	"encoding/base64"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGenerateRandomString(t *testing.T) {
	s, err := generateRandomString(32)
	require.NoError(t, err)
	assert.Len(t, s, 64) // 32 bytes = 64 hex characters

	// Should be unique.
	s2, err := generateRandomString(32)
	require.NoError(t, err)
	assert.NotEqual(t, s, s2)
}

func TestGenerateRandomString_DifferentLengths(t *testing.T) {
	tests := []struct {
		name    string
		byteLen int
		hexLen  int
	}{
		{"16 bytes", 16, 32},
		{"32 bytes", 32, 64},
		{"64 bytes", 64, 128},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s, err := generateRandomString(tt.byteLen)
			require.NoError(t, err)
			assert.Len(t, s, tt.hexLen)
		})
	}
}

func TestExtractSubjectFromIDToken(t *testing.T) {
	header := base64.RawURLEncoding.EncodeToString([]byte(`{"alg":"none","typ":"JWT"}`))
	payload := base64.RawURLEncoding.EncodeToString([]byte(`{"sub":"user-123"}`))
	token := header + "." + payload + "."

	subject, err := extractSubjectFromIDToken(token)
	require.NoError(t, err)
	assert.Equal(t, "user-123", subject)
}

func TestExtractSubjectFromIDToken_MissingSubject(t *testing.T) {
	header := base64.RawURLEncoding.EncodeToString([]byte(`{"alg":"none","typ":"JWT"}`))
	payload := base64.RawURLEncoding.EncodeToString([]byte(`{"aud":"client"}`))
	token := header + "." + payload + "."

	_, err := extractSubjectFromIDToken(token)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "sub claim is missing")
}
