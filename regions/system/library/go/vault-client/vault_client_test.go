package vaultclient_test

import (
	"context"
	"testing"
	"time"

	vaultclient "github.com/k1s0-platform/system-library-go-vault-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func makeConfig() vaultclient.VaultClientConfig {
	return vaultclient.VaultClientConfig{
		ServerURL:        "http://localhost:8080",
		CacheTTL:         10 * time.Minute,
		CacheMaxCapacity: 500,
	}
}

func makeSecret(path string) vaultclient.Secret {
	return vaultclient.Secret{
		Path: path,
		Data: map[string]string{
			"password": "s3cr3t",
			"username": "admin",
		},
		Version:   1,
		CreatedAt: time.Now(),
	}
}

func TestGetSecret_Found(t *testing.T) {
	c := vaultclient.NewInMemoryVaultClient(makeConfig())
	c.PutSecret(makeSecret("system/db/primary"))
	s, err := c.GetSecret(context.Background(), "system/db/primary")
	require.NoError(t, err)
	assert.Equal(t, "system/db/primary", s.Path)
	assert.Equal(t, "s3cr3t", s.Data["password"])
}

func TestGetSecret_NotFound(t *testing.T) {
	c := vaultclient.NewInMemoryVaultClient(makeConfig())
	_, err := c.GetSecret(context.Background(), "missing/path")
	require.Error(t, err)
	var ve *vaultclient.VaultError
	require.ErrorAs(t, err, &ve)
	assert.Equal(t, "NOT_FOUND", ve.Code)
}

func TestGetSecretValue_Found(t *testing.T) {
	c := vaultclient.NewInMemoryVaultClient(makeConfig())
	c.PutSecret(makeSecret("system/db"))
	v, err := c.GetSecretValue(context.Background(), "system/db", "password")
	require.NoError(t, err)
	assert.Equal(t, "s3cr3t", v)
}

func TestGetSecretValue_KeyNotFound(t *testing.T) {
	c := vaultclient.NewInMemoryVaultClient(makeConfig())
	c.PutSecret(makeSecret("system/db"))
	_, err := c.GetSecretValue(context.Background(), "system/db", "missing_key")
	require.Error(t, err)
}

func TestListSecrets(t *testing.T) {
	c := vaultclient.NewInMemoryVaultClient(makeConfig())
	c.PutSecret(makeSecret("system/db/primary"))
	c.PutSecret(makeSecret("system/db/replica"))
	c.PutSecret(makeSecret("business/api/key"))

	paths, err := c.ListSecrets(context.Background(), "system/")
	require.NoError(t, err)
	assert.Len(t, paths, 2)
}

func TestListSecrets_Empty(t *testing.T) {
	c := vaultclient.NewInMemoryVaultClient(makeConfig())
	paths, err := c.ListSecrets(context.Background(), "nothing/")
	require.NoError(t, err)
	assert.Empty(t, paths)
}

func TestWatchSecret_ReturnsChannel(t *testing.T) {
	c := vaultclient.NewInMemoryVaultClient(makeConfig())
	ch, err := c.WatchSecret(context.Background(), "system/db")
	require.NoError(t, err)
	assert.NotNil(t, ch)
}

func TestVaultError_Format(t *testing.T) {
	err := vaultclient.NewNotFoundError("system/missing")
	assert.Equal(t, "NOT_FOUND: system/missing", err.Error())
}

func TestVaultError_PermissionDenied(t *testing.T) {
	err := vaultclient.NewPermissionDeniedError("system/secret")
	assert.Equal(t, "PERMISSION_DENIED", err.Code)
}

func TestSecret_Fields(t *testing.T) {
	s := makeSecret("test/path")
	assert.Equal(t, "test/path", s.Path)
	assert.Equal(t, int64(1), s.Version)
	assert.Equal(t, "admin", s.Data["username"])
}
