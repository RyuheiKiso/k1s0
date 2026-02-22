package handler

import (
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
		name      string
		byteLen   int
		hexLen    int
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
