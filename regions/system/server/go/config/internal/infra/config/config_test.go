package config

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestLoad_ValidConfig(t *testing.T) {
	content := `
app:
  name: config-server
  version: "0.1.0"
  environment: test
  tier: system
server:
  port: 8082
  read_timeout: 10s
  write_timeout: 10s
  shutdown_timeout: 30s
grpc:
  port: 50053
database:
  host: localhost
  port: 5432
  user: postgres
  password: password
  dbname: config
  sslmode: disable
kafka:
  brokers:
    - localhost:9092
  topics:
    publish: k1s0.system.config.changed.v1
    subscribe: k1s0.system.config.changed.v1
`
	tmpDir := t.TempDir()
	tmpFile := filepath.Join(tmpDir, "config.yaml")
	err := os.WriteFile(tmpFile, []byte(content), 0644)
	assert.NoError(t, err)

	cfg, err := Load(tmpFile)

	assert.NoError(t, err)
	assert.NotNil(t, cfg)
	assert.Equal(t, "config-server", cfg.App.Name)
	assert.Equal(t, 8082, cfg.Server.Port)
	assert.Equal(t, 50053, cfg.GRPC.Port)
	assert.Equal(t, "localhost", cfg.Database.Host)
	assert.Equal(t, 5432, cfg.Database.Port)
	assert.Equal(t, "k1s0.system.config.changed.v1", cfg.Kafka.Topics.Publish)
}

func TestLoad_FileNotFound(t *testing.T) {
	cfg, err := Load("/nonexistent/config.yaml")

	assert.Error(t, err)
	assert.Nil(t, cfg)
}

func TestLoad_InvalidYAML(t *testing.T) {
	tmpDir := t.TempDir()
	tmpFile := filepath.Join(tmpDir, "config.yaml")
	err := os.WriteFile(tmpFile, []byte("invalid: [yaml: broken"), 0644)
	assert.NoError(t, err)

	cfg, err := Load(tmpFile)

	assert.Error(t, err)
	assert.Nil(t, cfg)
}

func TestConfig_Validate_Success(t *testing.T) {
	cfg := &Config{
		App:    AppConfig{Name: "config-server"},
		Server: ServerConfig{Port: 8082},
	}

	err := cfg.Validate()
	assert.NoError(t, err)
}

func TestConfig_Validate_MissingName(t *testing.T) {
	cfg := &Config{
		Server: ServerConfig{Port: 8082},
	}

	err := cfg.Validate()
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "app.name is required")
}

func TestConfig_Validate_InvalidPort(t *testing.T) {
	cfg := &Config{
		App:    AppConfig{Name: "config-server"},
		Server: ServerConfig{Port: 0},
	}

	err := cfg.Validate()
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "server.port must be positive")
}

func TestDatabaseConfig_DSN(t *testing.T) {
	dbConfig := &DatabaseConfig{
		Host:     "localhost",
		Port:     5432,
		User:     "postgres",
		Password: "password",
		DBName:   "config",
		SSLMode:  "disable",
	}

	dsn := dbConfig.DSN()
	assert.Equal(t, "host=localhost port=5432 user=postgres password=password dbname=config sslmode=disable", dsn)
}
