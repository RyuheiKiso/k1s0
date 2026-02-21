package saga_test

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	saga "github.com/k1s0-platform/system-library-go-saga"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewSagaClient_TrimsTrailingSlash(t *testing.T) {
	client := saga.NewSagaClient("http://localhost:8080/")
	// endpoint は非公開フィールドなので、StartSaga のリクエスト先URLで検証する
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/api/v1/sagas", r.URL.Path)
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(map[string]string{"saga_id": "test-id"})
	}))
	defer server.Close()

	client = saga.NewSagaClientWithHTTPClient(server.URL+"/", server.Client())
	_, err := client.StartSaga(context.Background(), &saga.StartSagaRequest{
		WorkflowName: "test",
		Payload:      map[string]string{"key": "value"},
	})
	require.NoError(t, err)
}

func TestStartSaga_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "/api/v1/sagas", r.URL.Path)
		assert.Equal(t, "application/json", r.Header.Get("Content-Type"))

		var req saga.StartSagaRequest
		err := json.NewDecoder(r.Body).Decode(&req)
		require.NoError(t, err)
		assert.Equal(t, "order-create", req.WorkflowName)

		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(saga.StartSagaResponse{SagaID: "saga-123"})
	}))
	defer server.Close()

	client := saga.NewSagaClientWithHTTPClient(server.URL, server.Client())
	resp, err := client.StartSaga(context.Background(), &saga.StartSagaRequest{
		WorkflowName: "order-create",
		Payload:      map[string]string{"order_id": "ord-1"},
	})

	require.NoError(t, err)
	assert.Equal(t, "saga-123", resp.SagaID)
}

func TestStartSaga_ErrorStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		w.Write([]byte("internal server error"))
	}))
	defer server.Close()

	client := saga.NewSagaClientWithHTTPClient(server.URL, server.Client())
	_, err := client.StartSaga(context.Background(), &saga.StartSagaRequest{
		WorkflowName: "test",
	})

	require.Error(t, err)
	var sagaErr *saga.SagaError
	require.True(t, errors.As(err, &sagaErr))
	assert.Equal(t, "start_saga", sagaErr.Op)
	assert.Equal(t, http.StatusInternalServerError, sagaErr.StatusCode)
}

func TestGetSaga_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodGet, r.Method)
		assert.Equal(t, "/api/v1/sagas/saga-456", r.URL.Path)

		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(map[string]any{
			"saga": map[string]any{
				"saga_id":       "saga-456",
				"workflow_name": "order-create",
				"status":     "RUNNING",
				"step_logs":  []any{},
				"created_at": "2024-01-01T00:00:00Z",
				"updated_at": "2024-01-01T00:00:00Z",
			},
		})
	}))
	defer server.Close()

	client := saga.NewSagaClientWithHTTPClient(server.URL, server.Client())
	state, err := client.GetSaga(context.Background(), "saga-456")

	require.NoError(t, err)
	assert.Equal(t, "saga-456", state.SagaID)
	assert.Equal(t, "order-create", state.WorkflowName)
	assert.Equal(t, saga.SagaStatusRunning, state.Status)
}

func TestGetSaga_ErrorStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		w.Write([]byte("not found"))
	}))
	defer server.Close()

	client := saga.NewSagaClientWithHTTPClient(server.URL, server.Client())
	_, err := client.GetSaga(context.Background(), "nonexistent")

	require.Error(t, err)
	var sagaErr *saga.SagaError
	require.True(t, errors.As(err, &sagaErr))
	assert.Equal(t, "get_saga", sagaErr.Op)
	assert.Equal(t, http.StatusNotFound, sagaErr.StatusCode)
}

func TestCancelSaga_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "/api/v1/sagas/saga-789/cancel", r.URL.Path)
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := saga.NewSagaClientWithHTTPClient(server.URL, server.Client())
	err := client.CancelSaga(context.Background(), "saga-789")

	require.NoError(t, err)
}

func TestCancelSaga_ErrorStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusConflict)
		w.Write([]byte("saga already completed"))
	}))
	defer server.Close()

	client := saga.NewSagaClientWithHTTPClient(server.URL, server.Client())
	err := client.CancelSaga(context.Background(), "saga-completed")

	require.Error(t, err)
	var sagaErr *saga.SagaError
	require.True(t, errors.As(err, &sagaErr))
	assert.Equal(t, "cancel_saga", sagaErr.Op)
	assert.Equal(t, http.StatusConflict, sagaErr.StatusCode)
}

func TestSagaStatus_Constants(t *testing.T) {
	assert.Equal(t, saga.SagaStatus("STARTED"), saga.SagaStatusStarted)
	assert.Equal(t, saga.SagaStatus("RUNNING"), saga.SagaStatusRunning)
	assert.Equal(t, saga.SagaStatus("COMPLETED"), saga.SagaStatusCompleted)
	assert.Equal(t, saga.SagaStatus("COMPENSATING"), saga.SagaStatusCompensating)
	assert.Equal(t, saga.SagaStatus("FAILED"), saga.SagaStatusFailed)
	assert.Equal(t, saga.SagaStatus("CANCELLED"), saga.SagaStatusCancelled)
}

func TestSagaError_ErrorMessage(t *testing.T) {
	err := &saga.SagaError{
		Op:         "start_saga",
		StatusCode: 500,
		Err:        errors.New("internal error"),
	}
	assert.Equal(t, "saga start_saga: status 500: internal error", err.Error())
}

func TestSagaError_ErrorMessageWithoutStatusCode(t *testing.T) {
	err := &saga.SagaError{
		Op:  "start_saga",
		Err: errors.New("connection refused"),
	}
	assert.Equal(t, "saga start_saga: connection refused", err.Error())
}

func TestSagaError_Unwrap(t *testing.T) {
	inner := errors.New("original error")
	err := &saga.SagaError{
		Op:  "get_saga",
		Err: inner,
	}
	assert.Equal(t, inner, errors.Unwrap(err))
}

func TestStartSaga_WithPayload(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var req saga.StartSagaRequest
		require.NoError(t, json.NewDecoder(r.Body).Decode(&req))
		assert.Equal(t, "order-create", req.WorkflowName)
		// payload が送信されていることを確認
		assert.NotNil(t, req.Payload)
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(saga.StartSagaResponse{SagaID: "saga-with-payload"})
	}))
	defer server.Close()

	client := saga.NewSagaClientWithHTTPClient(server.URL, server.Client())
	resp, err := client.StartSaga(context.Background(), &saga.StartSagaRequest{
		WorkflowName: "order-create",
		Payload:      map[string]string{"order_id": "ord-999"},
	})
	require.NoError(t, err)
	assert.Equal(t, "saga-with-payload", resp.SagaID)
}
