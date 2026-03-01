package kafka

import (
	"fmt"
	"strings"
)

// デフォルト値定数。
const (
	// DefaultConnectionTimeoutMs は接続タイムアウトのデフォルト値（ミリ秒）。
	DefaultConnectionTimeoutMs = 5000
	// DefaultRequestTimeoutMs はリクエストタイムアウトのデフォルト値（ミリ秒）。
	DefaultRequestTimeoutMs = 30000
	// DefaultMaxMessageBytes は最大メッセージサイズのデフォルト値（バイト）。
	DefaultMaxMessageBytes = 1000000
)

// KafkaConfig は Kafka 接続設定。
type KafkaConfig struct {
	// BootstrapServers は Kafka ブローカーアドレスのリスト。
	BootstrapServers []string
	// SecurityProtocol は PLAINTEXT, SSL, SASL_PLAINTEXT, SASL_SSL のいずれか。
	SecurityProtocol string
	// SASLMechanism は PLAIN, SCRAM-SHA-256, SCRAM-SHA-512 のいずれか。
	SASLMechanism string
	// SASLUsername は SASL 認証のユーザー名。
	SASLUsername string
	// SASLPassword は SASL 認証のパスワード。
	SASLPassword string
	// ConsumerGroup はコンシューマーグループ ID。
	ConsumerGroup string
	// ConnectionTimeoutMs は接続タイムアウト（ミリ秒）。0 の場合はデフォルト値 5000 を使用。
	ConnectionTimeoutMs int
	// RequestTimeoutMs はリクエストタイムアウト（ミリ秒）。0 の場合はデフォルト値 30000 を使用。
	RequestTimeoutMs int
	// MaxMessageBytes は最大メッセージサイズ（バイト）。0 の場合はデフォルト値 1000000 を使用。
	MaxMessageBytes int
}

// BootstrapServersString は BootstrapServers をカンマ区切りの文字列に変換する。
func (c *KafkaConfig) BootstrapServersString() string {
	return strings.Join(c.BootstrapServers, ",")
}

// UsesTLS は TLS 接続を使用するかどうかを返す。
func (c *KafkaConfig) UsesTLS() bool {
	return c.SecurityProtocol == "SSL" || c.SecurityProtocol == "SASL_SSL"
}

// EffectiveConnectionTimeoutMs は実効接続タイムアウト（ミリ秒）を返す。
// 0 の場合はデフォルト値を返す。
func (c *KafkaConfig) EffectiveConnectionTimeoutMs() int {
	if c.ConnectionTimeoutMs <= 0 {
		return DefaultConnectionTimeoutMs
	}
	return c.ConnectionTimeoutMs
}

// EffectiveRequestTimeoutMs は実効リクエストタイムアウト（ミリ秒）を返す。
// 0 の場合はデフォルト値を返す。
func (c *KafkaConfig) EffectiveRequestTimeoutMs() int {
	if c.RequestTimeoutMs <= 0 {
		return DefaultRequestTimeoutMs
	}
	return c.RequestTimeoutMs
}

// EffectiveMaxMessageBytes は実効最大メッセージサイズ（バイト）を返す。
// 0 の場合はデフォルト値を返す。
func (c *KafkaConfig) EffectiveMaxMessageBytes() int {
	if c.MaxMessageBytes <= 0 {
		return DefaultMaxMessageBytes
	}
	return c.MaxMessageBytes
}

// Validate は設定を検証する。
func (c *KafkaConfig) Validate() error {
	if len(c.BootstrapServers) == 0 {
		return fmt.Errorf("bootstrap servers must not be empty")
	}
	validProtocols := map[string]bool{
		"PLAINTEXT":      true,
		"SSL":            true,
		"SASL_PLAINTEXT": true,
		"SASL_SSL":       true,
	}
	if c.SecurityProtocol != "" && !validProtocols[c.SecurityProtocol] {
		return fmt.Errorf("invalid security protocol: %s", c.SecurityProtocol)
	}
	if c.ConnectionTimeoutMs < 0 {
		return fmt.Errorf("connection timeout must not be negative")
	}
	if c.RequestTimeoutMs < 0 {
		return fmt.Errorf("request timeout must not be negative")
	}
	if c.MaxMessageBytes < 0 {
		return fmt.Errorf("max message bytes must not be negative")
	}
	return nil
}
