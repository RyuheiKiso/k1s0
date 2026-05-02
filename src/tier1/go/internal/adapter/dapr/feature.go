// 本ファイルは Feature Flag（flagd）のアダプタ。
//
// 設計正典:
//   docs/02_構想設計/adr/ADR-FM-001-flagd-openfeature.md
//     - "flagd / OpenFeature を採用する"
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/11_Feature_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//
// 役割:
//   handler.go が呼び出す flag 評価を OpenFeature Go SDK + flagd provider 経由で
//   flagd（gRPC :8013）に直接ぶつけて解決する。Dapr Configuration API は flagd
//   バックエンドを持たない（Dapr 1.17.5 時点で `configuration.flagd` 非対応）ため、
//   Configuration 経路ではなく OpenFeature SDK の RPC リゾルバを採用する。
//
// 旧実装との差分（why）:
//   旧 daprFeatureAdapter は SDK の GetConfigurationItem("flagd-default", flagKey) を
//   呼んでいたが、Dapr 側に `flagd-default` という Configuration component を
//   登録できる正規 component type が存在せず、誤って `configuration.flagd` を
//   定義すると namespace 内全 daprd が起動 fatal で死ぬ（`couldn't find configuration
//   store configuration.flagd/v1`）。本実装は ADR-FM-001 の「flagd / OpenFeature」
//   原典に立ち返り、Dapr を経由せず OpenFeature gRPC で flagd に直結する。
//
// 接続先（環境変数で上書き可能）:
//   FLAGD_HOST → flagd の DNS（例: flagd.flagd.svc.cluster.local、既定）
//   FLAGD_PORT → flagd gRPC ポート（既定 8013）
//   FLAGD_TLS  → "true" のとき TLS で繋ぐ（既定 false、cluster 内 plaintext を許容）
//
// テスタビリティ:
//   `openFeatureClient` という narrow interface で `*openfeature.Client` を
//   抽象化し、production では実 SDK Client を、テストでは fake を注入できる。
//   `*openfeature.Client` のメソッドシグネチャをそのまま反映するため、
//   Real Client もそのまま渡せる（NewFlagdFeatureAdapter で wrap）。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"
	// 0 値判定に使う。
	"errors"
	// 起動時の Provider 初期化結果を stderr にログする。
	"log"
	// 環境変数読出し。
	"os"
	// boolean / number パースに使う。
	"strconv"
	// EvaluationContext.targetingKey 解決に使う（subject フォールバック）。

	// OpenFeature Go SDK：Client / EvaluationContext / 各 Details 型を提供する。
	"github.com/open-feature/go-sdk/openfeature"
	// flagd provider：OpenFeature の Provider 実装で gRPC で flagd に繋ぐ。
	flagd "github.com/open-feature/go-sdk-contrib/providers/flagd/pkg"
)

// flagdDefaultHost は本 cluster の flagd Service DNS 既定値。
// FLAGD_HOST 環境変数で上書き可能。
const flagdDefaultHost = "flagd.flagd.svc.cluster.local"

// flagdDefaultPort は flagd の Connect-RPC gRPC 既定 listen port。
// FLAGD_PORT 環境変数で上書き可能。
const flagdDefaultPort = uint16(8013)

// flagdEvalCtxKeyTenant は flagd EvaluationContext に詰める tenant キー名。
// flagd 側のターゲティングルールが `$evaluation.context.tenant` を参照する運用前提。
// state.go の metadataKeyTenant（"tenantId"）と意味が違う（Dapr metadata vs OpenFeature
// attribute）ため別定義にしている。
const flagdEvalCtxKeyTenant = "tenant"

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
	// JSON シリアライズ済 bytes（Object 値を JSON で返す）。
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

// openFeatureClient は本パッケージが OpenFeature SDK Client から「実際に使う」
// 評価メソッドだけを集めた narrow interface。`*openfeature.Client` がこれを満たすため、
// production では Client 実体を、test では fake を差し替え可能。
type openFeatureClient interface {
	BooleanValueDetails(ctx context.Context, flag string, defaultValue bool, evalCtx openfeature.EvaluationContext, options ...openfeature.Option) (openfeature.BooleanEvaluationDetails, error)
	StringValueDetails(ctx context.Context, flag string, defaultValue string, evalCtx openfeature.EvaluationContext, options ...openfeature.Option) (openfeature.StringEvaluationDetails, error)
	FloatValueDetails(ctx context.Context, flag string, defaultValue float64, evalCtx openfeature.EvaluationContext, options ...openfeature.Option) (openfeature.FloatEvaluationDetails, error)
	ObjectValueDetails(ctx context.Context, flag string, defaultValue any, evalCtx openfeature.EvaluationContext, options ...openfeature.Option) (openfeature.InterfaceEvaluationDetails, error)
}

// flagdFeatureAdapter は OpenFeature Client 越しに flagd に繋ぐ実装。
type flagdFeatureAdapter struct {
	// SDK の Client（narrow interface 経由でテスト可）。
	client openFeatureClient
}

// NewFlagdFeatureAdapter は OpenFeature Client から FeatureAdapter を生成する。
// production では `BuildFlagdProvider()` 経由で初期化された Client を渡す。
// テストでは fake openFeatureClient を渡して内部分岐をユニットテスト可能。
func NewFlagdFeatureAdapter(client openFeatureClient) FeatureAdapter {
	return &flagdFeatureAdapter{client: client}
}

// NewFeatureAdapter は後方互換のため残す（旧呼出側が dapr.NewFeatureAdapter(client)
// を期待）。dapr.Client は flagd 経路では使わず、env から flagd Provider を組み立てて
// global SetProviderAndWait → openfeature.NewClient("tier1") で Client を作る。
// 起動時にしか呼ばれない想定なので global 副作用を許容する。
func NewFeatureAdapter(_ *Client) FeatureAdapter {
	provider, perr := BuildFlagdProvider()
	if perr != nil {
		// Provider 構築自体に失敗するのは設定値が異常なケース（ポート文字列パース失敗等）。
		// 既定値で再構築する fallback を試みる。
		log.Printf("tier1/feature: BuildFlagdProvider error: %v; falling back to defaults", perr)
		provider, _ = flagd.NewProvider(
			flagd.WithRPCResolver(),
			flagd.WithHost(flagdDefaultHost),
			flagd.WithPort(flagdDefaultPort),
		)
	}
	if provider != nil {
		// global SetProvider は副作用だが、tier1 facade では process 単一 provider 運用で支障なし。
		// SetProviderAndWait は同期的に Provider 初期化を待つ（init イベントを subscribe する分の手間がない）。
		if err := openfeature.SetProviderAndWait(provider); err != nil {
			// 初期化失敗時はログに残す（呼出側の評価は fail-soft で ERROR reason が返る）。
			log.Printf("tier1/feature: openfeature.SetProviderAndWait error: %v", err)
		} else {
			log.Printf("tier1/feature: openfeature flagd provider initialized")
		}
	}
	return NewFlagdFeatureAdapter(openfeature.NewClient("tier1"))
}

// BuildFlagdProvider は環境変数（FLAGD_HOST / FLAGD_PORT / FLAGD_TLS）から
// flagd Provider を構築する。設定値が未指定なら本ファイル冒頭の既定値を採用する。
// process 起動時に 1 度だけ呼ばれることを想定。
func BuildFlagdProvider() (*flagd.Provider, error) {
	host := os.Getenv("FLAGD_HOST")
	if host == "" {
		host = flagdDefaultHost
	}
	port := flagdDefaultPort
	if v := os.Getenv("FLAGD_PORT"); v != "" {
		// uint16 で parse（0–65535）。負値や 65535 超過は ErrPort を返す。
		n, err := strconv.ParseUint(v, 10, 16)
		if err != nil {
			return nil, errors.New("FLAGD_PORT must be a valid uint16")
		}
		port = uint16(n)
	}
	opts := []flagd.ProviderOption{
		// RPC リゾルバ：flagd の Connect-RPC :8013 経路を使う。
		// In-process リゾルバ（flagd-proxy 経由）も選択肢だが本実装は単純な RPC を選ぶ。
		flagd.WithRPCResolver(),
		flagd.WithHost(host),
		flagd.WithPort(port),
	}
	if v := os.Getenv("FLAGD_TLS"); v == "true" {
		// TLS 有効化（FLAGD_TLS_CA でルート CA が必要な場合は flagd.WithTLS で渡す）。
		caPath := os.Getenv("FLAGD_TLS_CA")
		opts = append(opts, flagd.WithTLS(caPath))
	}
	return flagd.NewProvider(opts...)
}

// makeEvalCtx は FlagEvaluateRequest から openfeature.EvaluationContext を構築する。
// targetingKey は subject（`subject` キー）優先、無ければ tenantID をフォールバック。
// すべての EvaluationContext key は string→any として attribute に積む。
func makeEvalCtx(req FlagEvaluateRequest) openfeature.EvaluationContext {
	// targetingKey の優先順序: subject > targetingKey > tenant。
	targeting := req.EvaluationContext["subject"]
	if targeting == "" {
		targeting = req.EvaluationContext["targetingKey"]
	}
	if targeting == "" {
		targeting = req.TenantID
	}
	attrs := make(map[string]any, len(req.EvaluationContext)+1)
	for k, v := range req.EvaluationContext {
		attrs[k] = v
	}
	if req.TenantID != "" {
		attrs[flagdEvalCtxKeyTenant] = req.TenantID
	}
	return openfeature.NewEvaluationContext(targeting, attrs)
}

// reasonStr は openfeature.Reason を string に正規化する。空時は "DEFAULT"。
func reasonStr(r openfeature.Reason) string {
	if r == "" {
		return "DEFAULT"
	}
	return string(r)
}

// variantOrDefault は variant が空なら "default"、そうでなければそのまま返す。
func variantOrDefault(variant string) string {
	if variant == "" {
		return "default"
	}
	return variant
}

// EvaluateBoolean は flagd から bool 値を取得する。
func (a *flagdFeatureAdapter) EvaluateBoolean(ctx context.Context, req FlagEvaluateRequest) (FlagBooleanResponse, error) {
	d, err := a.client.BooleanValueDetails(ctx, req.FlagKey, false, makeEvalCtx(req))
	if err != nil {
		return FlagBooleanResponse{}, err
	}
	return FlagBooleanResponse{
		Value:   d.Value,
		Variant: variantOrDefault(d.Variant),
		Reason:  reasonStr(d.Reason),
	}, nil
}

// EvaluateString は flagd から string 値を取得する。
func (a *flagdFeatureAdapter) EvaluateString(ctx context.Context, req FlagEvaluateRequest) (FlagStringResponse, error) {
	d, err := a.client.StringValueDetails(ctx, req.FlagKey, "", makeEvalCtx(req))
	if err != nil {
		return FlagStringResponse{}, err
	}
	return FlagStringResponse{
		Value:   d.Value,
		Variant: variantOrDefault(d.Variant),
		Reason:  reasonStr(d.Reason),
	}, nil
}

// EvaluateNumber は flagd から数値（float64）を取得する。
// FlagNumberResponse は float64 専用。整数 flag 用には FlagNumber* で分けるべきだが、
// release-initial では float64 に統一する。
func (a *flagdFeatureAdapter) EvaluateNumber(ctx context.Context, req FlagEvaluateRequest) (FlagNumberResponse, error) {
	d, err := a.client.FloatValueDetails(ctx, req.FlagKey, 0, makeEvalCtx(req))
	if err != nil {
		return FlagNumberResponse{}, err
	}
	return FlagNumberResponse{
		Value:   d.Value,
		Variant: variantOrDefault(d.Variant),
		Reason:  reasonStr(d.Reason),
	}, nil
}

// EvaluateObject は flagd から JSON encoded object を取得する。
// flagd は object を map[string]any 等の生 Go 値で返すため、JSON marshal して
// FlagObjectResponse.ValueJSON に詰め直す。
func (a *flagdFeatureAdapter) EvaluateObject(ctx context.Context, req FlagEvaluateRequest) (FlagObjectResponse, error) {
	d, err := a.client.ObjectValueDetails(ctx, req.FlagKey, nil, makeEvalCtx(req))
	if err != nil {
		return FlagObjectResponse{}, err
	}
	jsonBytes, mErr := jsonMarshal(d.Value)
	if mErr != nil {
		// Marshal 失敗は ERROR として扱い、空 JSON を返す（fail-soft 経路は handler 側）。
		return FlagObjectResponse{
			ValueJSON: nil,
			Variant:   variantOrDefault(d.Variant),
			Reason:    "ERROR",
		}, nil
	}
	return FlagObjectResponse{
		ValueJSON: jsonBytes,
		Variant:   variantOrDefault(d.Variant),
		Reason:    reasonStr(d.Reason),
	}, nil
}
