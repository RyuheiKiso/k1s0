// 本ファイルは t1-state Pod の InvokeService 2 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/01_Service_Invoke_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/01_Service_Invoke_API.md
//     - FR-T1-INVOKE-001: 同期サービス呼び出し（gRPC）
//     - FR-T1-INVOKE-003: タイムアウト・リトライ制御
//     - FR-T1-INVOKE-005: 認証トークン自動伝搬
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「タイムアウトとデッドライン伝播」
//
// 役割:
//   - request.TimeoutMs を context.WithTimeout で適用し、未指定時は共通規約の既定値（3 秒）を使う
//   - 呼出元 incoming context の Authorization / W3C Trace Context メタデータを
//     outgoing context に転写する（FR-T1-INVOKE-005「呼出元 request の Authorization
//     ヘッダが呼び出し先 request に自動付与される」）
//   - Service Invocation 自体は呼出先サービスの副作用が不明なため tier1 facade では
//     auto-retry を行わない（共通規約「クライアント SDK が retry を担う」）。retry
//     ポリシーは呼出元 SDK 側の責務とする

package state

import (
	// context 伝搬と timeout 制御。
	"context"
	// 既定 timeout の Duration 計算。
	"time"

	// Dapr adapter。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// FR-T1-INVOKE-004 Circuit Breaker。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// SDK 生成 stub の InvokeService 型。
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// 認証ヘッダ転写用の outgoing metadata。
	"google.golang.org/grpc/metadata"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// invokeHandler は InvokeService の handler 実装。
type invokeHandler struct {
	// 将来 RPC 用の埋め込み。
	serviceinvokev1.UnimplementedInvokeServiceServer
	// adapter 集合への参照。
	deps Deps
}

// invokeCBRetryAfterMs は Open 状態で caller に返す retry_after_ms（共通規約 §「エラー型」）。
// Circuit Breaker の half-open 待機時間を ms に変換した値で返す。
func (h *invokeHandler) invokeCBRetryAfterMs(cb *common.CircuitBreaker) int64 {
	return cb.HalfOpenAfter().Milliseconds()
}

// invokeDefaultTimeout は共通規約 §「タイムアウトとデッドライン伝播」の既定 3 秒。
// request.TimeoutMs == 0 の場合に適用される。
const invokeDefaultTimeout = 3 * time.Second

// invokeMaxTimeout は呼出元から指定可能な最大 timeout。共通規約の既定 3 秒は
// 短期サービス呼び出し向けで、Workflow.Start（30 秒）/ Binding.Send（10 秒）の
// 別 API 経路と比較しても 60 秒を超える呼び出しは tier1 facade の責務外として弾く。
const invokeMaxTimeout = 60 * time.Second

// invokeForwardedHeaders は呼出元 → 呼出先に自動転写するヘッダ集合。
// FR-T1-INVOKE-005「呼出元 request の Authorization ヘッダが呼び出し先 request に
// 自動付与される」「W3C Trace Context が tier1 ファサードが自動で伝搬する」。
//
// gRPC metadata key は小文字正規化されているため、ここでも小文字で並べる。
var invokeForwardedHeaders = []string{
	// JWT Bearer 等の認証ヘッダ。
	"authorization",
	// W3C Trace Context（traceparent / tracestate）。
	"traceparent",
	"tracestate",
	// Baggage（OpenTelemetry の業務情報伝搬）。
	"baggage",
}

// applyInvokeTimeout は request.TimeoutMs を context.WithTimeout で適用する。
//
// 動作:
//   - timeoutMs == 0: 既定 3 秒を適用
//   - 0 < timeoutMs <= invokeMaxTimeout: 指定値を適用
//   - timeoutMs > invokeMaxTimeout: InvalidArgument を返す（爆撃防止）
//
// 親 context にすでに deadline がある場合、context.WithTimeout は早い方を採用する。
func applyInvokeTimeout(parent context.Context, timeoutMs int32) (context.Context, context.CancelFunc, error) {
	// 不正値（負）は InvalidArgument。
	if timeoutMs < 0 {
		// nil cancel と error を返す。
		return parent, func() {}, status.Errorf(codes.InvalidArgument, "tier1/serviceinvoke: timeout_ms must be >= 0, got %d", timeoutMs)
	}
	// 既定値の決定。
	timeout := invokeDefaultTimeout
	// 明示指定がある場合は上書き。
	if timeoutMs > 0 {
		// 過大値は弾く（DoS 防止）。
		if time.Duration(timeoutMs)*time.Millisecond > invokeMaxTimeout {
			return parent, func() {}, status.Errorf(codes.InvalidArgument, "tier1/serviceinvoke: timeout_ms %d exceeds maximum %d", timeoutMs, int32(invokeMaxTimeout/time.Millisecond))
		}
		// ms → Duration へ変換する。
		timeout = time.Duration(timeoutMs) * time.Millisecond
	}
	// timeout を被せた child context を返す。
	ctx, cancel := context.WithTimeout(parent, timeout)
	// 呼出側 defer で cancel を呼ぶ前提。
	return ctx, cancel, nil
}

// withForwardedAuthMetadata は incoming gRPC metadata から FR-T1-INVOKE-005 の
// 転写対象ヘッダを取り出して outgoing context に詰め直す。
//
// 動作:
//   - incoming context に metadata が存在しない（HTTP gateway 経由など）場合は
//     parent をそのまま返す
//   - 既存の outgoing metadata を尊重しつつ、不在キーのみ補完する（呼出側が
//     明示的に上書きした場合は壊さない）
func withForwardedAuthMetadata(parent context.Context) context.Context {
	// incoming metadata を取り出す（gRPC 経路）。
	in, ok := metadata.FromIncomingContext(parent)
	if !ok || in.Len() == 0 {
		// HTTP gateway 経路など metadata 不在の場合はそのまま返す。
		return parent
	}
	// 既存の outgoing metadata を尊重するため Copy で結合する。
	out, _ := metadata.FromOutgoingContext(parent)
	if out == nil {
		out = metadata.MD{}
	}
	// 転写対象キーを 1 つずつコピーする。
	for _, key := range invokeForwardedHeaders {
		// すでに outgoing 側に値があれば呼出側の意図を尊重して上書きしない。
		if existing := out.Get(key); len(existing) > 0 {
			continue
		}
		// incoming 側に値があればコピーする。
		if vs := in.Get(key); len(vs) > 0 {
			out.Set(key, vs...)
		}
	}
	// 転写後 outgoing context を作って返す。
	return metadata.NewOutgoingContext(parent, out)
}

// Invoke は任意サービスの任意メソッド呼出。
func (h *invokeHandler) Invoke(ctx context.Context, req *serviceinvokev1.InvokeRequest) (*serviceinvokev1.InvokeResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/serviceinvoke: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Invoke.Invoke")
	if err != nil {
		return nil, err
	}
	// 必須入力（app_id / method）の事前検証。空のまま Dapr に流すと InvokeMethodWithCustomContent
	// が plain error を返し codes.Internal に潰れるため handler 段で弾く。
	if req.GetAppId() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/serviceinvoke: app_id required")
	}
	if req.GetMethod() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/serviceinvoke: method required")
	}
	// FR-T1-INVOKE-003: TimeoutMs を context.WithTimeout で適用する。
	ctx, cancel, terr := applyInvokeTimeout(ctx, req.GetTimeoutMs())
	if terr != nil {
		return nil, terr
	}
	defer cancel()
	// FR-T1-INVOKE-005: 認証ヘッダを呼出先 request に自動転写する。
	ctx = withForwardedAuthMetadata(ctx)
	// FR-T1-INVOKE-004: Circuit Breaker（appId 単位）が closed/half-open のときのみ呼出許可。
	// open のときは即座に Unavailable + retry_after_ms を返し、下流障害の連鎖を切る。
	cb := h.cbForTarget(req.GetAppId())
	if cb != nil && !cb.Allow() {
		return nil, status.Errorf(codes.Unavailable,
			"tier1/serviceinvoke: circuit breaker open for app_id=%q (retry_after_ms=%d)",
			req.GetAppId(), h.invokeCBRetryAfterMs(cb))
	}
	// adapter 入力に変換。
	areq := dapr.InvokeRequest{
		// 呼出先アプリ識別子。
		AppID: req.GetAppId(),
		// メソッド名。
		Method: req.GetMethod(),
		// データ本文。
		Data: req.GetData(),
		// Content-Type。
		ContentType: req.GetContentType(),
		// テナント。
		TenantID: tid,
		// タイムアウト（adapter には参考情報として渡すが、context deadline で上書きされる）。
		TimeoutMs: req.GetTimeoutMs(),
	}
	// adapter 呼出。
	aresp, err := h.deps.InvokeAdapter.Invoke(ctx, areq)
	// エラー翻訳と CB 更新。
	if err != nil {
		// FR-T1-INVOKE-004: Unavailable / DeadlineExceeded のみ failure としてカウントする。
		// PermissionDenied 等の業務エラーで CB を開けると過剰反応になる。
		if cb != nil && isCBFailure(err) {
			cb.RecordFailure()
		} else if cb != nil {
			// 業務エラーは breaker から見て「成功」（=down じゃない）として記録する。
			cb.RecordSuccess()
		}
		// 翻訳 helper（state.go 定義）を invoke 用にカスタマイズ。
		return nil, translateInvokeErr(err, "Invoke")
	}
	// 成功を CB に記録。
	if cb != nil {
		cb.RecordSuccess()
	}
	// proto 応答に変換して返却する。
	return &serviceinvokev1.InvokeResponse{
		// 応答本文。
		Data: aresp.Data,
		// Content-Type。
		ContentType: aresp.ContentType,
		// HTTP ステータス相当。
		Status: aresp.Status,
	}, nil
}

// cbForTarget は appId 単位の CircuitBreaker を取得する。Registry 未注入時は nil を返す。
func (h *invokeHandler) cbForTarget(appID string) *common.CircuitBreaker {
	if h.deps.InvokeCircuitBreakers == nil {
		return nil
	}
	return h.deps.InvokeCircuitBreakers.Get(appID)
}

// isCBFailure は Circuit Breaker から見て「下流障害」と扱うエラーかを判定する。
// Unavailable / DeadlineExceeded のみが対象。InvalidArgument / PermissionDenied 等の
// 呼出側起因エラーで CB を開けると業務ロジック誤りで全呼出が遮断される過剰反応になる。
func isCBFailure(err error) bool {
	st, ok := status.FromError(err)
	if !ok {
		// gRPC status を持たない plain error は Unavailable と同等として扱う。
		return true
	}
	switch st.Code() {
	case codes.Unavailable, codes.DeadlineExceeded, codes.ResourceExhausted, codes.Internal:
		return true
	default:
		return false
	}
}

// chunkSize は InvokeStream で応答 bytes を分割するときのデフォルトチャンクサイズ（4 KiB）。
// gRPC のフレーム上限 (default 4 MiB) よりはるかに小さく、レイテンシよりスループット優先。
const invokeStreamChunkSize = 4 * 1024

// InvokeStream は server-streaming RPC。Dapr SDK の InvokeMethod は完全な
// streaming を直接公開しないため、まず adapter.Invoke で全 bytes を取得し、
// それをチャンク分割して stream.Send する。proto 契約（stream InvokeChunk + eof
// フラグ）を満たす最小実装。upstream が真の streaming に対応した時点で本実装を
// 直接 streaming proxy に置き換える（adapter interface 不変）。
func (h *invokeHandler) InvokeStream(req *serviceinvokev1.InvokeRequest, stream serviceinvokev1.InvokeService_InvokeStreamServer) error {
	if req == nil {
		return status.Error(codes.InvalidArgument, "tier1/serviceinvoke: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, terr := requireTenantIDFromCtx(stream.Context(), req.GetContext(), "Invoke.InvokeStream")
	if terr != nil {
		return terr
	}
	if req.GetAppId() == "" {
		return status.Error(codes.InvalidArgument, "tier1/serviceinvoke: app_id required")
	}
	if req.GetMethod() == "" {
		return status.Error(codes.InvalidArgument, "tier1/serviceinvoke: method required")
	}
	// FR-T1-INVOKE-003: stream context にも timeout を適用する。
	ctx, cancel, applyErr := applyInvokeTimeout(stream.Context(), req.GetTimeoutMs())
	if applyErr != nil {
		return applyErr
	}
	defer cancel()
	// FR-T1-INVOKE-005: 認証ヘッダを転写する。
	ctx = withForwardedAuthMetadata(ctx)
	areq := dapr.InvokeRequest{
		AppID:       req.GetAppId(),
		Method:      req.GetMethod(),
		Data:        req.GetData(),
		ContentType: req.GetContentType(),
		TenantID:    tid,
		TimeoutMs:   req.GetTimeoutMs(),
	}
	aresp, err := h.deps.InvokeAdapter.Invoke(ctx, areq)
	if err != nil {
		return translateInvokeErr(err, "InvokeStream")
	}
	body := aresp.Data
	// 本文が空なら eof=true の単一チャンクを 1 件だけ送る（proto 契約に沿う）。
	if len(body) == 0 {
		return stream.Send(&serviceinvokev1.InvokeChunk{Eof: true})
	}
	for offset := 0; offset < len(body); offset += invokeStreamChunkSize {
		end := offset + invokeStreamChunkSize
		if end > len(body) {
			end = len(body)
		}
		eof := end == len(body)
		if err := stream.Send(&serviceinvokev1.InvokeChunk{
			Data: body[offset:end],
			Eof:  eof,
		}); err != nil {
			return status.Errorf(codes.Internal, "tier1/serviceinvoke: stream.Send: %v", err)
		}
	}
	return nil
}

// translateInvokeErr は ServiceInvoke 用のエラー翻訳。
func translateInvokeErr(err error, rpc string) error {
	// dapr が返す gRPC status を尊重する。serviceinvoke は対象 service が gRPC で
	// status を返すケース（NotFound / PermissionDenied / Unavailable 等）が多く、
	// それらを Internal に潰すと client は適切な再試行判定ができない。
	if st, ok := status.FromError(err); ok && st.Code() != codes.Unknown && st.Code() != codes.OK {
		return status.Errorf(st.Code(), "tier1/serviceinvoke: %s adapter error: %s", rpc, st.Message())
	}
	// 想定外エラーは Internal。
	return status.Errorf(codes.Internal, "tier1/serviceinvoke: %s adapter error: %v", rpc, err)
}

