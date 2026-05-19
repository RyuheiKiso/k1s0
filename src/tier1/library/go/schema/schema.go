// schema.go — k1s0 tier1 Library Go 実装: Schema Registry の L2* interface
// 12_スキーマレジストリ適合仕様.md §SchemaRegistryClient（族内共通 L2*）に準拠する。
// Confluent Schema Registry / Apicurio 等の OSS API を Library 独自語彙に翻訳する L2* facade を宣言する。
// 公開シグネチャに OSS 型（schemaregistry.Client 等）を一切含まない。

// パッケージ名: schema（tier1 Library の Schema Registry API を提供する）
package schema

import (
	// context: context.Context（非同期操作に使用する）
	"context"
)

// SchemaFormat はスキーマフォーマットを宣言する型。
// Confluent Schema Registry の schema type に準拠した Library 独自語彙とする。
type SchemaFormat string

const (
	// SchemaFormatAvro: Apache Avro フォーマット
	SchemaFormatAvro SchemaFormat = "avro"
	// SchemaFormatJSON: JSON Schema フォーマット
	SchemaFormatJSON SchemaFormat = "json"
	// SchemaFormatProtobuf: Protocol Buffers フォーマット
	SchemaFormatProtobuf SchemaFormat = "protobuf"
)

// CompatibilityMode はスキーマ互換性モードを宣言する型。
// Confluent Schema Registry の compatibility 設定に準拠した Library 独自語彙とする。
type CompatibilityMode string

const (
	// CompatibilityBackward: 後方互換（新スキーマで旧データが読める）
	CompatibilityBackward CompatibilityMode = "backward"
	// CompatibilityForward: 前方互換（旧スキーマで新データが読める）
	CompatibilityForward CompatibilityMode = "forward"
	// CompatibilityFull: 双方向互換（backward + forward）
	CompatibilityFull CompatibilityMode = "full"
	// CompatibilityNone: 互換性チェックなし（開発環境のみ使用可）
	CompatibilityNone CompatibilityMode = "none"
)

// SchemaInfo は Schema Registry に登録されたスキーマ情報を宣言する型。
// OSS の SchemaInfo レスポンス型を露出せず Library 独自語彙で表現する。
type SchemaInfo struct {
	// ID: Schema Registry が発行したスキーマ ID（グローバルユニーク）
	ID int64
	// Subject: スキーマが属する Subject 名（"{topic}-value" / "{topic}-key" 形式）
	Subject string
	// Version: スキーマバージョン（1 始まりの単調増加整数）
	Version int
	// Format: スキーマフォーマット（Avro / JSON / Protobuf）
	Format SchemaFormat
	// Schema: スキーマ定義文字列（Avro JSON / JSON Schema / .proto 等）
	Schema string
	// References: 参照するスキーマの一覧（Protobuf import 等）
	References []SchemaReference
}

// SchemaReference は別のスキーマを参照する定義を宣言する型。
// Confluent Schema Registry の SchemaReference に準拠した Library 独自語彙とする。
type SchemaReference struct {
	// Name: 参照名（Protobuf の import パス等）
	Name string
	// Subject: 参照先 Subject 名
	Subject string
	// Version: 参照するバージョン
	Version int
}

// SchemaRegistryClient は Schema Registry の L2* 抽象 interface を宣言する。
// Confluent Schema Registry / Apicurio / AWS Glue Schema Registry 等を抽象化する。
// OSS 型（schemaregistry.Client 等）を引数・戻り値に一切含まない。
type SchemaRegistryClient interface {
	// Register は Subject にスキーマを登録して ID を返す。
	// subject はスキーマ Subject 名（"{topic}-value" 形式を推奨する）。
	// schema はスキーマ定義文字列、format はスキーマフォーマット。
	// 同一スキーマが既に登録されている場合は既存 ID を返す（idempotent 操作）。
	Register(ctx context.Context, subject string, schema string, format SchemaFormat) (int64, error)

	// GetByID は ID に対応するスキーマ情報を取得する。
	// スキーマが存在しない場合は nil, nil を返す（エラーと区別する）。
	GetByID(ctx context.Context, id int64) (*SchemaInfo, error)

	// GetLatest は Subject の最新バージョンのスキーマ情報を取得する。
	// Subject が存在しない場合は nil, nil を返す（エラーと区別する）。
	GetLatest(ctx context.Context, subject string) (*SchemaInfo, error)

	// GetByVersion は Subject の指定バージョンのスキーマ情報を取得する。
	GetByVersion(ctx context.Context, subject string, version int) (*SchemaInfo, error)

	// ListSubjects は登録されている Subject 名の一覧を返す。
	ListSubjects(ctx context.Context) ([]string, error)

	// ListVersions は Subject のバージョン番号一覧を返す。
	ListVersions(ctx context.Context, subject string) ([]int, error)

	// CheckCompatibility は新しいスキーマが Subject の既存スキーマと互換性があるかチェックする。
	// mode は互換性チェックのモード（nil = Subject の設定を使用する）。
	CheckCompatibility(ctx context.Context, subject string, schema string, format SchemaFormat) (bool, error)

	// SetCompatibilityMode は Subject のスキーマ互換性モードを設定する。
	SetCompatibilityMode(ctx context.Context, subject string, mode CompatibilityMode) error

	// DeleteSubject は Subject を全バージョン削除する（hard delete または soft delete）。
	// permanent=true の場合は hard delete（復元不可）、false は soft delete。
	DeleteSubject(ctx context.Context, subject string, permanent bool) error
}

// SchemaCodec はスキーマを使ってメッセージをエンコード / デコードする L2* interface を宣言する。
// Confluent Wire Format（magic byte + schema ID）を Library 独自語彙で抽象化する。
type SchemaCodec interface {
	// Encode はスキーマ ID とデータバイト列から Confluent Wire Format のバイト列を生成する。
	// schemaID は Schema Registry で登録されたスキーマ ID（magic byte 付きで先頭に埋め込む）。
	// data はスキーマに従ってシリアライズ済みのバイト列（Avro binary / Protobuf wire 等）。
	Encode(ctx context.Context, schemaID int64, data []byte) ([]byte, error)

	// Decode は Confluent Wire Format のバイト列からスキーマ ID とデータバイト列を取り出す。
	// 戻り値の int64 はスキーマ ID、[]byte はスキーマ固有の wire format バイト列。
	Decode(ctx context.Context, wireFormatBytes []byte) (int64, []byte, error)
}
