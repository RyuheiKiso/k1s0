package grpc

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-config/internal/usecase"
)

// --- Mock: GetConfigExecutor ---

type MockGetConfigUC struct {
	mock.Mock
}

func (m *MockGetConfigUC) Execute(ctx context.Context, input usecase.GetConfigInput) (*usecase.GetConfigOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*usecase.GetConfigOutput), args.Error(1)
}

// --- Mock: ListConfigsExecutor ---

type MockListConfigsUC struct {
	mock.Mock
}

func (m *MockListConfigsUC) Execute(ctx context.Context, input usecase.ListConfigsInput) (*usecase.ListConfigsOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*usecase.ListConfigsOutput), args.Error(1)
}

// --- Mock: GetServiceConfigExecutor ---

type MockGetServiceConfigUC struct {
	mock.Mock
}

func (m *MockGetServiceConfigUC) Execute(ctx context.Context, input usecase.GetServiceConfigInput) (*usecase.GetServiceConfigOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*usecase.GetServiceConfigOutput), args.Error(1)
}

// --- Mock: UpdateConfigExecutor ---

type MockUpdateConfigUC struct {
	mock.Mock
}

func (m *MockUpdateConfigUC) Execute(ctx context.Context, input usecase.UpdateConfigInput) (*usecase.UpdateConfigOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*usecase.UpdateConfigOutput), args.Error(1)
}

// --- Mock: DeleteConfigExecutor ---

type MockDeleteConfigUC struct {
	mock.Mock
}

func (m *MockDeleteConfigUC) Execute(ctx context.Context, input usecase.DeleteConfigInput) error {
	args := m.Called(ctx, input)
	return args.Error(0)
}

// --- TestGetConfig_Success ---

func TestGetConfig_Success(t *testing.T) {
	mockUC := new(MockGetConfigUC)
	svc := &ConfigGRPCService{
		getConfigUC: mockUC,
	}

	expectedOutput := &usecase.GetConfigOutput{
		Namespace:   "system",
		Key:         "database.pool_size",
		Value:       json.RawMessage(`10`),
		Version:     3,
		Description: "Database connection pool size",
		UpdatedBy:   "admin",
		UpdatedAt:   time.Date(2025, 6, 15, 10, 0, 0, 0, time.UTC),
	}

	mockUC.On("Execute", mock.Anything, usecase.GetConfigInput{
		Namespace: "system",
		Key:       "database.pool_size",
	}).Return(expectedOutput, nil)

	req := &GetConfigRequest{Namespace: "system", Key: "database.pool_size"}
	resp, err := svc.GetConfig(context.Background(), req)

	assert.NoError(t, err)
	assert.Equal(t, "system", resp.Entry.Namespace)
	assert.Equal(t, "database.pool_size", resp.Entry.Key)
	assert.Equal(t, json.RawMessage(`10`), resp.Entry.Value)
	assert.Equal(t, int32(3), resp.Entry.Version)
	assert.Equal(t, "Database connection pool size", resp.Entry.Description)
	mockUC.AssertExpectations(t)
}

// --- TestGetConfig_NotFound ---

func TestGetConfig_NotFound(t *testing.T) {
	mockUC := new(MockGetConfigUC)
	svc := &ConfigGRPCService{
		getConfigUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, usecase.GetConfigInput{
		Namespace: "system",
		Key:       "nonexistent",
	}).Return(nil, errors.New("failed to get config entry: record not found"))

	req := &GetConfigRequest{Namespace: "system", Key: "nonexistent"}
	resp, err := svc.GetConfig(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "NotFound")
	mockUC.AssertExpectations(t)
}

// --- TestGetConfig_InvalidArgument ---

func TestGetConfig_InvalidArgument(t *testing.T) {
	svc := &ConfigGRPCService{}

	req := &GetConfigRequest{Namespace: "", Key: ""}
	resp, err := svc.GetConfig(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "InvalidArgument")
}

// --- TestListConfigs_Success ---

func TestListConfigs_Success(t *testing.T) {
	mockUC := new(MockListConfigsUC)
	svc := &ConfigGRPCService{
		listConfigsUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.MatchedBy(func(input usecase.ListConfigsInput) bool {
		return input.Namespace == "system" && input.Page == 1 && input.PageSize == 20
	})).Return(&usecase.ListConfigsOutput{
		Entries: []usecase.ListConfigsEntry{
			{
				Namespace:   "system",
				Key:         "database.pool_size",
				Value:       json.RawMessage(`10`),
				Version:     1,
				Description: "Pool size",
				UpdatedAt:   time.Date(2025, 6, 15, 10, 0, 0, 0, time.UTC),
			},
			{
				Namespace:   "system",
				Key:         "cache.ttl",
				Value:       json.RawMessage(`300`),
				Version:     2,
				Description: "Cache TTL in seconds",
				UpdatedAt:   time.Date(2025, 6, 15, 11, 0, 0, 0, time.UTC),
			},
		},
		TotalCount: 2,
		Page:       1,
		PageSize:   20,
		HasNext:    false,
	}, nil)

	req := &ListConfigsRequest{Namespace: "system"}
	resp, err := svc.ListConfigs(context.Background(), req)

	assert.NoError(t, err)
	assert.Len(t, resp.Entries, 2)
	assert.Equal(t, "database.pool_size", resp.Entries[0].Key)
	assert.Equal(t, "cache.ttl", resp.Entries[1].Key)
	assert.Equal(t, int32(2), resp.Pagination.TotalCount)
	assert.False(t, resp.Pagination.HasNext)
	mockUC.AssertExpectations(t)
}

// --- TestListConfigs_WithPagination ---

func TestListConfigs_WithPagination(t *testing.T) {
	mockUC := new(MockListConfigsUC)
	svc := &ConfigGRPCService{
		listConfigsUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.MatchedBy(func(input usecase.ListConfigsInput) bool {
		return input.Page == 2 && input.PageSize == 10
	})).Return(&usecase.ListConfigsOutput{
		Entries: []usecase.ListConfigsEntry{
			{
				Namespace: "system",
				Key:       "feature.flag_x",
				Value:     json.RawMessage(`true`),
				Version:   1,
			},
		},
		TotalCount: 15,
		Page:       2,
		PageSize:   10,
		HasNext:    false,
	}, nil)

	req := &ListConfigsRequest{
		Namespace:  "system",
		Pagination: &Pagination{Page: 2, PageSize: 10},
	}
	resp, err := svc.ListConfigs(context.Background(), req)

	assert.NoError(t, err)
	assert.Len(t, resp.Entries, 1)
	assert.Equal(t, int32(15), resp.Pagination.TotalCount)
	assert.Equal(t, int32(2), resp.Pagination.Page)
	assert.Equal(t, int32(10), resp.Pagination.PageSize)
	assert.False(t, resp.Pagination.HasNext)
	mockUC.AssertExpectations(t)
}

// --- TestListConfigs_WithSearch ---

func TestListConfigs_WithSearch(t *testing.T) {
	mockUC := new(MockListConfigsUC)
	svc := &ConfigGRPCService{
		listConfigsUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.MatchedBy(func(input usecase.ListConfigsInput) bool {
		return input.Search == "database"
	})).Return(&usecase.ListConfigsOutput{
		Entries: []usecase.ListConfigsEntry{
			{
				Namespace: "system",
				Key:       "database.pool_size",
				Value:     json.RawMessage(`10`),
				Version:   1,
			},
		},
		TotalCount: 1,
		Page:       1,
		PageSize:   20,
		HasNext:    false,
	}, nil)

	req := &ListConfigsRequest{
		Namespace: "system",
		Search:    "database",
	}
	resp, err := svc.ListConfigs(context.Background(), req)

	assert.NoError(t, err)
	assert.Len(t, resp.Entries, 1)
	assert.Equal(t, "database.pool_size", resp.Entries[0].Key)
	mockUC.AssertExpectations(t)
}

// --- TestGetServiceConfig_Success ---

func TestGetServiceConfig_Success(t *testing.T) {
	mockUC := new(MockGetServiceConfigUC)
	svc := &ConfigGRPCService{
		getServiceConfigUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, usecase.GetServiceConfigInput{
		ServiceName: "order-service",
	}).Return(&usecase.GetServiceConfigOutput{
		ServiceName: "order-service",
		Entries: []usecase.ServiceConfigEntry{
			{
				Namespace: "service.order",
				Key:       "max_retry",
				Value:     json.RawMessage(`3`),
			},
			{
				Namespace: "service.order",
				Key:       "timeout_ms",
				Value:     json.RawMessage(`5000`),
			},
		},
	}, nil)

	req := &GetServiceConfigRequest{ServiceName: "order-service"}
	resp, err := svc.GetServiceConfig(context.Background(), req)

	assert.NoError(t, err)
	assert.Equal(t, "order-service", resp.ServiceName)
	assert.Len(t, resp.Entries, 2)
	assert.Equal(t, "max_retry", resp.Entries[0].Key)
	assert.Equal(t, "timeout_ms", resp.Entries[1].Key)
	mockUC.AssertExpectations(t)
}

// --- TestGetServiceConfig_NotFound ---

func TestGetServiceConfig_NotFound(t *testing.T) {
	mockUC := new(MockGetServiceConfigUC)
	svc := &ConfigGRPCService{
		getServiceConfigUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, usecase.GetServiceConfigInput{
		ServiceName: "nonexistent-service",
	}).Return(nil, errors.New("service config not found: nonexistent-service"))

	req := &GetServiceConfigRequest{ServiceName: "nonexistent-service"}
	resp, err := svc.GetServiceConfig(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "NotFound")
	mockUC.AssertExpectations(t)
}

// --- TestUpdateConfig_Success ---

func TestUpdateConfig_Success(t *testing.T) {
	mockUC := new(MockUpdateConfigUC)
	svc := &ConfigGRPCService{
		updateConfigUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.MatchedBy(func(input usecase.UpdateConfigInput) bool {
		return input.Namespace == "system" &&
			input.Key == "database.pool_size" &&
			input.Version == 3 &&
			input.UpdatedBy == "admin"
	})).Return(&usecase.UpdateConfigOutput{
		Namespace:   "system",
		Key:         "database.pool_size",
		Value:       json.RawMessage(`20`),
		Version:     4,
		Description: "Database connection pool size",
		UpdatedBy:   "admin",
		UpdatedAt:   time.Date(2025, 6, 15, 12, 0, 0, 0, time.UTC),
	}, nil)

	req := &UpdateConfigRequest{
		Namespace:   "system",
		Key:         "database.pool_size",
		Value:       json.RawMessage(`20`),
		Version:     3,
		Description: "Database connection pool size",
		UpdatedBy:   "admin",
	}
	resp, err := svc.UpdateConfig(context.Background(), req)

	assert.NoError(t, err)
	assert.Equal(t, "system", resp.Entry.Namespace)
	assert.Equal(t, "database.pool_size", resp.Entry.Key)
	assert.Equal(t, json.RawMessage(`20`), resp.Entry.Value)
	assert.Equal(t, int32(4), resp.Entry.Version)
	mockUC.AssertExpectations(t)
}

// --- TestUpdateConfig_InvalidArgument ---

func TestUpdateConfig_InvalidArgument(t *testing.T) {
	svc := &ConfigGRPCService{}

	req := &UpdateConfigRequest{
		Namespace: "",
		Key:       "",
		Value:     nil,
		UpdatedBy: "",
	}
	resp, err := svc.UpdateConfig(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "InvalidArgument")
}

// --- TestUpdateConfig_VersionConflict ---

func TestUpdateConfig_VersionConflict(t *testing.T) {
	mockUC := new(MockUpdateConfigUC)
	svc := &ConfigGRPCService{
		updateConfigUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.Anything).Return(nil, usecase.ErrVersionConflict)

	req := &UpdateConfigRequest{
		Namespace: "system",
		Key:       "database.pool_size",
		Value:     json.RawMessage(`20`),
		Version:   1,
		UpdatedBy: "admin",
	}
	resp, err := svc.UpdateConfig(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "Aborted")
	mockUC.AssertExpectations(t)
}

// --- TestDeleteConfig_Success ---

func TestDeleteConfig_Success(t *testing.T) {
	mockUC := new(MockDeleteConfigUC)
	svc := &ConfigGRPCService{
		deleteConfigUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, usecase.DeleteConfigInput{
		Namespace: "system",
		Key:       "deprecated.key",
		DeletedBy: "admin",
	}).Return(nil)

	req := &DeleteConfigRequest{
		Namespace: "system",
		Key:       "deprecated.key",
		DeletedBy: "admin",
	}
	resp, err := svc.DeleteConfig(context.Background(), req)

	assert.NoError(t, err)
	assert.True(t, resp.Success)
	mockUC.AssertExpectations(t)
}

// --- TestDeleteConfig_NotFound ---

func TestDeleteConfig_NotFound(t *testing.T) {
	mockUC := new(MockDeleteConfigUC)
	svc := &ConfigGRPCService{
		deleteConfigUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, usecase.DeleteConfigInput{
		Namespace: "system",
		Key:       "nonexistent",
		DeletedBy: "admin",
	}).Return(errors.New("failed to get config entry: record not found"))

	req := &DeleteConfigRequest{
		Namespace: "system",
		Key:       "nonexistent",
		DeletedBy: "admin",
	}
	resp, err := svc.DeleteConfig(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "NotFound")
	mockUC.AssertExpectations(t)
}

// --- TestWatchConfig_Success ---

func TestWatchConfig_Success(t *testing.T) {
	svc := &ConfigGRPCService{}

	req := &WatchConfigRequest{
		Namespace: "system",
		Key:       "database.pool_size",
	}
	resp, err := svc.WatchConfig(context.Background(), req)

	// WatchConfig はストリーミング RPC のため、現時点では Unimplemented を返す
	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Contains(t, err.Error(), "Unimplemented")
}
