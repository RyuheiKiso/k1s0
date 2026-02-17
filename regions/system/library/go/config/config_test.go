package config

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestLoad(t *testing.T) {
	dir := t.TempDir()
	base := filepath.Join(dir, "config.yaml")
	os.WriteFile(base, []byte(`
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
`), 0644)

	cfg, err := Load(base)
	require.NoError(t, err)
	assert.Equal(t, "test-server", cfg.App.Name)
	assert.Equal(t, 8080, cfg.Server.Port)
}

func TestLoad_FileNotFound(t *testing.T) {
	_, err := Load("/nonexistent/config.yaml")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to read config")
}

func TestLoad_InvalidYAML(t *testing.T) {
	dir := t.TempDir()
	base := filepath.Join(dir, "config.yaml")
	os.WriteFile(base, []byte(`invalid: [yaml: broken`), 0644)

	_, err := Load(base)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to parse config")
}

func TestLoad_WithEnvOverride(t *testing.T) {
	dir := t.TempDir()
	base := filepath.Join(dir, "config.yaml")
	os.WriteFile(base, []byte(`
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
`), 0644)

	envFile := filepath.Join(dir, "config.staging.yaml")
	os.WriteFile(envFile, []byte(`
app:
  environment: staging
server:
  port: 9090
observability:
  log:
    level: info
`), 0644)

	cfg, err := Load(base, envFile)
	require.NoError(t, err)
	assert.Equal(t, "staging", cfg.App.Environment)
	assert.Equal(t, 9090, cfg.Server.Port)
	assert.Equal(t, "test-server", cfg.App.Name) // base value preserved
}

func TestLoad_WithEmptyEnvPath(t *testing.T) {
	dir := t.TempDir()
	base := filepath.Join(dir, "config.yaml")
	os.WriteFile(base, []byte(`
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
`), 0644)

	cfg, err := Load(base, "")
	require.NoError(t, err)
	assert.Equal(t, "test-server", cfg.App.Name)
}

func TestValidate_ValidConfig(t *testing.T) {
	dir := t.TempDir()
	base := filepath.Join(dir, "config.yaml")
	os.WriteFile(base, []byte(`
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
`), 0644)

	cfg, err := Load(base)
	require.NoError(t, err)

	err = cfg.Validate()
	assert.NoError(t, err)
}

func TestValidate_MissingRequired(t *testing.T) {
	cfg := &Config{}
	err := cfg.Validate()
	assert.Error(t, err)
}

func TestValidate_InvalidTier(t *testing.T) {
	cfg := &Config{
		App: AppConfig{
			Name:        "test",
			Version:     "1.0.0",
			Tier:        "invalid",
			Environment: "dev",
		},
		Server: ServerConfig{
			Host: "0.0.0.0",
			Port: 8080,
		},
		Observability: ObservabilityConfig{
			Log: LogConfig{Level: "info", Format: "json"},
		},
		Auth: AuthConfig{
			JWT: JWTConfig{
				Issuer:   "http://localhost",
				Audience: "test",
			},
		},
	}
	err := cfg.Validate()
	assert.Error(t, err)
}

func TestValidate_InvalidPort(t *testing.T) {
	cfg := &Config{
		App: AppConfig{
			Name:        "test",
			Version:     "1.0.0",
			Tier:        "system",
			Environment: "dev",
		},
		Server: ServerConfig{
			Host: "0.0.0.0",
			Port: 0,
		},
		Observability: ObservabilityConfig{
			Log: LogConfig{Level: "info", Format: "json"},
		},
		Auth: AuthConfig{
			JWT: JWTConfig{
				Issuer:   "http://localhost",
				Audience: "test",
			},
		},
	}
	err := cfg.Validate()
	assert.Error(t, err)
}

func TestMergeVaultSecrets(t *testing.T) {
	cfg := &Config{
		Database: &DatabaseConfig{Password: ""},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"database.password": "secret123",
	})
	assert.Equal(t, "secret123", cfg.Database.Password)
}

func TestMergeVaultSecrets_RedisPassword(t *testing.T) {
	cfg := &Config{
		Redis: &RedisConfig{Password: ""},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"redis.password": "redis-secret",
	})
	assert.Equal(t, "redis-secret", cfg.Redis.Password)
}

func TestMergeVaultSecrets_KafkaSASL(t *testing.T) {
	cfg := &Config{
		Kafka: &KafkaConfig{
			SASL: &KafkaSASLConfig{},
		},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"kafka.sasl.username": "kafka-user",
		"kafka.sasl.password": "kafka-pass",
	})
	assert.Equal(t, "kafka-user", cfg.Kafka.SASL.Username)
	assert.Equal(t, "kafka-pass", cfg.Kafka.SASL.Password)
}

func TestMergeVaultSecrets_OIDCClientSecret(t *testing.T) {
	cfg := &Config{
		Auth: AuthConfig{
			OIDC: &OIDCConfig{},
		},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"auth.oidc.client_secret": "oidc-secret",
	})
	assert.Equal(t, "oidc-secret", cfg.Auth.OIDC.ClientSecret)
}

func TestMergeVaultSecrets_NilOptionalFields(t *testing.T) {
	cfg := &Config{}
	// Should not panic when optional fields are nil
	cfg.MergeVaultSecrets(map[string]string{
		"database.password":       "secret",
		"redis.password":          "secret",
		"kafka.sasl.username":     "user",
		"kafka.sasl.password":     "pass",
		"auth.oidc.client_secret": "secret",
	})
	assert.Nil(t, cfg.Database)
	assert.Nil(t, cfg.Redis)
	assert.Nil(t, cfg.Kafka)
	assert.Nil(t, cfg.Auth.OIDC)
}

func TestLoad_FullConfig(t *testing.T) {
	dir := t.TempDir()
	base := filepath.Join(dir, "config.yaml")
	os.WriteFile(base, []byte(`
app:
  name: order-server
  version: "1.0.0"
  tier: service
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"
grpc:
  port: 50051
  max_recv_msg_size: 4194304
database:
  host: "localhost"
  port: 5432
  name: "order_db"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
kafka:
  brokers:
    - "localhost:9092"
  consumer_group: "order-server.default"
  security_protocol: "PLAINTEXT"
  topics:
    publish:
      - "k1s0.service.order.created.v1"
    subscribe:
      - "k1s0.service.payment.completed.v1"
redis:
  host: "localhost"
  port: 6379
  password: ""
  db: 0
  pool_size: 10
observability:
  log:
    level: info
    format: json
  trace:
    enabled: true
    endpoint: "localhost:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
  oidc:
    discovery_url: "http://localhost:8180/realms/k1s0/.well-known/openid-configuration"
    client_id: "k1s0-bff"
    client_secret: ""
    redirect_uri: "http://localhost:3000/callback"
    scopes:
      - "openid"
      - "profile"
    jwks_uri: "http://localhost:8180/realms/k1s0/protocol/openid-connect/certs"
    jwks_cache_ttl: "10m"
`), 0644)

	cfg, err := Load(base)
	require.NoError(t, err)

	assert.Equal(t, "order-server", cfg.App.Name)
	assert.Equal(t, "service", cfg.App.Tier)
	assert.NotNil(t, cfg.GRPC)
	assert.Equal(t, 50051, cfg.GRPC.Port)
	assert.NotNil(t, cfg.Database)
	assert.Equal(t, "order_db", cfg.Database.Name)
	assert.NotNil(t, cfg.Kafka)
	assert.Equal(t, "PLAINTEXT", cfg.Kafka.SecurityProtocol)
	assert.NotNil(t, cfg.Redis)
	assert.Equal(t, 6379, cfg.Redis.Port)
	assert.NotNil(t, cfg.Auth.OIDC)
	assert.Equal(t, "k1s0-bff", cfg.Auth.OIDC.ClientID)

	err = cfg.Validate()
	assert.NoError(t, err)
}
