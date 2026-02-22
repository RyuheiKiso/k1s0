package config

import (
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestLoad(t *testing.T) {
	yaml := `
app:
  name: bff-proxy
  version: "0.1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: info
    format: json
auth:
  discovery_url: "http://keycloak:8080/realms/k1s0"
  client_id: "k1s0-bff"
  redirect_uri: "http://localhost:8080/auth/callback"
session:
  redis:
    addr: "redis:6379"
  ttl: "30m"
upstream:
  base_url: "http://auth-server:8080"
`
	dir := t.TempDir()
	cfgPath := filepath.Join(dir, "config.yaml")
	err := os.WriteFile(cfgPath, []byte(yaml), 0644)
	require.NoError(t, err)

	cfg, err := Load(cfgPath)
	require.NoError(t, err)
	assert.Equal(t, "bff-proxy", cfg.App.Name)
	assert.Equal(t, 8080, cfg.Server.Port)
	assert.Equal(t, "k1s0-bff", cfg.Auth.ClientID)
	assert.Equal(t, "redis:6379", cfg.Session.Redis.Addr)
	assert.Equal(t, "http://auth-server:8080", cfg.Upstream.BaseURL)
}

func TestLoad_NotFound(t *testing.T) {
	_, err := Load("/nonexistent/config.yaml")
	assert.Error(t, err)
}

func TestParseDuration(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		fallback time.Duration
		expected time.Duration
	}{
		{"valid", "30m", 5 * time.Minute, 30 * time.Minute},
		{"empty", "", 5 * time.Minute, 5 * time.Minute},
		{"invalid", "not-a-duration", 10 * time.Second, 10 * time.Second},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ParseDuration(tt.input, tt.fallback)
			assert.Equal(t, tt.expected, got)
		})
	}
}
