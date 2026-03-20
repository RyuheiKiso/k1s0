package config

import (
	"errors"
	"fmt"
	"log/slog"
	"os"
	"strings"

	"github.com/go-playground/validator/v10"
	"gopkg.in/yaml.v3"
)

// センチネルエラー: errors.Is() によるエラー判定を可能にする。
var (
	// ErrConfigNotFound は設定ファイルが見つからない場合のエラー。
	ErrConfigNotFound = errors.New("設定ファイルが見つからない")
	// ErrConfigInvalid は設定ファイルの内容が無効（パース不可）な場合のエラー。
	ErrConfigInvalid = errors.New("設定が無効")
	// ErrConfigLoadFailed は設定の読み込みに失敗した場合の汎用エラー。
	ErrConfigLoadFailed = errors.New("設定の読み込みに失敗")
	// ErrConfigValidation は設定値のバリデーションに失敗した場合のエラー。
	ErrConfigValidation = errors.New("設定のバリデーションに失敗")
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
	SecurityProtocol string           `yaml:"security_protocol" validate:"required,oneof=PLAINTEXT SSL SASL_PLAINTEXT SASL_SSL"`
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
// ファイルが見つからない場合は ErrConfigNotFound、パース失敗時は ErrConfigInvalid、
// その他の読み込み失敗時は ErrConfigLoadFailed をラップして返す。
func Load(basePath string, envPath ...string) (*Config, error) {
	data, err := os.ReadFile(basePath)
	if err != nil {
		// ファイルが存在しない場合は ErrConfigNotFound、それ以外は ErrConfigLoadFailed
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("%w: %s: %w", ErrConfigNotFound, basePath, err)
		}
		return nil, fmt.Errorf("%w: %s: %w", ErrConfigLoadFailed, basePath, err)
	}

	var cfg Config
	// YAML パースに失敗した場合は ErrConfigInvalid
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("%w: %w", ErrConfigInvalid, err)
	}

	if len(envPath) > 0 && envPath[0] != "" {
		// 環境設定ファイルのマージに失敗した場合は ErrConfigLoadFailed
		if err := mergeFromFile(&cfg, envPath[0]); err != nil {
			return nil, fmt.Errorf("%w: %w", ErrConfigLoadFailed, err)
		}
	}

	return &cfg, nil
}

// Validate は設定値のバリデーションを実行する。
// バリデーション失敗時は ErrConfigValidation をラップして返す。
// OIDC の DiscoveryURL/JWKSURI が TLS (https) を使用していない場合は警告ログを出力する。
func (c *Config) Validate() error {
	v := validator.New()
	if err := v.Struct(c); err != nil {
		return fmt.Errorf("%w: %w", ErrConfigValidation, err)
	}

	// OIDC エンドポイントが TLS を使用していない場合に警告する
	// 本番環境では IdP との通信に https を使用すべき
	if c.Auth.OIDC != nil {
		if c.Auth.OIDC.DiscoveryURL != "" && !strings.HasPrefix(c.Auth.OIDC.DiscoveryURL, "https://") {
			slog.Warn("OIDC discovery_url が TLS (https) を使用していません。本番環境では https を使用してください",
				slog.String("discovery_url", c.Auth.OIDC.DiscoveryURL),
			)
		}
		if c.Auth.OIDC.JWKSURI != "" && !strings.HasPrefix(c.Auth.OIDC.JWKSURI, "https://") {
			slog.Warn("OIDC jwks_uri が TLS (https) を使用していません。本番環境では https を使用してください",
				slog.String("jwks_uri", c.Auth.OIDC.JWKSURI),
			)
		}
	}

	return nil
}

// String は DatabaseConfig の文字列表現を返す。パスワードはマスクされる。
func (c DatabaseConfig) String() string {
	// type alias で Stringer インターフェースを回避し無限再帰を防ぐ
	type plain DatabaseConfig
	p := plain(c)
	p.Password = "***"
	return fmt.Sprintf("%+v", p)
}

// String は KafkaSASLConfig の文字列表現を返す。パスワードはマスクされる。
func (c KafkaSASLConfig) String() string {
	// type alias で Stringer インターフェースを回避し無限再帰を防ぐ
	type plain KafkaSASLConfig
	p := plain(c)
	p.Password = "***"
	return fmt.Sprintf("%+v", p)
}

// String は RedisConfig の文字列表現を返す。パスワードはマスクされる。
func (c RedisConfig) String() string {
	// type alias で Stringer インターフェースを回避し無限再帰を防ぐ
	type plain RedisConfig
	p := plain(c)
	p.Password = "***"
	return fmt.Sprintf("%+v", p)
}

// String は OIDCConfig の文字列表現を返す。クライアントシークレットはマスクされる。
func (c OIDCConfig) String() string {
	// type alias で Stringer インターフェースを回避し無限再帰を防ぐ
	type plain OIDCConfig
	p := plain(c)
	p.ClientSecret = "***"
	return fmt.Sprintf("%+v", p)
}
