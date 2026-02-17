package grpc

import (
	"context"
	"fmt"
	"time"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
)

// RecordAuditLogExecutor は RecordAuditLogUseCase の実行インターフェース。
type RecordAuditLogExecutor interface {
	Execute(ctx context.Context, input usecase.RecordAuditLogInput) (*usecase.RecordAuditLogOutput, error)
}

// SearchAuditLogsExecutor は SearchAuditLogsUseCase の実行インターフェース。
type SearchAuditLogsExecutor interface {
	Execute(ctx context.Context, input usecase.SearchAuditLogsInput) (*usecase.SearchAuditLogsOutput, error)
}

// AuditGRPCService は gRPC AuditService の実装。
type AuditGRPCService struct {
	recordAuditLogUC  RecordAuditLogExecutor
	searchAuditLogsUC SearchAuditLogsExecutor
}

// NewAuditGRPCService は AuditGRPCService のコンストラクタ。
func NewAuditGRPCService(
	recordAuditLogUC RecordAuditLogExecutor,
	searchAuditLogsUC SearchAuditLogsExecutor,
) *AuditGRPCService {
	return &AuditGRPCService{
		recordAuditLogUC:  recordAuditLogUC,
		searchAuditLogsUC: searchAuditLogsUC,
	}
}

// RecordAuditLog は監査ログエントリを記録する。
func (s *AuditGRPCService) RecordAuditLog(ctx context.Context, req *RecordAuditLogRequest) (*RecordAuditLogResponse, error) {
	// バリデーション
	if req.EventType == "" {
		return nil, fmt.Errorf("rpc error: code = InvalidArgument desc = event_type is required")
	}

	input := usecase.RecordAuditLogInput{
		EventType: req.EventType,
		UserID:    req.UserId,
		IPAddress: req.IpAddress,
		UserAgent: req.UserAgent,
		Resource:  req.Resource,
		Action:    req.Action,
		Result:    req.Result,
		Metadata:  req.Metadata,
	}

	output, err := s.recordAuditLogUC.Execute(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	return &RecordAuditLogResponse{
		Id:         output.ID,
		RecordedAt: timeToTimestamp(output.RecordedAt),
	}, nil
}

// SearchAuditLogs は監査ログを検索する。
func (s *AuditGRPCService) SearchAuditLogs(ctx context.Context, req *SearchAuditLogsRequest) (*SearchAuditLogsResponse, error) {
	input := usecase.SearchAuditLogsInput{
		Page:      1,
		PageSize:  50,
		UserID:    req.UserId,
		EventType: req.EventType,
		Result:    req.Result,
	}

	if req.Pagination != nil {
		if req.Pagination.Page > 0 {
			input.Page = int(req.Pagination.Page)
		}
		if req.Pagination.PageSize > 0 {
			input.PageSize = int(req.Pagination.PageSize)
		}
	}

	if req.From != nil {
		t := time.Unix(req.From.Seconds, int64(req.From.Nanos)).UTC()
		input.From = &t
	}
	if req.To != nil {
		t := time.Unix(req.To.Seconds, int64(req.To.Nanos)).UTC()
		input.To = &t
	}

	output, err := s.searchAuditLogsUC.Execute(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	pbLogs := make([]*PbAuditLog, 0, len(output.Logs))
	for _, log := range output.Logs {
		pbLogs = append(pbLogs, domainAuditLogToPb(log))
	}

	return &SearchAuditLogsResponse{
		Logs: pbLogs,
		Pagination: &PaginationResult{
			TotalCount: int32(output.TotalCount),
			Page:       int32(output.Page),
			PageSize:   int32(output.PageSize),
			HasNext:    output.HasNext,
		},
	}, nil
}

// domainAuditLogToPb はドメインの AuditLog を proto 互換型に変換する。
func domainAuditLogToPb(log *model.AuditLog) *PbAuditLog {
	return &PbAuditLog{
		Id:         log.ID,
		EventType:  log.EventType,
		UserId:     log.UserID,
		IpAddress:  log.IPAddress,
		UserAgent:  log.UserAgent,
		Resource:   log.Resource,
		Action:     log.Action,
		Result:     log.Result,
		Metadata:   log.Metadata,
		RecordedAt: timeToTimestamp(log.RecordedAt),
	}
}
