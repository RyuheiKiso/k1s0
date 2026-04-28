// 本ファイルは Feature Flag（flagd）のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - Feature API → flagd（Dapr Configuration component の Backend として flagd を使う）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/11_Feature_API.md
//
// 役割（plan 04-13 結線済、Configuration API ベース）:
//   handler.go が呼び出す flag 評価を Dapr SDK の GetConfigurationItem で実装する。
//   Dapr Configuration API は単一 string 値を返すため、boolean / number / object は
//   adapter 側で string → 目的型へパースする。リッチな variant / reason は Dapr が
//   exposing しないため adapter 側で導出する:
//     - Variant: ConfigurationItem.Version を流用（component 側で flag バリアント識別子を
//       Version field に格納する運用を前提）。空なら "default" を使う
//     - Reason:  ConfigurationItem.Metadata["reason"] があれば使用、なければ "TARGETING_MATCH"
//                （値が取得できれば一致したとみなす慣用）。値未存在なら "DEFAULT"
//
// 旧バージョン置換:
//   plan 04-13 でリッチな evaluation API（OpenFeature flagd provider 等）に切替予定。
//   その際は本実装を OpenFeature SDK call に差し替える（narrow interface 不変）。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"
	// boolean / number パースに使う。
	"strconv"
)

// store 名は flagd Configuration component の Dapr Component 名と整合させる。
// k1s0 既定は infra/feature-management/flagd/ で flagd-default として deploy する想定。
const featureConfigStore = "flagd-default"

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
	Value   bool
	Variant string
	Reason  string
}

// FlagStringResponse は string 評価応答。
type FlagStringResponse struct {
	Value   string
	Variant string
	Reason  string
}

// FlagNumberResponse は number 評価応答。
type FlagNumberResponse struct {
	Value   float64
	Variant string
	Reason  string
}

// FlagObjectResponse は object 評価応答。
type FlagObjectResponse struct {
	// JSON シリアライズ済 bytes（Dapr Configuration が string で返したものをそのまま渡す）。
	ValueJSON []byte
	Variant   string
	Reason    string
}

// FeatureAdapter は Feature Flag 操作の interface。
type FeatureAdapter interface {
	EvaluateBoolean(ctx context.Context, req FlagEvaluateRequest) (FlagBooleanResponse, error)
	EvaluateString(ctx context.Context, req FlagEvaluateRequest) (FlagStringResponse, error)
	EvaluateNumber(ctx context.Context, req FlagEvaluateRequest) (FlagNumberResponse, error)
	EvaluateObject(ctx context.Context, req FlagEvaluateRequest) (FlagObjectResponse, error)
}

// daprFeatureAdapter は Client（narrow interface）越しに SDK を呼ぶ実装。
type daprFeatureAdapter struct {
	client *Client
}

// NewFeatureAdapter は FeatureAdapter を生成する。
func NewFeatureAdapter(client *Client) FeatureAdapter {
	return &daprFeatureAdapter{client: client}
}

// resolveVariantReason は ConfigurationItem から Variant / Reason を導出する。
// ConfigurationItem == nil の時は ("", "DEFAULT") を返し、値ありなら
// Version を Variant に、metadata["reason"] か "TARGETING_MATCH" を Reason に使う。
func resolveVariantReason(item interface{ GetVersion() string; GetReason() string }) (string, string) {
	v := item.GetVersion()
	if v == "" {
		v = "default"
	}
	r := item.GetReason()
	if r == "" {
		r = "TARGETING_MATCH"
	}
	return v, r
}

// configItemView は ConfigurationItem を resolveVariantReason 用に薄くラップする。
// Dapr SDK の ConfigurationItem は public field なので getter を提供して interface 化する。
type configItemView struct {
	version string
	reason  string
}

func (v configItemView) GetVersion() string { return v.version }
func (v configItemView) GetReason() string  { return v.reason }

// fetchItem は flagd から FlagKey の Configuration を取得し、ConfigurationItem を返す。
// 評価コンテキストは Dapr Configuration API では metadata 経由で渡す（component 依存）。
func (a *daprFeatureAdapter) fetchItem(ctx context.Context, req FlagEvaluateRequest) (string, configItemView, bool, error) {
	// metadata 合成（テナント + 評価コンテキスト）。Component 側が targetingKey 等を解釈する。
	meta := make(map[string]string, len(req.EvaluationContext)+1)
	for k, v := range req.EvaluationContext {
		meta[k] = v
	}
	if req.TenantID != "" {
		meta[metadataKeyTenant] = req.TenantID
	}
	// metadata は ConfigurationOpt として渡す必要がある場合があるが、SDK の現行 API は
	// keys だけで metadata 引数を取らない設計のため、本実装は flag-key のみで取得する。
	// metadata 伝搬は component 側で sidecar metadata を使う運用に委ねる（plan 04-13 で詰める）。
	item, err := a.client.configClient().GetConfigurationItem(ctx, featureConfigStore, req.FlagKey)
	if err != nil {
		return "", configItemView{}, false, err
	}
	if item == nil {
		// 値未存在: 既定値返却用に空文字 + DEFAULT 理由。
		return "", configItemView{version: "default", reason: "DEFAULT"}, false, nil
	}
	// metadata から reason を取り出す（component 側の規約依存）。
	reason := item.Metadata["reason"]
	return item.Value, configItemView{version: item.Version, reason: reason}, true, nil
}

// EvaluateBoolean は flagd から bool 値を取得する。
// flagd 側で "true" / "false" の string として保存される運用を前提とする。
func (a *daprFeatureAdapter) EvaluateBoolean(ctx context.Context, req FlagEvaluateRequest) (FlagBooleanResponse, error) {
	value, view, found, err := a.fetchItem(ctx, req)
	if err != nil {
		return FlagBooleanResponse{}, err
	}
	if !found {
		variant, reason := resolveVariantReason(view)
		return FlagBooleanResponse{Value: false, Variant: variant, Reason: reason}, nil
	}
	parsed, perr := strconv.ParseBool(value)
	if perr != nil {
		// パース失敗時は false + ERROR reason。
		return FlagBooleanResponse{Value: false, Variant: view.version, Reason: "ERROR"}, nil
	}
	variant, reason := resolveVariantReason(view)
	return FlagBooleanResponse{Value: parsed, Variant: variant, Reason: reason}, nil
}

// EvaluateString は flagd から string 値を取得する。
func (a *daprFeatureAdapter) EvaluateString(ctx context.Context, req FlagEvaluateRequest) (FlagStringResponse, error) {
	value, view, found, err := a.fetchItem(ctx, req)
	if err != nil {
		return FlagStringResponse{}, err
	}
	if !found {
		variant, reason := resolveVariantReason(view)
		return FlagStringResponse{Value: "", Variant: variant, Reason: reason}, nil
	}
	variant, reason := resolveVariantReason(view)
	return FlagStringResponse{Value: value, Variant: variant, Reason: reason}, nil
}

// EvaluateNumber は flagd から数値を取得する。
// flagd 側で 64-bit float に decode 可能な string として保存される運用を前提とする。
func (a *daprFeatureAdapter) EvaluateNumber(ctx context.Context, req FlagEvaluateRequest) (FlagNumberResponse, error) {
	value, view, found, err := a.fetchItem(ctx, req)
	if err != nil {
		return FlagNumberResponse{}, err
	}
	if !found {
		variant, reason := resolveVariantReason(view)
		return FlagNumberResponse{Value: 0, Variant: variant, Reason: reason}, nil
	}
	parsed, perr := strconv.ParseFloat(value, 64)
	if perr != nil {
		return FlagNumberResponse{Value: 0, Variant: view.version, Reason: "ERROR"}, nil
	}
	variant, reason := resolveVariantReason(view)
	return FlagNumberResponse{Value: parsed, Variant: variant, Reason: reason}, nil
}

// EvaluateObject は flagd から JSON encoded object を取得する。
// flagd Component が JSON 文字列のまま Value に格納する運用を前提とする。
func (a *daprFeatureAdapter) EvaluateObject(ctx context.Context, req FlagEvaluateRequest) (FlagObjectResponse, error) {
	value, view, found, err := a.fetchItem(ctx, req)
	if err != nil {
		return FlagObjectResponse{}, err
	}
	if !found {
		variant, reason := resolveVariantReason(view)
		return FlagObjectResponse{ValueJSON: nil, Variant: variant, Reason: reason}, nil
	}
	variant, reason := resolveVariantReason(view)
	return FlagObjectResponse{ValueJSON: []byte(value), Variant: variant, Reason: reason}, nil
}
