// 本ファイルは t1-state Pod の LogService 2 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/07_Log_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-037（Log Adapter: stdout JSON Lines / OTel Collector / Loki 集約）
//
// 役割（plan 04-13 結線済）:
//   SDK 側 facade からの gRPC 入口で proto LogEntry を受け取り、
//   internal/otel.LogEmitter 越しに OTel Logs パイプラインへ流す。
//   LogEmitter は cmd/state/main.go で必ず注入される（OTLP 未設定時は stdout fallback）。

package state

import (
	// context 伝搬。
	"context"

	// 共通 OTel adapter。
	"github.com/k1s0/k1s0/src/tier1/go/internal/otel"
	// SDK 生成 stub の LogService 型。
	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// logHandler は LogService の handler 実装。
type logHandler struct {
	// 将来 RPC 用埋め込み（forward compatibility）。
	logv1.UnimplementedLogServiceServer
	// adapter 集合（LogEmitter を取り出して使う）。
	deps Deps
}

// convertLogEntry は proto LogEntry を otel.LogEntry に詰め替える。
func convertLogEntry(e *logv1.LogEntry) otel.LogEntry {
	if e == nil {
		return otel.LogEntry{}
	}
	var ts int64
	if e.GetTimestamp() != nil {
		ts = e.GetTimestamp().AsTime().UnixNano()
	}
	return otel.LogEntry{
		Timestamp:    ts,
		Severity:     otel.SeverityFromProto(e.GetSeverity()),
		SeverityText: otel.SeverityText(e.GetSeverity()),
		Body:         e.GetBody(),
		Attributes:   e.GetAttributes(),
		StackTrace:   e.GetStackTrace(),
	}
}

// Send は単一エントリ送信。
func (h *logHandler) Send(ctx context.Context, req *logv1.SendLogRequest) (*logv1.SendLogResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/log: nil request")
	}
	// FR-T1-LOG-003 / NFR-E-AC-003: tenant_id 必須強制。
	if _, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Log.Send"); err != nil {
		return nil, err
	}
	if err := h.deps.LogEmitter.Emit(ctx, convertLogEntry(req.GetEntry())); err != nil {
		return nil, status.Errorf(codes.Internal, "tier1/log: emit failed: %v", err)
	}
	return &logv1.SendLogResponse{}, nil
}

// BulkSend は複数エントリの一括送信。
func (h *logHandler) BulkSend(ctx context.Context, req *logv1.BulkSendLogRequest) (*logv1.BulkSendLogResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/log: nil request")
	}
	// FR-T1-LOG-003 / NFR-E-AC-003: tenant_id 必須強制。
	if _, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Log.BulkSend"); err != nil {
		return nil, err
	}
	// 各エントリを順次 emit。1 件失敗で全体失敗とせず、最後にエラー件数を集計する方針も
	// 取れるが、現状は最初のエラーで即返却（gRPC 慣用）。
	accepted := int32(0)
	for _, entry := range req.GetEntries() {
		if err := h.deps.LogEmitter.Emit(ctx, convertLogEntry(entry)); err != nil {
			return nil, status.Errorf(codes.Internal, "tier1/log: emit failed at entry %d: %v", accepted, err)
		}
		accepted++
	}
	return &logv1.BulkSendLogResponse{Accepted: accepted}, nil
}
