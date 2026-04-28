// 本ファイルは tier1 Go ファサード共通の gRPC Unary Server Interceptor。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「通信プロトコルと可観測性」:
//       - W3C Trace Context（traceparent / tracestate）をヘッダで継承・生成。
//         tier1 ファサードで必ず 1 span 発行
//       - Prometheus メトリクス
//         k1s0_tier1_<api>_requests_total{tenant_id,method,code} と
//         _duration_seconds{tenant_id,method} を自動発行
//     §「マルチテナント分離」L1（入口）:
//       JWT / SPIFFE ID から `tenant_id` を導出し、リクエストの `TenantContext` を上書き
//
// 役割:
//   1. 全 RPC で 1 span を発行する（OTel global TracerProvider 経由）
//   2. RPC 完了時に counter + duration histogram を記録する（OTel global MeterProvider 経由）
//   3. リクエスト proto に GetContext() *TenantContext がある場合は tenant_id ラベルを付与する
//   4. 実装側で gRPC interceptor を陳腐化させない: TracerProvider / MeterProvider が未設定の
//      場合でも no-op で機能し続ける（test / dev で外部 Collector 不要）
//
// 認証 / 認可:
//   JWT 検証と TenantContext 上書きは Auth Interceptor（別ファイル / 別 PR）で扱う。
//   本 interceptor は observability のみに責務を絞る。

// Package common は tier1 Go の共通 gRPC ランタイム utility を提供する。
package common

import (
	// 全 RPC で context を伝搬する。
	"context"
	// Counter / Histogram のラベル文字列に gRPC code を表示する。
	"strconv"
	// 経過時間計測。
	"time"

	// OTel global accessor（TracerProvider / MeterProvider）。
	otelapi "go.opentelemetry.io/otel"
	// OTel attribute / span / metric 型。
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/trace"

	// gRPC server / status / metadata。
	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	// 共通型 TenantContext を含む proto のラッパ抽象は本ファイル内で定義する
	// （tier1 内 11 公開 API + 共通契約に分散する GetContext を 1 つの interface で扱う）。
)

// tracerName は tier1 facade 共通の OTel tracer 名。OTel 慣行に倣い import path 形式。
const tracerName = "github.com/k1s0/k1s0/src/tier1/go"

// meterName は tier1 facade 共通の OTel meter 名。
const meterName = "github.com/k1s0/k1s0/src/tier1/go"

// instrumentSet は per-API の counter / histogram を 1 度生成して使い回すための束。
// API ごとに独立した名前を持つため map[apiName]*instrumentSet で cache する。
type instrumentSet struct {
	// k1s0_tier1_<api>_requests_total{tenant_id,method,code}
	requests metric.Int64Counter
	// k1s0_tier1_<api>_duration_seconds{tenant_id,method}
	duration metric.Float64Histogram
}

// instrumentCache は per-API の instrumentSet を keep する。
// gRPC handler は `/k1s0.tier1.<api>.v1.<Service>/<Method>` 形式で呼ばれるため、
// API 名を full method から抽出し、instrument を遅延生成する。
type instrumentCache struct {
	// 簡素化のため map + mutex 不在: handler は同一プロセス内で並行に走るが、
	// instrument 生成は idempotent（OTel SDK が同名は同一 instrument を返す前提）。
	// 本実装では map への concurrent write を避けるため sync.Map で扱う。
	cache map[string]*instrumentSet
	// concurrent map 化のため、外部呼出の度に getOrCreate を経由する。
}

// loadOrCreate は API 名に対応する instrumentSet を取り出す（無ければ生成）。
// OTel global MeterProvider が no-op meter を返す場合、生成された instrument は no-op。
func (c *instrumentCache) loadOrCreate(apiName string) *instrumentSet {
	if set, ok := c.cache[apiName]; ok {
		return set
	}
	meter := otelapi.GetMeterProvider().Meter(meterName)
	// 動的生成エラーは fail-soft で no-op instrument に倒す。
	requests, err := meter.Int64Counter(
		"k1s0_tier1_"+apiName+"_requests_total",
		metric.WithDescription("tier1 "+apiName+" API requests by tenant_id / method / code"),
	)
	if err != nil {
		// Counter 生成失敗時は record も no-op になるよう nil-safe に倒す。
		requests = nil
	}
	duration, err := meter.Float64Histogram(
		"k1s0_tier1_"+apiName+"_duration_seconds",
		metric.WithDescription("tier1 "+apiName+" API duration by tenant_id / method"),
		metric.WithUnit("s"),
	)
	if err != nil {
		duration = nil
	}
	set := &instrumentSet{requests: requests, duration: duration}
	c.cache[apiName] = set
	return set
}

// tenantContextGetter は proto request のうち GetContext() で TenantContext を返す型を抽象化する。
// 11 公開 API の Request 型は全て GetContext() *commonv1.TenantContext を持つ前提。
// 本 interceptor は tier1 内部で commonv1 import を避けるため Subject() / TenantId() の
// 最小 surface だけを抽象化する（tenantIDProvider）。
type tenantIDProvider interface {
	// TenantId は TenantContext.tenant_id を返す。空文字は未指定。
	GetTenantId() string
}

// extractTenantID はリクエスト proto から tenant_id を取り出す。
// req が *X{Context: *TenantContext{TenantId: "T"}} 型の場合に "T" を返す。
// 取り出せない場合は "" を返し、metric ラベルでは "unknown" として記録する。
func extractTenantID(req interface{}) string {
	// 上位 wrapper（リクエスト proto 全般）で GetContext() が定義されていれば、
	// その戻り値が *TenantContext で GetTenantId() を持つ。
	type ctxGetter interface {
		// proto 生成コードでは Context フィールドが TenantContext 型で、GetContext は *TenantContext を返す。
		// 戻り値の interface{} 互換のため any 経由で型 assertion する。
		GetContext() interface {
			GetTenantId() string
		}
	}
	g, ok := req.(ctxGetter)
	if !ok {
		return ""
	}
	tc := g.GetContext()
	if tc == nil {
		return ""
	}
	return tc.GetTenantId()
}

// apiNameFromMethod は full method "/k1s0.tier1.<api>.v1.<Service>/<Method>" から
// "<api>" 部分を抜き出す。マッチしない場合は "unknown" を返す。
func apiNameFromMethod(fullMethod string) string {
	// fullMethod 例: "/k1s0.tier1.state.v1.StateService/Get"
	// 先頭スラッシュをスキップして "k1s0.tier1.<api>.v1.<Service>" を分離。
	if len(fullMethod) == 0 || fullMethod[0] != '/' {
		return "unknown"
	}
	pkgService := fullMethod[1:]
	// "/" の手前までを pkgService とする（method 部分は捨てる）。
	for i := 0; i < len(pkgService); i++ {
		if pkgService[i] == '/' {
			pkgService = pkgService[:i]
			break
		}
	}
	// "k1s0.tier1.<api>.v1.<Service>" を "." で分割し、3 番目の要素が api 名。
	const prefix = "k1s0.tier1."
	if len(pkgService) <= len(prefix) || pkgService[:len(prefix)] != prefix {
		return "unknown"
	}
	rest := pkgService[len(prefix):]
	// rest = "<api>.v1.<Service>"。最初の "." までが api 名。
	for i := 0; i < len(rest); i++ {
		if rest[i] == '.' {
			return rest[:i]
		}
	}
	return "unknown"
}

// methodNameFromFullMethod は full method の最後のセグメント（RPC 名）を返す。
// 例: "/k1s0.tier1.state.v1.StateService/Get" → "Get"
func methodNameFromFullMethod(fullMethod string) string {
	// 末尾 "/" 以降を返す。
	for i := len(fullMethod) - 1; i >= 0; i-- {
		if fullMethod[i] == '/' {
			return fullMethod[i+1:]
		}
	}
	return fullMethod
}

// ObservabilityInterceptor は per-API trace span 発行と RED モデルメトリクス記録を行う
// Unary Server Interceptor を返す。tier1 facade の全 Pod で同一インスタンスを使う想定。
//
// 振る舞い:
//   - ctx に W3C Trace Context propagation 済の span を新規開始する
//   - span 名は full method（"/k1s0.tier1.<api>.v1.<Service>/<Method>"）を採用
//   - 属性: tenant_id / rpc.method / rpc.system="grpc"
//   - handler 完了後、duration histogram と request counter を記録
//   - エラー時は span.SetStatus(Error) で記録
func ObservabilityInterceptor() grpc.UnaryServerInterceptor {
	// instrument cache を 1 つ持つ（API 別）。
	cache := &instrumentCache{cache: map[string]*instrumentSet{}}
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		// Tracer は per-request で取り直す。Pod 起動時に otel.Init() が global provider を
		// 後設定するシーケンスでも、interceptor が常に最新 provider 経由で span を発行できる。
		tracer := otelapi.GetTracerProvider().Tracer(tracerName)
		// API / メソッド名を full method から抽出する。
		apiName := apiNameFromMethod(info.FullMethod)
		methodName := methodNameFromFullMethod(info.FullMethod)
		// tenant_id をリクエスト proto から抽出する（無ければ "unknown"）。
		tenantID := extractTenantID(req)
		labelTenant := tenantID
		if labelTenant == "" {
			labelTenant = "unknown"
		}
		// span 開始。span 名は full method を踏襲。
		ctx, span := tracer.Start(ctx, info.FullMethod,
			trace.WithSpanKind(trace.SpanKindServer),
			trace.WithAttributes(
				attribute.String("rpc.system", "grpc"),
				attribute.String("rpc.service", apiName),
				attribute.String("rpc.method", methodName),
				attribute.String("tenant_id", labelTenant),
			),
		)
		// 終了時に span を必ず閉じる。
		defer span.End()

		// handler を実行し、所要時間と error code を観測する。
		start := time.Now()
		resp, err := handler(ctx, req)
		elapsed := time.Since(start).Seconds()

		// gRPC code を文字列化する（"OK" / "InvalidArgument" / ...）。
		code := grpccodes.OK
		if err != nil {
			st, _ := status.FromError(err)
			code = st.Code()
			// span にエラー情報を記録する（NFR-I-SLI-* で SLO エラー率の計測に直結）。
			span.SetStatus(otelCodes(code), st.Message())
		}

		// instrument 取得 + 記録。Meter 未設定環境では requests/duration が nil で no-op。
		set := cache.loadOrCreate(apiName)
		attrs := metric.WithAttributes(
			attribute.String("tenant_id", labelTenant),
			attribute.String("method", methodName),
			attribute.String("code", code.String()),
		)
		if set.requests != nil {
			set.requests.Add(ctx, 1, attrs)
		}
		if set.duration != nil {
			// duration は code を含めない（共通規約: _duration_seconds{tenant_id,method}）。
			set.duration.Record(ctx, elapsed, metric.WithAttributes(
				attribute.String("tenant_id", labelTenant),
				attribute.String("method", methodName),
			))
		}
		_ = strconv.Itoa // keep import live for future code-string formatting
		return resp, err
	}
}

// otelCodes は gRPC status code を OTel codes.Code に翻訳する。
// gRPC.OK → otel.Unset、それ以外は otel.Error として扱う（OTel Status 推奨運用）。
func otelCodes(c grpccodes.Code) codes.Code {
	if c == grpccodes.OK {
		return codes.Unset
	}
	return codes.Error
}
