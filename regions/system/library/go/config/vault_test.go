package config

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestVault_DatabasePasswordMerge(t *testing.T) {
	cfg := &Config{
		Database: &DatabaseConfig{Password: "old"},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"database.password": "vault-db-pass",
	})
	assert.Equal(t, "vault-db-pass", cfg.Database.Password)
}

func TestVault_RedisPasswordMerge(t *testing.T) {
	cfg := &Config{
		Redis: &RedisConfig{Password: "old"},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"redis.password": "vault-redis-pass",
	})
	assert.Equal(t, "vault-redis-pass", cfg.Redis.Password)
}

func TestVault_KafkaSASLMerge(t *testing.T) {
	cfg := &Config{
		Kafka: &KafkaConfig{
			SASL: &KafkaSASLConfig{
				Mechanism: "SCRAM-SHA-512",
				Username:  "",
				Password:  "",
			},
		},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"kafka.sasl.username": "vault-kafka-user",
		"kafka.sasl.password": "vault-kafka-pass",
	})
	assert.Equal(t, "vault-kafka-user", cfg.Kafka.SASL.Username)
	assert.Equal(t, "vault-kafka-pass", cfg.Kafka.SASL.Password)
}

func TestVault_RedisSessionPasswordMerge(t *testing.T) {
	cfg := &Config{
		RedisSession: &RedisConfig{Password: ""},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"redis_session.password": "vault-session-pass",
	})
	assert.Equal(t, "vault-session-pass", cfg.RedisSession.Password)
}

func TestVault_OIDCClientSecretMerge(t *testing.T) {
	cfg := &Config{
		Auth: AuthConfig{
			OIDC: &OIDCConfig{
				DiscoveryURL: "http://localhost/.well-known",
				ClientID:     "test",
				RedirectURI:  "http://localhost/callback",
				JWKSURI:      "http://localhost/jwks",
			},
		},
	}
	cfg.MergeVaultSecrets(map[string]string{
		"auth.oidc.client_secret": "vault-oidc-secret",
	})
	assert.Equal(t, "vault-oidc-secret", cfg.Auth.OIDC.ClientSecret)
}

func TestVault_EmptySecrets_NoChange(t *testing.T) {
	cfg := &Config{
		Database: &DatabaseConfig{Password: "original"},
		Redis:    &RedisConfig{Password: "original"},
	}
	cfg.MergeVaultSecrets(map[string]string{})
	assert.Equal(t, "original", cfg.Database.Password)
	assert.Equal(t, "original", cfg.Redis.Password)
}

func TestVault_NilSections_Safe(t *testing.T) {
	cfg := &Config{}
	// Should not panic when all optional sections are nil
	cfg.MergeVaultSecrets(map[string]string{
		"database.password":       "secret",
		"redis.password":          "secret",
		"kafka.sasl.username":     "user",
		"kafka.sasl.password":     "pass",
		"redis_session.password":  "secret",
		"auth.oidc.client_secret": "secret",
	})
	assert.Nil(t, cfg.Database)
	assert.Nil(t, cfg.Redis)
	assert.Nil(t, cfg.Kafka)
	assert.Nil(t, cfg.RedisSession)
	assert.Nil(t, cfg.Auth.OIDC)
}

func TestVault_PartialSecrets(t *testing.T) {
	cfg := &Config{
		Database: &DatabaseConfig{Password: "old-db"},
		Redis:    &RedisConfig{Password: "old-redis"},
		Auth: AuthConfig{
			OIDC: &OIDCConfig{
				DiscoveryURL: "http://localhost/.well-known",
				ClientID:     "test",
				RedirectURI:  "http://localhost/callback",
				JWKSURI:      "http://localhost/jwks",
				ClientSecret: "old-oidc",
			},
		},
	}
	// Only database.password is provided; redis and oidc should keep originals
	cfg.MergeVaultSecrets(map[string]string{
		"database.password": "new-db",
	})
	assert.Equal(t, "new-db", cfg.Database.Password)
	assert.Equal(t, "old-redis", cfg.Redis.Password)
	assert.Equal(t, "old-oidc", cfg.Auth.OIDC.ClientSecret)
}
