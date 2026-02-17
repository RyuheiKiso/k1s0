package config

import (
	"fmt"
	"os"

	"github.com/go-playground/validator/v10"
	"gopkg.in/yaml.v3"
)

// Config は config設計.md のスキーマに準拠する。
type Config struct {
	App           AppConfig           `yaml:"app" validate:"required"`
	Server        ServerConfig        `yaml:"server" validate:"required"`
	GRPC          *GRPCConfig         `yaml:"grpc,omitempty"`
	Database      *DatabaseConfig     `yaml:"database,omitempty"`
	Kafka         *KafkaConfig        `yaml:"kafka,omitempty"`
	Redis         *RedisConfig        `yaml:"redis,omitempty"`
	RedisSession  *RedisConfig        `yaml:"redis_session,omitempty"`
	Observability ObservabilityConfig `yaml:"observability" validate:"required"`
	Auth          AuthConfig          `yaml:"auth" validate:"required"`
}

type AppConfig struct {
	Name        string `yaml:"name" validate:"required"`
	Version     string `yaml:"version" validate:"required"`
	Tier        string `yaml:"tier" validate:"required,oneof=system business service"`
	Environment string `yaml:"environment" validate:"required,oneof=dev staging prod"`
}

type ServerConfig struct {
	Host            string `yaml:"host" validate:"required"`
	Port            int    `yaml:"port" validate:"required,min=1,max=65535"`
	ReadTimeout     string `yaml:"read_timeout"`
	WriteTimeout    string `yaml:"write_timeout"`
	ShutdownTimeout string `yaml:"shutdown_timeout"`
}

type GRPCConfig struct {
	Port           int `yaml:"port" validate:"required,min=1,max=65535"`
	MaxRecvMsgSize int `yaml:"max_recv_msg_size"`
}

type DatabaseConfig struct {
	Host            string `yaml:"host" validate:"required"`
	Port            int    `yaml:"port" validate:"required,min=1,max=65535"`
	Name            string `yaml:"name" validate:"required"`
	User            string `yaml:"user" validate:"required"`
	Password        string `yaml:"password"`
	SSLMode         string `yaml:"ssl_mode" validate:"oneof=disable require verify-full"`
	MaxOpenConns    int    `yaml:"max_open_conns"`
	MaxIdleConns    int    `yaml:"max_idle_conns"`
	ConnMaxLifetime string `yaml:"conn_max_lifetime"`
}

type KafkaConfig struct {
	Brokers          []string         `yaml:"brokers" validate:"required,min=1"`
	ConsumerGroup    string           `yaml:"consumer_group" validate:"required"`
	SecurityProtocol string           `yaml:"security_protocol" validate:"required,oneof=PLAINTEXT SASL_SSL"`
	SASL             *KafkaSASLConfig `yaml:"sasl,omitempty"`
	TLS              *KafkaTLSConfig  `yaml:"tls,omitempty"`
	Topics           KafkaTopics      `yaml:"topics"`
}

type KafkaSASLConfig struct {
	Mechanism string `yaml:"mechanism" validate:"required,oneof=SCRAM-SHA-512 PLAIN"`
	Username  string `yaml:"username"`
	Password  string `yaml:"password"`
}

type KafkaTLSConfig struct {
	CACertPath string `yaml:"ca_cert_path"`
}

type KafkaTopics struct {
	Publish   []string `yaml:"publish"`
	Subscribe []string `yaml:"subscribe"`
}

type RedisConfig struct {
	Host     string `yaml:"host" validate:"required"`
	Port     int    `yaml:"port" validate:"required,min=1,max=65535"`
	Password string `yaml:"password"`
	DB       int    `yaml:"db"`
	PoolSize int    `yaml:"pool_size"`
}

type ObservabilityConfig struct {
	Log     LogConfig     `yaml:"log"`
	Trace   TraceConfig   `yaml:"trace"`
	Metrics MetricsConfig `yaml:"metrics"`
}

type LogConfig struct {
	Level  string `yaml:"level" validate:"oneof=debug info warn error"`
	Format string `yaml:"format" validate:"oneof=json text"`
}

type TraceConfig struct {
	Enabled    bool    `yaml:"enabled"`
	Endpoint   string  `yaml:"endpoint"`
	SampleRate float64 `yaml:"sample_rate" validate:"min=0,max=1"`
}

type MetricsConfig struct {
	Enabled bool   `yaml:"enabled"`
	Path    string `yaml:"path"`
}

type AuthConfig struct {
	JWT  JWTConfig   `yaml:"jwt" validate:"required"`
	OIDC *OIDCConfig `yaml:"oidc,omitempty"`
}

type JWTConfig struct {
	Issuer        string `yaml:"issuer" validate:"required"`
	Audience      string `yaml:"audience" validate:"required"`
	PublicKeyPath string `yaml:"public_key_path"`
}

type OIDCConfig struct {
	DiscoveryURL string   `yaml:"discovery_url" validate:"required,url"`
	ClientID     string   `yaml:"client_id" validate:"required"`
	ClientSecret string   `yaml:"client_secret"`
	RedirectURI  string   `yaml:"redirect_uri" validate:"required,url"`
	Scopes       []string `yaml:"scopes"`
	JWKSURI      string   `yaml:"jwks_uri" validate:"required,url"`
	JWKSCacheTTL string   `yaml:"jwks_cache_ttl"`
}

// Load は basePath の YAML を読み込み、envPath があればマージする。
func Load(basePath string, envPath ...string) (*Config, error) {
	data, err := os.ReadFile(basePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	var cfg Config
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	if len(envPath) > 0 && envPath[0] != "" {
		if err := mergeFromFile(&cfg, envPath[0]); err != nil {
			return nil, fmt.Errorf("failed to merge env config: %w", err)
		}
	}

	return &cfg, nil
}

// Validate は設定値のバリデーションを実行する。
func (c *Config) Validate() error {
	v := validator.New()
	if err := v.Struct(c); err != nil {
		return fmt.Errorf("config validation failed: %w", err)
	}
	return nil
}
