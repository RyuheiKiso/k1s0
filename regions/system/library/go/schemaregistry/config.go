package schemaregistry

import "fmt"

// SchemaRegistryConfig は Schema Registry 接続設定。
type SchemaRegistryConfig struct {
	// URL は Schema Registry のベース URL。
	URL string
	// Username は基本認証のユーザー名（省略可能）。
	Username string
	// Password は基本認証のパスワード（省略可能）。
	Password string
}

// SubjectName はトピック名からサブジェクト名を生成する。
// Confluent の命名規則: <topic>-value または <topic>-key
func (c *SchemaRegistryConfig) SubjectName(topic, keyOrValue string) string {
	return fmt.Sprintf("%s-%s", topic, keyOrValue)
}

// Validate は設定を検証する。
func (c *SchemaRegistryConfig) Validate() error {
	if c.URL == "" {
		return fmt.Errorf("schema registry URL must not be empty")
	}
	return nil
}
