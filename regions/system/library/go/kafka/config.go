package kafka

import (
	"fmt"
	"strings"
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
}

// BootstrapServersString は BootstrapServers をカンマ区切りの文字列に変換する。
func (c *KafkaConfig) BootstrapServersString() string {
	return strings.Join(c.BootstrapServers, ",")
}

// UsesTLS は TLS 接続を使用するかどうかを返す。
func (c *KafkaConfig) UsesTLS() bool {
	return c.SecurityProtocol == "SSL" || c.SecurityProtocol == "SASL_SSL"
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
	return nil
}
