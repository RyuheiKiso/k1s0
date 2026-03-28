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
	// CORS は Cross-Origin Resource Sharing の設定（H-1対応）
	CORS CORSConfig `yaml:"cors"`
	// RateLimit は IP ベースのレート制限設定（H-2対応）
	RateLimit RateLimitConfig `yaml:"rate_limit"`
	Upstream  UpstreamConfig  `yaml:"upstream" validate:"required"`
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
	// OTLPInsecure は gRPC トレースエクスポーターを insecure モード（TLS なし）で接続するかどうかを制御する（HIGH-11 対応）。
	// エンドポイントの URL スキームによる自動判定は不完全なため、明示的なフラグで制御する。
	// 本番環境では false に設定し、TLS による暗号化通信を使用すること。
	// 開発・テスト環境でローカルの OTLP コレクターに接続する場合のみ true にすること。
	OTLPInsecure bool `yaml:"otlp_insecure"`
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
	// AbsoluteMaxTTL はセッションの絶対最大有効期間（M-17 監査対応）。
	// スライディングウィンドウで TTL が延長され続けても、この期間を超えたセッションは無効化される。
	// デフォルト値: "24h"
	AbsoluteMaxTTL string `yaml:"absolute_max_ttl"`
}

// RedisSessionConfig holds Redis connection parameters for session storage.
type RedisSessionConfig struct {
	Addr       string `yaml:"addr" validate:"required"`
	Password   string `yaml:"password"`
	DB         int    `yaml:"db"`
	MasterName string `yaml:"master_name"`
	// コネクションプール設定（M-011）
	// PoolSize: 最大コネクション数（デフォルト: CPU数 * 10）
	PoolSize int `yaml:"pool_size"`
	// MinIdleConns: アイドル状態で維持する最小コネクション数
	MinIdleConns int `yaml:"min_idle_conns"`
	// MaxRetries: コマンド失敗時の最大リトライ回数（デフォルト: 3）
	MaxRetries int `yaml:"max_retries"`
	// DialTimeout: 接続タイムアウト（例: "5s"）
	DialTimeout string `yaml:"dial_timeout"`
	// ReadTimeout: 読み込みタイムアウト（例: "3s"）
	ReadTimeout string `yaml:"read_timeout"`
	// WriteTimeout: 書き込みタイムアウト（例: "3s"）
	WriteTimeout string `yaml:"write_timeout"`
}

// CSRFConfig holds CSRF protection settings.
type CSRFConfig struct {
	Enabled    bool   `yaml:"enabled"`
	HeaderName string `yaml:"header_name"`
}

// CORSConfig holds Cross-Origin Resource Sharing settings.
// H-1対応: 明示的な Origin ホワイトリストを設定し、意図せぬオリジンを拒否する。
type CORSConfig struct {
	// Enabled は CORS ミドルウェアを有効にするかどうか。
	Enabled bool `yaml:"enabled"`
	// AllowOrigins は許可するオリジンのリスト（例: ["https://app.k1s0.example.com", "http://localhost:3000"]）
	AllowOrigins []string `yaml:"allow_origins"`
	// AllowHeaders は許可するリクエストヘッダーのリスト（未設定時はデフォルト値を使用する）
	AllowHeaders []string `yaml:"allow_headers"`
	// ExposeHeaders はブラウザが参照できるレスポンスヘッダーのリスト（未設定時はデフォルト値を使用する）
	ExposeHeaders []string `yaml:"expose_headers"`
	// MaxAgeSecs はプリフライトレスポンスのキャッシュ時間（秒）。0 の場合は 600 秒を使用する。
	MaxAgeSecs int `yaml:"max_age_secs"`
	// CredentialsPaths は Access-Control-Allow-Credentials: true を返すパスプレフィックスのリスト。
	// H-13 監査対応: 認証が不要な公開エンドポイント（/healthz, /metrics 等）には credentials を付与しない。
	// 未設定時はすべてのホワイトリストオリジンに対して credentials を付与する（後方互換性）。
	// 例: ["/auth/", "/api/"] のように末尾スラッシュ付きで指定する。
	CredentialsPaths []string `yaml:"credentials_paths"`
}

// RateLimitConfig holds IP-based rate limiting settings.
// H-2対応: DDoS攻撃や大量リクエストから BFF Proxy を保護するためのレート制限。
type RateLimitConfig struct {
	// Enabled は IP ベースのレート制限を有効にするかどうか。
	Enabled bool `yaml:"enabled"`
	// RPS は 1 IP あたりの秒間リクエスト上限（requests per second）。0 の場合は 100 を使用する。
	RPS float64 `yaml:"rps"`
	// Burst は瞬間的に許可するリクエスト数の上限（バースト許容量）。0 の場合は RPS * 2 を使用する。
	Burst int `yaml:"burst"`
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
