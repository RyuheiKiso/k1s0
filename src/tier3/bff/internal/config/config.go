// BFF サービスの設定ロード。

// Package config は portal-bff / admin-bff の起動時設定を集約する。
package config

// 標準 import。
import (
	// 文字列整形。
	"fmt"
	// 環境変数。
	"os"
	// strconv で数値変換。
	"strconv"
	// 文字列処理。
	"strings"
)

// Config は BFF のトップレベル設定。
type Config struct {
	// アプリ名（"portal-bff" / "admin-bff"）。OTel resource attribute service.name に投入。
	AppName string
	// サービスバージョン。
	ServiceVersion string
	// 環境（dev / staging / prod）。
	Environment string
	// OTLP collector endpoint。
	OTLPEndpoint string
	// HTTP server 設定。
	HTTP HTTPConfig
	// k1s0 facade 接続設定。
	K1s0 K1s0Config
}

// HTTPConfig は HTTP server の設定。
type HTTPConfig struct {
	Addr            string
	ReadTimeoutSec  int
	WriteTimeoutSec int
}

// K1s0Config は k1s0 SDK Client 構築時の設定。
type K1s0Config struct {
	Target   string
	TenantID string
	Subject  string
	UseTLS   bool
}

// Load は appName を引数に受け、環境変数から Config を組み立てる。
func Load(appName string) (*Config, error) {
	// 構造体を組み立てる。
	cfg := &Config{
		// アプリ名。
		AppName: appName,
		// サービスバージョン。
		ServiceVersion: getenvDefault("SERVICE_VERSION", "0.0.0-dev"),
		// 環境。
		Environment: getenvDefault("ENVIRONMENT", "dev"),
		// OTel exporter endpoint。
		OTLPEndpoint: os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT"),
		// HTTP server。
		HTTP: HTTPConfig{
			Addr:            getenvDefault("HTTP_ADDR", ":8080"),
			ReadTimeoutSec:  getenvIntDefault("HTTP_READ_TIMEOUT_SEC", 15),
			WriteTimeoutSec: getenvIntDefault("HTTP_WRITE_TIMEOUT_SEC", 15),
		},
		// k1s0 SDK 設定（subject はアプリ名で正規化）。
		K1s0: K1s0Config{
			Target:   getenvDefault("K1S0_TARGET", "tier1-state.k1s0-system.svc.cluster.local:50001"),
			TenantID: os.Getenv("K1S0_TENANT_ID"),
			Subject:  getenvDefault("K1S0_SUBJECT", "tier3/"+appName),
			UseTLS:   getenvBoolDefault("K1S0_USE_TLS", false),
		},
	}
	// 必須項目の検証。
	if err := cfg.validate(); err != nil {
		return nil, err
	}
	return cfg, nil
}

// validate は最低限の必須項目を検査する。
func (c *Config) validate() error {
	// テナント ID は必須。
	if strings.TrimSpace(c.K1s0.TenantID) == "" {
		return fmt.Errorf("config: K1S0_TENANT_ID is required")
	}
	if c.K1s0.Target == "" {
		return fmt.Errorf("config: K1S0_TARGET is required")
	}
	if c.AppName == "" {
		return fmt.Errorf("config: appName is required")
	}
	return nil
}

func getenvDefault(key, def string) string {
	v := os.Getenv(key)
	if v == "" {
		return def
	}
	return v
}

func getenvIntDefault(key string, def int) int {
	v := os.Getenv(key)
	if v == "" {
		return def
	}
	parsed, err := strconv.Atoi(v)
	if err != nil {
		return def
	}
	return parsed
}

func getenvBoolDefault(key string, def bool) bool {
	v := strings.ToLower(strings.TrimSpace(os.Getenv(key)))
	if v == "" {
		return def
	}
	switch v {
	case "1", "true", "t", "yes", "y", "on":
		return true
	case "0", "false", "f", "no", "n", "off":
		return false
	default:
		return def
	}
}
