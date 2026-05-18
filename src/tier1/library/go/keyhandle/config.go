// config.go — k1s0 tier1 Library Go 実装: Configuration / Feature Flag の L2* interface
// 07_設定適合仕様.md §ConfigClient / §FeatureFlagClient（族内共通 API）に準拠する。
// OpenFeature / LaunchDarkly 等の OSS API を Library 独自語彙に翻訳する L2* facade を宣言する。
// 公開シグネチャに OSS 型を露出しない（ldclient.LDClient 等は一切含まない）。

// パッケージ名: keyhandle（tier1 Library の設定・Feature Flag API を提供する）
package keyhandle

import (
	// context: context.Context（設定取得 / フラグ評価の非同期操作に使用する）
	"context"
)

// EvalContext は L2* Feature Flag の評価コンテキストを宣言する型。
// OpenFeature EvaluationContext に準拠した Library 独自語彙とする。
// tenant_id / user_id を必須フィールドとして持ち、追加属性は Attrs マップに格納する。
type EvalContext struct {
	// TenantID: フラグ評価の対象テナント識別子（必須: tenant 分離フラグ評価に使用する）
	TenantID string
	// UserID: フラグ評価の対象ユーザー識別子（省略可能: A/B test 等で使用する）
	UserID string
	// Attrs: 追加評価属性（region / plan / custom 等）
	Attrs map[string]string
}

// FlagVariant はフラグ評価の値の種別を宣言する型。
// L2* として OpenFeature の Variant 型を Library 独自語彙に翻訳する。
type FlagVariant string

const (
	// FlagVariantBool: bool 型フラグ値（true / false）
	FlagVariantBool FlagVariant = "bool"
	// FlagVariantString: string 型フラグ値（文字列バリアント）
	FlagVariantString FlagVariant = "string"
	// FlagVariantInt: int64 型フラグ値（数値バリアント）
	FlagVariantInt FlagVariant = "int"
	// FlagVariantFloat: float64 型フラグ値（浮動小数点バリアント）
	FlagVariantFloat FlagVariant = "float"
)

// FlagEvalReason はフラグ評価理由を宣言する型。
// OpenFeature EvaluationReason に準拠した Library 独自語彙とする。
type FlagEvalReason string

const (
	// FlagEvalReasonStatic: 静的（ルールマッチなし / デフォルト値）
	FlagEvalReasonStatic FlagEvalReason = "static"
	// FlagEvalReasonTargeting: ターゲティングルールに一致して評価した
	FlagEvalReasonTargeting FlagEvalReason = "targeting"
	// FlagEvalReasonSplit: A/B split test により評価した
	FlagEvalReasonSplit FlagEvalReason = "split"
	// FlagEvalReasonDefault: デフォルト値にフォールバックした（エラー発生時等）
	FlagEvalReasonDefault FlagEvalReason = "default"
)

// BoolEvalResult は bool 型フラグの評価結果を宣言する型。
// 値だけでなく評価理由も含めて返す（監査ログ・デバッグに使用する）。
type BoolEvalResult struct {
	// Value: 評価されたフラグ値
	Value bool
	// Reason: フラグ評価理由
	Reason FlagEvalReason
	// Variant: 評価されたバリアント名（規約的な名前）
	Variant string
}

// StringEvalResult は string 型フラグの評価結果を宣言する型。
type StringEvalResult struct {
	// Value: 評価されたフラグ値
	Value string
	// Reason: フラグ評価理由
	Reason FlagEvalReason
	// Variant: 評価されたバリアント名
	Variant string
}

// IntEvalResult は int64 型フラグの評価結果を宣言する型。
type IntEvalResult struct {
	// Value: 評価されたフラグ値
	Value int64
	// Reason: フラグ評価理由
	Reason FlagEvalReason
	// Variant: 評価されたバリアント名
	Variant string
}

// FloatEvalResult は float64 型フラグの評価結果を宣言する型。
type FloatEvalResult struct {
	// Value: 評価されたフラグ値
	Value float64
	// Reason: フラグ評価理由
	Reason FlagEvalReason
	// Variant: 評価されたバリアント名
	Variant string
}

// FeatureFlagClient は L2* Feature Flag 評価 interface を宣言する。
// OpenFeature Provider を Library 独自語彙で抽象化する。
// OSS 型（ldclient.LDClient / unleash.Client 等）を引数・戻り値に一切含まない。
type FeatureFlagClient interface {
	// BoolValue は bool 型フラグを評価して値を返す。
	// key はフラグキー（"feature.new-ui" 等のドット記法を推奨する）。
	// ec は評価コンテキスト（TenantID は必須: tenant 分離評価を保証する）。
	// defaultVal はフォールバック値（エラー発生時に使用する）。
	BoolValue(ctx context.Context, key string, ec EvalContext, defaultVal bool) bool

	// BoolDetail は bool 型フラグを評価して詳細結果を返す（評価理由を含む）。
	// 監査ログ・デバッグ用途で BoolValue より詳細な情報が必要な場合に使用する。
	BoolDetail(ctx context.Context, key string, ec EvalContext, defaultVal bool) (BoolEvalResult, error)

	// StringValue は string 型フラグを評価して値を返す。
	// key はフラグキー、ec は評価コンテキスト、defaultVal はフォールバック値。
	StringValue(ctx context.Context, key string, ec EvalContext, defaultVal string) string

	// StringDetail は string 型フラグを評価して詳細結果を返す。
	StringDetail(ctx context.Context, key string, ec EvalContext, defaultVal string) (StringEvalResult, error)

	// IntValue は int64 型フラグを評価して値を返す。
	IntValue(ctx context.Context, key string, ec EvalContext, defaultVal int64) int64

	// FloatValue は float64 型フラグを評価して値を返す。
	FloatValue(ctx context.Context, key string, ec EvalContext, defaultVal float64) float64
}

// ConfigValue は設定値と付随するメタデータを宣言する型。
// OSS の viper.Value 等を露出せず Library 独自語彙で表現する。
type ConfigValue struct {
	// Raw: 設定の生値（文字列表現）
	Raw string
	// Source: 設定の取得元（"env" / "file" / "remote" 等）
	Source string
	// Version: 設定バージョン（リモート設定ストアの revision / etcd version 等）
	Version int64
}

// ConfigClient は L2* 設定取得 interface を宣言する。
// Consul / etcd / ConfigMap 等の設定ストアを Library 独自語彙で抽象化する。
// OSS 型（consul.Client / etcd.Client 等）を引数・戻り値に一切含まない。
type ConfigClient interface {
	// GetString は string 型設定値を取得する。
	// key はドット記法のキー（"service.timeout" 等）。
	// 値が存在しない場合は defaultVal を返す。
	GetString(ctx context.Context, key string, defaultVal string) (string, error)

	// GetInt は int64 型設定値を取得する。
	GetInt(ctx context.Context, key string, defaultVal int64) (int64, error)

	// GetBool は bool 型設定値を取得する。
	GetBool(ctx context.Context, key string, defaultVal bool) (bool, error)

	// GetFloat は float64 型設定値を取得する。
	GetFloat(ctx context.Context, key string, defaultVal float64) (float64, error)

	// GetValue は ConfigValue（メタデータ付き）で設定値を取得する。
	// バージョン管理・監査ログが必要な場合に GetString の代わりに使用する。
	GetValue(ctx context.Context, key string) (*ConfigValue, error)

	// Watch は設定キーの変更を監視して変更時に ch にイベントを送信する。
	// ctx がキャンセルされると監視を停止する（goroutine リークを防ぐため必ず ctx を使用する）。
	Watch(ctx context.Context, key string) (<-chan ConfigValue, error)
}
