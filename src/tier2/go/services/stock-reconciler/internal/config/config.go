// stock-reconciler サービスの設定ロード。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   環境変数から起動時設定を組み立てる。.env / config file は本サービスの責務外
//   （K8s ConfigMap / Secret + envFrom が標準配備）。

// Package config はサービス起動時の設定を集約する。
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

// Config は stock-reconciler のトップレベル設定。
type Config struct {
	// サービスバージョン（OTel resource attribute service.version、image tag を環境変数で投入）。
	ServiceVersion string
	// 環境（dev / staging / prod）。
	Environment string
	// OTLP collector endpoint（空なら OTel no-op）。
	OTLPEndpoint string
	// HTTP server 設定。
	HTTP HTTPConfig
	// k1s0 facade 接続設定。
	K1s0 K1s0Config
	// PubSub 設定（reconcile 完了イベント発火）。
	PubSub PubSubConfig
}

// HTTPConfig は API 層 HTTP server の設定。
type HTTPConfig struct {
	// listen address（例: ":8080"）。
	Addr string
	// read timeout（秒）。
	ReadTimeoutSec int
	// write timeout（秒）。
	WriteTimeoutSec int
}

// K1s0Config は k1s0 SDK Client 構築時の設定。
type K1s0Config struct {
	// gRPC 接続先。
	Target string
	// テナント ID。
	TenantID string
	// 主体識別子（subject、tier2 サービス自身）。
	Subject string
	// TLS を使うか（本番 true）。
	UseTLS bool
	// k1s0 State の store 名（Dapr Component 名、デフォルト "postgres"）。
	StoreName string
}

// PubSubConfig は reconcile 完了イベントを発火する PubSub の設定。
type PubSubConfig struct {
	// PubSub Component 名（Dapr の pubsub.kafka など、デフォルト "kafka"）。
	ComponentName string
	// Topic 名（デフォルト "stock.reconciled"）。
	Topic string
}

// Load は環境変数から Config を組み立てる。
func Load() (*Config, error) {
	// 構造体を組み立てる。
	cfg := &Config{
		// バージョンは IMAGE_TAG 等の env で投入する。
		ServiceVersion: getenvDefault("SERVICE_VERSION", "0.0.0-dev"),
		// 環境名。
		Environment: getenvDefault("ENVIRONMENT", "dev"),
		// OTel exporter endpoint（未設定なら no-op）。
		OTLPEndpoint: os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT"),
		// HTTP server 設定。
		HTTP: HTTPConfig{
			// listen address。
			Addr: getenvDefault("HTTP_ADDR", ":8080"),
			// read timeout。
			ReadTimeoutSec: getenvIntDefault("HTTP_READ_TIMEOUT_SEC", 15),
			// write timeout。
			WriteTimeoutSec: getenvIntDefault("HTTP_WRITE_TIMEOUT_SEC", 15),
		},
		// k1s0 SDK 設定。
		K1s0: K1s0Config{
			// gRPC target。
			Target: getenvDefault("K1S0_TARGET", "tier1-state.k1s0-system.svc.cluster.local:50001"),
			// テナント ID（K8s Pod 起動時に投入）。
			TenantID: os.Getenv("K1S0_TENANT_ID"),
			// 主体識別子（SPIRE SVID から取れる場合は spec.workloadID と統一）。
			Subject: getenvDefault("K1S0_SUBJECT", "tier2/stock-reconciler"),
			// 本番は必ず TLS、dev は false。
			UseTLS: getenvBoolDefault("K1S0_USE_TLS", false),
			// k1s0 State の Dapr Component 名（デフォルト "postgres"）。
			StoreName: getenvDefault("K1S0_STATE_STORE", "postgres"),
		},
		// PubSub 設定。
		PubSub: PubSubConfig{
			// Component 名（Dapr Component CRD の metadata.name）。
			ComponentName: getenvDefault("PUBSUB_COMPONENT", "kafka"),
			// Topic 名。
			Topic: getenvDefault("PUBSUB_TOPIC", "stock.reconciled"),
		},
	}
	// 必須項目を検証する。
	if err := cfg.validate(); err != nil {
		// 不備があれば呼出元に返す。
		return nil, err
	}
	// 構築済 Config を返す。
	return cfg, nil
}

// validate は最低限の必須項目を検査する。
func (c *Config) validate() error {
	// テナント ID は tier1 facade 呼出に必須。
	if strings.TrimSpace(c.K1s0.TenantID) == "" {
		// 起動時に明示的にエラー。
		return fmt.Errorf("config: K1S0_TENANT_ID is required")
	}
	// Target が空なら接続不可。
	if c.K1s0.Target == "" {
		// 早期 fail。
		return fmt.Errorf("config: K1S0_TARGET is required")
	}
	// Topic が空なら PubSub 発火が成立しない。
	if c.PubSub.Topic == "" {
		// 早期 fail。
		return fmt.Errorf("config: PUBSUB_TOPIC is required")
	}
	// チェック通過。
	return nil
}

// getenvDefault は環境変数を取り、未設定 / 空ならデフォルト値を返す。
func getenvDefault(key, def string) string {
	// 環境変数を取得する。
	v := os.Getenv(key)
	// 空ならデフォルトを返す。
	if v == "" {
		// デフォルト値。
		return def
	}
	// そのまま返す。
	return v
}

// getenvIntDefault は環境変数を int に変換する。
func getenvIntDefault(key string, def int) int {
	// 環境変数を取得する。
	v := os.Getenv(key)
	// 空ならデフォルト。
	if v == "" {
		// デフォルト値を返す。
		return def
	}
	// int 変換を試みる。
	parsed, err := strconv.Atoi(v)
	// 失敗時はデフォルト（fail-fast せず実装の運用容易性を優先）。
	if err != nil {
		// 不正入力でもデフォルトで運用継続。
		return def
	}
	// 変換成功時は parsed 値を返す。
	return parsed
}

// getenvBoolDefault は環境変数を bool に変換する。
func getenvBoolDefault(key string, def bool) bool {
	// 環境変数を取得する。
	v := strings.ToLower(strings.TrimSpace(os.Getenv(key)))
	// 空ならデフォルト。
	if v == "" {
		// デフォルト値を返す。
		return def
	}
	// 真値の集合を判定する。
	switch v {
	// true として扱う表現。
	case "1", "true", "t", "yes", "y", "on":
		// true。
		return true
	// false として扱う表現。
	case "0", "false", "f", "no", "n", "off":
		// false。
		return false
	// 不明な値はデフォルト。
	default:
		// 不正入力でもデフォルトで運用継続。
		return def
	}
}
