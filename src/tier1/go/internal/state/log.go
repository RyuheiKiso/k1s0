// 本ファイルは t1-state Pod の LogService 2 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/07_Log_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-037（Log Adapter: stdout JSON Lines / OTel Collector / Loki 集約）
//   src/tier1/README.md（t1-state Pod の責務に Log を含む）
//
// scope（リリース時点 placeholder）:
//   実 OTel Logs パイプライン（OTel Collector → Loki）への結線は plan 04-13 同期。
//   現状は SDK 接続点を提供するため skeleton として登録し、全 RPC は Unimplemented を返す。
//   FR-T1-LOG-001〜004 のうち SDK 側 facade（src/sdk/*/log）は同梱済、本 handler は
//   それを受け止める空 server として機能する。

package state

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// SDK 生成 stub の LogService 型。
	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// logHandler は LogService の handler 実装。
// LogService は OTel Logs パイプライン（OTel Collector → Loki）に直接乗せる設計のため、
// 本 handler は SDK 側 facade からの gRPC 入口を確保するだけで、本格実装は plan 04-13 で
// OTel SDK / Collector 連携の文脈で行う。
type logHandler struct {
	// 将来 RPC 用埋め込み（forward compatibility）。
	logv1.UnimplementedLogServiceServer
	// adapter 集合（Log は OTel 側へ流すため Dapr adapter は使わない）。
	deps Deps
}

// Send は単一エントリ送信。
func (h *logHandler) Send(_ context.Context, req *logv1.SendLogRequest) (*logv1.SendLogResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/log: nil request")
	}
	// 実 OTel Logs パイプライン結線は plan 04-13。
	return nil, status.Error(codes.Unimplemented, "tier1/log: Send not yet wired to OTel Collector (plan 04-13)")
}

// BulkSend は複数エントリの一括送信。
func (h *logHandler) BulkSend(_ context.Context, req *logv1.BulkSendLogRequest) (*logv1.BulkSendLogResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/log: nil request")
	}
	// 実 OTel Logs パイプライン結線は plan 04-13。
	return nil, status.Error(codes.Unimplemented, "tier1/log: BulkSend not yet wired to OTel Collector (plan 04-13)")
}
