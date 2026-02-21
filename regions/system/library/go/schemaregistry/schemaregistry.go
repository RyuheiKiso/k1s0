package schemaregistry

import (
	"context"
	"fmt"
)

// RegisteredSchema は Schema Registry に登録されたスキーマ。
type RegisteredSchema struct {
	// ID はスキーマの一意な識別子。
	ID int `json:"id"`
	// Subject はサブジェクト名（例: "user.created.v1-value"）。
	Subject string `json:"subject"`
	// Version はスキーマのバージョン番号。
	Version int `json:"version"`
	// Schema はスキーマの JSON 文字列。
	Schema string `json:"schema"`
	// SchemaType はスキーマの型（"AVRO", "JSON", "PROTOBUF"）。
	SchemaType string `json:"schemaType"`
}

// SchemaRegistryClient は Confluent Schema Registry API クライアントインターフェース。
type SchemaRegistryClient interface {
	// RegisterSchema はスキーマを登録し、スキーマ ID を返す。
	RegisterSchema(ctx context.Context, subject, schema, schemaType string) (int, error)
	// GetSchemaByID はスキーマ ID でスキーマを取得する。
	GetSchemaByID(ctx context.Context, id int) (*RegisteredSchema, error)
	// GetLatestSchema はサブジェクトの最新スキーマを取得する。
	GetLatestSchema(ctx context.Context, subject string) (*RegisteredSchema, error)
	// GetSchemaVersion はサブジェクトの特定バージョンのスキーマを取得する。
	GetSchemaVersion(ctx context.Context, subject string, version int) (*RegisteredSchema, error)
	// ListSubjects は全サブジェクトの一覧を返す。
	ListSubjects(ctx context.Context) ([]string, error)
	// CheckCompatibility はスキーマの後方互換性を検証する。
	CheckCompatibility(ctx context.Context, subject, schema string) (bool, error)
	// HealthCheck は Schema Registry の疎通確認を行う。
	HealthCheck(ctx context.Context) error
}

// NotFoundError はスキーマが見つからない場合のエラー。
type NotFoundError struct {
	Resource string
}

// Error は NotFoundError の文字列表現を返す。
func (e *NotFoundError) Error() string {
	return fmt.Sprintf("not found: %s", e.Resource)
}

// IsNotFound は err が NotFoundError かどうかを返す。
func IsNotFound(err error) bool {
	_, ok := err.(*NotFoundError)
	return ok
}

// SchemaRegistryError は Schema Registry API のエラー。
type SchemaRegistryError struct {
	StatusCode int
	Message    string
}

// Error は SchemaRegistryError の文字列表現を返す。
func (e *SchemaRegistryError) Error() string {
	return fmt.Sprintf("schema registry error (status %d): %s", e.StatusCode, e.Message)
}
