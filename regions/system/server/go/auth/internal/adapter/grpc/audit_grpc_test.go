package grpc

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
)

// --- Mock: RecordAuditLogExecutor ---

type MockRecordAuditLogUC struct {
	mock.Mock
}

func (m *MockRecordAuditLogUC) Execute(ctx context.Context, input usecase.RecordAuditLogInput) (*usecase.RecordAuditLogOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*usecase.RecordAuditLogOutput), args.Error(1)
}

// --- Mock: SearchAuditLogsExecutor ---

type MockSearchAuditLogsUC struct {
	mock.Mock
}

func (m *MockSearchAuditLogsUC) Execute(ctx context.Context, input usecase.SearchAuditLogsInput) (*usecase.SearchAuditLogsOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*usecase.SearchAuditLogsOutput), args.Error(1)
}

// --- TestRecordAuditLog_Success ---

func TestRecordAuditLog_Success(t *testing.T) {
	mockUC := new(MockRecordAuditLogUC)
	svc := &AuditGRPCService{
		recordAuditLogUC: mockUC,
	}

	now := time.Now().UTC()
	mockUC.On("Execute", mock.Anything, mock.MatchedBy(func(input usecase.RecordAuditLogInput) bool {
		return input.EventType == "LOGIN_SUCCESS" && input.UserID == "user-uuid-1234"
	})).Return(&usecase.RecordAuditLogOutput{
		ID:         "audit-uuid-1",
		RecordedAt: now,
	}, nil)

	req := &RecordAuditLogRequest{
		EventType: "LOGIN_SUCCESS",
		UserId:    "user-uuid-1234",
		IpAddress: "192.168.1.100",
		UserAgent: "Mozilla/5.0",
		Resource:  "/api/v1/auth/token",
		Action:    "POST",
		Result:    "SUCCESS",
		Metadata:  map[string]string{"client_id": "react-spa"},
	}

	resp, err := svc.RecordAuditLog(context.Background(), req)

	assert.NoError(t, err)
	assert.Equal(t, "audit-uuid-1", resp.Id)
	assert.NotNil(t, resp.RecordedAt)
	assert.Equal(t, now.Unix(), resp.RecordedAt.Seconds)
	mockUC.AssertExpectations(t)
}

// --- TestRecordAuditLog_EmptyEventType ---

func TestRecordAuditLog_EmptyEventType(t *testing.T) {
	mockUC := new(MockRecordAuditLogUC)
	svc := &AuditGRPCService{
		recordAuditLogUC: mockUC,
	}

	req := &RecordAuditLogRequest{
		EventType: "",
		UserId:    "user-uuid-1234",
		Result:    "SUCCESS",
	}

	resp, err := svc.RecordAuditLog(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "event_type is required")
}

// --- TestSearchAuditLogs_WithFilters ---

func TestSearchAuditLogs_WithFilters(t *testing.T) {
	mockUC := new(MockSearchAuditLogsUC)
	svc := &AuditGRPCService{
		searchAuditLogsUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.MatchedBy(func(input usecase.SearchAuditLogsInput) bool {
		return input.UserID == "user-1" && input.EventType == "LOGIN_SUCCESS" && input.Page == 1 && input.PageSize == 50
	})).Return(&usecase.SearchAuditLogsOutput{
		Logs:       nil,
		TotalCount: 5,
		Page:       1,
		PageSize:   50,
		HasNext:    false,
	}, nil)

	req := &SearchAuditLogsRequest{
		Pagination: &Pagination{Page: 1, PageSize: 50},
		UserId:     "user-1",
		EventType:  "LOGIN_SUCCESS",
	}

	resp, err := svc.SearchAuditLogs(context.Background(), req)

	assert.NoError(t, err)
	assert.NotNil(t, resp.Pagination)
	assert.Equal(t, int32(5), resp.Pagination.TotalCount)
	assert.Equal(t, int32(1), resp.Pagination.Page)
	assert.False(t, resp.Pagination.HasNext)
	mockUC.AssertExpectations(t)
}

// --- TestSearchAuditLogs_Empty ---

func TestSearchAuditLogs_Empty(t *testing.T) {
	mockUC := new(MockSearchAuditLogsUC)
	svc := &AuditGRPCService{
		searchAuditLogsUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.Anything).Return(&usecase.SearchAuditLogsOutput{
		Logs:       nil,
		TotalCount: 0,
		Page:       1,
		PageSize:   50,
		HasNext:    false,
	}, nil)

	req := &SearchAuditLogsRequest{
		Pagination: &Pagination{Page: 1, PageSize: 50},
	}

	resp, err := svc.SearchAuditLogs(context.Background(), req)

	assert.NoError(t, err)
	assert.Empty(t, resp.Logs)
	assert.Equal(t, int32(0), resp.Pagination.TotalCount)
	assert.False(t, resp.Pagination.HasNext)
	mockUC.AssertExpectations(t)
}

// --- TestSearchAuditLogs_RepositoryError ---

func TestSearchAuditLogs_RepositoryError(t *testing.T) {
	mockUC := new(MockSearchAuditLogsUC)
	svc := &AuditGRPCService{
		searchAuditLogsUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.Anything).Return(nil, errors.New("database connection failed"))

	req := &SearchAuditLogsRequest{
		Pagination: &Pagination{Page: 1, PageSize: 50},
	}

	resp, err := svc.SearchAuditLogs(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "database connection failed")
}
