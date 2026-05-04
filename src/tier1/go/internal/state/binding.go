// 本ファイルは t1-state Pod の BindingService 1 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/05_Binding_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/05_Binding_API.md
//
// 関連要件:
//   FR-T1-BINDING-001 (MinIO Output Binding)
//   FR-T1-BINDING-002 (SMTP Output Binding)
//   FR-T1-BINDING-003 (HTTP Output Binding)
//   FR-T1-BINDING-004 (Cron Input Binding) — Component YAML のみ提供、handler 経路は SDK 採用初期で結線
//
// 自動 Audit 連動:
//   common/audit.go の AuditInterceptor が Binding.Invoke を privileged RPC として
//   自動 emit する (NFR-E-MON-002)。handler 側で個別の Audit.Record 呼出は不要。

package state

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// Dapr adapter。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// 共通 idempotency cache（共通規約 §「冪等性と再試行」）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// SDK 生成 stub の BindingService 型。
	bindingv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/binding/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// bindingMaxObjectSize は FR-T1-BINDING-001 受け入れ基準「オブジェクトサイズ上限 5GB」。
// MinIO multipart upload を内部で使用する場合でも、handler 段で受信サイズの上限を
// 弾いて Pod メモリ使用量を保護する。
const bindingMaxObjectSize = 5 * 1024 * 1024 * 1024 // 5 GiB

// Component name prefix → Binding 種別の対応。docstring で示した命名規約に基づき、
// handler 段で必須 metadata の事前検証を行うために使う。
const (
	// FR-T1-BINDING-002: SMTP Output Binding を識別する name prefix。
	bindingNamePrefixSMTP = "smtp-"
	// FR-T1-BINDING-003: HTTP Output Binding を識別する name prefix。
	bindingNamePrefixHTTP = "http-"
)

// SMTP / HTTP Binding が要求する必須 metadata key。
const (
	// FR-T1-BINDING-002: SMTP の宛先メールアドレス (Dapr bindings.smtp 標準キー)。
	bindingMetaKeySMTPEmailTo = "emailTo"
	// FR-T1-BINDING-002: SMTP の件名 (Dapr bindings.smtp 標準キー)。
	bindingMetaKeySMTPSubject = "subject"
	// FR-T1-BINDING-003: HTTP の endpoint path (host は Component 側で固定)。
	bindingMetaKeyHTTPPath = "path"
)

// bindingHandler は BindingService の handler 実装。
type bindingHandler struct {
	// 将来 RPC 用埋め込み。
	bindingv1.UnimplementedBindingServiceServer
	// adapter 集合。
	deps Deps
}

// Invoke は出力バインディング呼出。
// 共通規約 §「冪等性と再試行」: idempotency_key 指定時は外部送信（SMTP / S3 等）の
// 重複を防ぐため、同一キーの再試行で初回 InvokeBindingResponse を返す。
func (h *bindingHandler) Invoke(ctx context.Context, req *bindingv1.InvokeBindingRequest) (*bindingv1.InvokeBindingResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/binding: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Binding.Invoke")
	if err != nil {
		return nil, err
	}
	// 必須入力の事前検証（adapter 越しに dapr SDK が返す errors.New("...required") を
	// codes.Internal として上位に漏らさないよう、handler で InvalidArgument として弾く）。
	if req.GetName() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/binding: name required")
	}
	if req.GetOperation() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/binding: operation required")
	}
	// FR-T1-BINDING-001: オブジェクトサイズ上限 5GB を handler で弾く。
	// Pod メモリ保護と DoS 防止のため、adapter 呼出前に容量を検証する。
	if len(req.GetData()) > bindingMaxObjectSize {
		return nil, status.Errorf(codes.ResourceExhausted,
			"tier1/binding: data size %d exceeds maximum %d (5 GiB)", len(req.GetData()), bindingMaxObjectSize)
	}
	// FR-T1-BINDING-002 / 003: Component 種別ごとの必須 metadata 検証。
	// Dapr 側で missing metadata は codes.Internal に潰れがちなので handler 段で
	// InvalidArgument に統一する (schemathesis E2 で同種の修正経緯あり)。
	if err := validateBindingMetadata(req.GetName(), req.GetMetadata()); err != nil {
		return nil, err
	}
	// 実 Invoke 実行クロージャ。idempotency cache hit 時は呼ばれない。
	doInvoke := func() (interface{}, error) {
		areq := dapr.BindingRequest{
			Name:      req.GetName(),
			Operation: req.GetOperation(),
			Data:      req.GetData(),
			Metadata:  req.GetMetadata(),
			TenantID:  tid,
		}
		aresp, err := h.deps.BindingAdapter.Invoke(ctx, areq)
		if err != nil {
			// dapr が返す gRPC status を尊重する（PermissionDenied / FailedPrecondition 等）
			if st, ok := status.FromError(err); ok && st.Code() != codes.Unknown && st.Code() != codes.OK {
				return nil, status.Errorf(st.Code(), "tier1/binding: Invoke adapter error: %s", st.Message())
			}
			return nil, status.Errorf(codes.Internal, "tier1/binding: Invoke adapter error: %v", err)
		}
		return &bindingv1.InvokeBindingResponse{
			Data:     aresp.Data,
			Metadata: aresp.Metadata,
		}, nil
	}
	idempKey := common.IdempotencyKey(tid, "Binding.Invoke", req.GetIdempotencyKey())
	if idempKey == "" || h.deps.Idempotency == nil {
		resp, err := doInvoke()
		if err != nil {
			return nil, err
		}
		return resp.(*bindingv1.InvokeBindingResponse), nil
	}
	resp, err := h.deps.Idempotency.GetOrCompute(ctx, idempKey, doInvoke)
	if err != nil {
		return nil, err
	}
	return resp.(*bindingv1.InvokeBindingResponse), nil
}

// validateBindingMetadata は Component 種別ごとの必須 metadata を handler 段で検証する。
// Component 種別の識別は name の prefix で行う:
//   - "smtp-..." → FR-T1-BINDING-002 SMTP (emailTo / subject 必須)
//   - "http-..." → FR-T1-BINDING-003 HTTP (path 必須、host は Component で固定)
//   - その他 (minio-* / s3-* / cron-* / 任意 generic) → 検証 skip (Dapr に委譲)
//
// 仕様根拠: docs/03_要件定義/20_機能要件/10_tier1_API要件/05_Binding_API.md
//
// hasASCIIPrefix は同 state package の pubsub.go で定義済の strings.HasPrefix 等価関数。
func validateBindingMetadata(name string, metadata map[string]string) error {
	// SMTP: 宛先メールアドレスと件名は MUST。
	if hasASCIIPrefix(name, bindingNamePrefixSMTP) {
		// emailTo 空は宛先不明で送信不能 → InvalidArgument。
		if metadata[bindingMetaKeySMTPEmailTo] == "" {
			return status.Errorf(codes.InvalidArgument,
				"tier1/binding: SMTP %s metadata required (FR-T1-BINDING-002)", bindingMetaKeySMTPEmailTo)
		}
		// subject 空は SPAM フィルタで弾かれやすく、運用上の事故を構造的に防ぐ。
		if metadata[bindingMetaKeySMTPSubject] == "" {
			return status.Errorf(codes.InvalidArgument,
				"tier1/binding: SMTP %s metadata required (FR-T1-BINDING-002)", bindingMetaKeySMTPSubject)
		}
		return nil
	}
	// HTTP: endpoint path は MUST。host は Component 側で固定 (NFR-E-NW-001 SSRF 防止)。
	if hasASCIIPrefix(name, bindingNamePrefixHTTP) {
		// path 空は endpoint 不明で送信不能。
		if metadata[bindingMetaKeyHTTPPath] == "" {
			return status.Errorf(codes.InvalidArgument,
				"tier1/binding: HTTP %s metadata required (FR-T1-BINDING-003)", bindingMetaKeyHTTPPath)
		}
		return nil
	}
	// 識別不能な name は generic として通過 (MinIO / S3 / Cron / 採用組織のカスタム binding)。
	return nil
}
