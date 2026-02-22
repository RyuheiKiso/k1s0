package config

import (
	"fmt"
	"os"
	"time"

	"gopkg.in/yaml.v3"
)

// BFFConfig holds BFF Proxy specific configuration on top of common fields.
type BFFConfig struct {
	App           AppConfig           `yaml:"app" validate:"required"`
	Server        ServerConfig        `yaml:"server" validate:"required"`
	Observability ObservabilityConfig `yaml:"observability" validate:"required"`
	Auth          AuthConfig          `yaml:"auth" validate:"required"`
	Session       SessionConfig       `yaml:"session" validate:"required"`
	CSRF          CSRFConfig          `yaml:"csrf"`
	Upstream      UpstreamConfig      `yaml:"upstream" validate:"required"`
}

// AppConfig identifies the service.
type AppConfig struct {
	Name        string `yaml:"name" validate:"required"`
	Version     string `yaml:"version" validate:"required"`
	Tier        string `yaml:"tier" validate:"required"`
	Environment string `yaml:"environment" validate:"required"`
}

// ServerConfig holds HTTP listener settings.
type ServerConfig struct {
	Host            string `yaml:"host" validate:"required"`
	Port            int    `yaml:"port" validate:"required,min=1,max=65535"`
	ReadTimeout     string `yaml:"read_timeout"`
	WriteTimeout    string `yaml:"write_timeout"`
	ShutdownTimeout string `yaml:"shutdown_timeout"`
}

// ObservabilityConfig holds log/trace/metrics settings.
type ObservabilityConfig struct {
	Log     LogConfig     `yaml:"log"`
	Trace   TraceConfig   `yaml:"trace"`
	Metrics MetricsConfig `yaml:"metrics"`
}

// LogConfig configures structured logging.
type LogConfig struct {
	Level  string `yaml:"level" validate:"oneof=debug info warn error"`
	Format string `yaml:"format" validate:"oneof=json text"`
}

// TraceConfig configures OpenTelemetry tracing.
type TraceConfig struct {
	Enabled    bool    `yaml:"enabled"`
	Endpoint   string  `yaml:"endpoint"`
	SampleRate float64 `yaml:"sample_rate" validate:"min=0,max=1"`
}

// MetricsConfig configures Prometheus metrics.
type MetricsConfig struct {
	Enabled bool   `yaml:"enabled"`
	Path    string `yaml:"path"`
}

// AuthConfig holds OIDC provider settings for the BFF.
type AuthConfig struct {
	DiscoveryURL string   `yaml:"discovery_url" validate:"required,url"`
	ClientID     string   `yaml:"client_id" validate:"required"`
	ClientSecret string   `yaml:"client_secret"`
	RedirectURI  string   `yaml:"redirect_uri" validate:"required,url"`
	PostLogout   string   `yaml:"post_logout_redirect_uri"`
	Scopes       []string `yaml:"scopes"`
}

// SessionConfig holds Redis session store settings.
type SessionConfig struct {
	Redis   RedisSessionConfig `yaml:"redis" validate:"required"`
	TTL     string             `yaml:"ttl"`
	Prefix  string             `yaml:"prefix"`
	Sliding bool               `yaml:"sliding"`
}

// RedisSessionConfig holds Redis connection parameters for session storage.
type RedisSessionConfig struct {
	Addr       string `yaml:"addr" validate:"required"`
	Password   string `yaml:"password"`
	DB         int    `yaml:"db"`
	MasterName string `yaml:"master_name"`
}

// CSRFConfig holds CSRF protection settings.
type CSRFConfig struct {
	Enabled    bool   `yaml:"enabled"`
	HeaderName string `yaml:"header_name"`
}

// UpstreamConfig holds the backend API base URL.
type UpstreamConfig struct {
	BaseURL string `yaml:"base_url" validate:"required,url"`
	Timeout string `yaml:"timeout"`
}

// ParseDuration parses a duration string with a fallback default.
func ParseDuration(s string, fallback time.Duration) time.Duration {
	if s == "" {
		return fallback
	}
	d, err := time.ParseDuration(s)
	if err != nil {
		return fallback
	}
	return d
}

// Load reads the base YAML configuration and optionally merges an environment overlay.
func Load(basePath string, envPath ...string) (*BFFConfig, error) {
	data, err := os.ReadFile(basePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	var cfg BFFConfig
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	if len(envPath) > 0 && envPath[0] != "" {
		envData, err := os.ReadFile(envPath[0])
		if err != nil {
			return nil, fmt.Errorf("failed to read env config: %w", err)
		}
		if err := yaml.Unmarshal(envData, &cfg); err != nil {
			return nil, fmt.Errorf("failed to merge env config: %w", err)
		}
	}

	return &cfg, nil
}
