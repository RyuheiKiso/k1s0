package config

import (
	"bytes"
	"fmt"
	"os"
	"strings"
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
	// Cookie は Cookie のデフォルト設定。
	Cookie CookieConfig `yaml:"cookie"`
}

// CookieConfig は Cookie のデフォルト設定。
type CookieConfig struct {
	// Domain は Cookie を発行するドメイン。空文字の場合はブラウザがオリジンから自動設定する。
	Domain string `yaml:"domain"`
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

// IsDevEnvironment は開発環境かどうかを判定する。
// "dev", "development", "local" のいずれかを開発環境として扱う。
func IsDevEnvironment(env string) bool {
	switch strings.ToLower(strings.TrimSpace(env)) {
	case "dev", "development", "local":
		return true
	}
	return false
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
// 未知フィールドが含まれる場合はエラーを返し、設定ミスを早期に検出する。
func Load(basePath string, envPath ...string) (*BFFConfig, error) {
	data, err := os.ReadFile(basePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	var cfg BFFConfig
	// 未知フィールドを拒否する: タイポや廃止フィールドの混入を防ぐ
	baseDecoder := yaml.NewDecoder(bytes.NewReader(data))
	baseDecoder.KnownFields(true)
	if err := baseDecoder.Decode(&cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	if len(envPath) > 0 && envPath[0] != "" {
		envData, err := os.ReadFile(envPath[0])
		if err != nil {
			return nil, fmt.Errorf("failed to read env config: %w", err)
		}
		// 環境オーバーレイでも未知フィールドを拒否する
		envDecoder := yaml.NewDecoder(bytes.NewReader(envData))
		envDecoder.KnownFields(true)
		if err := envDecoder.Decode(&cfg); err != nil {
			return nil, fmt.Errorf("failed to merge env config: %w", err)
		}
	}

	return &cfg, nil
}
