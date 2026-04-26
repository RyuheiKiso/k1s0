// 本ファイルは Feature Flag（flagd 直結）のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - Feature API → flagd（Dapr Binding 経由 / 直結のいずれか、リリース時点 中に確定）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/11_Feature_API.md
//
// リリース時点 placeholder。実 flagd 接続は plan 04-13 で実装。

package dapr

// 標準 Go ライブラリ。
import (
	// 全 RPC で context を伝搬する。
	"context"
)

// FlagEvaluateRequest は Flag 評価の入力。
type FlagEvaluateRequest struct {
	// Flag キー（命名規則: <tenant>.<component>.<feature>）。
	FlagKey string
	// 評価コンテキスト（targetingKey は subject と同一）。
	EvaluationContext map[string]string
	// テナント識別子。
	TenantID string
}

// FlagBooleanResponse は boolean 評価応答。
type FlagBooleanResponse struct {
	// 評価値。
	Value bool
	// バリアント名。
	Variant string
	// 評価理由（DEFAULT / TARGETING_MATCH / SPLIT / ERROR）。
	Reason string
}

// FlagStringResponse は string 評価応答。
type FlagStringResponse struct {
	// 評価値。
	Value string
	// バリアント名。
	Variant string
	// 評価理由。
	Reason string
}

// FlagNumberResponse は number 評価応答。
type FlagNumberResponse struct {
	// 評価値。
	Value float64
	// バリアント名。
	Variant string
	// 評価理由。
	Reason string
}

// FlagObjectResponse は object 評価応答。
type FlagObjectResponse struct {
	// 評価値（JSON シリアライズ済 bytes）。
	ValueJSON []byte
	// バリアント名。
	Variant string
	// 評価理由。
	Reason string
}

// FeatureAdapter は Feature Flag 操作の interface。
type FeatureAdapter interface {
	// boolean 評価。
	EvaluateBoolean(ctx context.Context, req FlagEvaluateRequest) (FlagBooleanResponse, error)
	// string 評価。
	EvaluateString(ctx context.Context, req FlagEvaluateRequest) (FlagStringResponse, error)
	// number 評価。
	EvaluateNumber(ctx context.Context, req FlagEvaluateRequest) (FlagNumberResponse, error)
	// object 評価。
	EvaluateObject(ctx context.Context, req FlagEvaluateRequest) (FlagObjectResponse, error)
}

// daprFeatureAdapter は実装（リリース時点 placeholder）。
type daprFeatureAdapter struct {
	// Dapr Client への参照。
	client *Client
}

// NewFeatureAdapter は FeatureAdapter を生成する。
func NewFeatureAdapter(client *Client) FeatureAdapter {
	// 実装インスタンスを構築する。
	return &daprFeatureAdapter{client: client}
}

// EvaluateBoolean は plan 04-13 で実装。
func (a *daprFeatureAdapter) EvaluateBoolean(_ context.Context, _ FlagEvaluateRequest) (FlagBooleanResponse, error) {
	// placeholder
	return FlagBooleanResponse{}, ErrNotWired
}

// EvaluateString は plan 04-13 で実装。
func (a *daprFeatureAdapter) EvaluateString(_ context.Context, _ FlagEvaluateRequest) (FlagStringResponse, error) {
	// placeholder
	return FlagStringResponse{}, ErrNotWired
}

// EvaluateNumber は plan 04-13 で実装。
func (a *daprFeatureAdapter) EvaluateNumber(_ context.Context, _ FlagEvaluateRequest) (FlagNumberResponse, error) {
	// placeholder
	return FlagNumberResponse{}, ErrNotWired
}

// EvaluateObject は plan 04-13 で実装。
func (a *daprFeatureAdapter) EvaluateObject(_ context.Context, _ FlagEvaluateRequest) (FlagObjectResponse, error) {
	// placeholder
	return FlagObjectResponse{}, ErrNotWired
}
