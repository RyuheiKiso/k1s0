package dlq_test

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	dlq "github.com/k1s0-platform/system-library-go-dlq-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// NewDlqClientがベースURLの末尾スラッシュを除去してリクエストを正常に送信することを確認する。
func TestNewDlqClient_TrimsTrailingSlash(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/api/v1/dlq/test-topic", r.URL.Path)
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(dlq.ListDlqMessagesResponse{
			Messages: []dlq.DlqMessage{},
			Total:    0,
			Page:     1,
		})
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL+"/", server.Client())
	_, err := client.ListMessages(context.Background(), &dlq.ListDlqMessagesRequest{
		Topic:    "test-topic",
		Page:     1,
		PageSize: 10,
	})
	require.NoError(t, err)
}

// ListMessagesが指定トピックのDLQメッセージ一覧を正常に取得することを確認する。
func TestListMessages_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodGet, r.Method)
		assert.Equal(t, "/api/v1/dlq/order-events", r.URL.Path)
		assert.Equal(t, "1", r.URL.Query().Get("page"))
		assert.Equal(t, "10", r.URL.Query().Get("page_size"))

		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(dlq.ListDlqMessagesResponse{
			Messages: []dlq.DlqMessage{
				{
					ID:            "msg-1",
					OriginalTopic: "order-events",
					ErrorMessage:  "processing failed",
					RetryCount:    2,
					MaxRetries:    5,
					Status:        dlq.DlqStatusPending,
				},
			},
			Total: 1,
			Page:  1,
		})
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	resp, err := client.ListMessages(context.Background(), &dlq.ListDlqMessagesRequest{
		Topic:    "order-events",
		Page:     1,
		PageSize: 10,
	})

	require.NoError(t, err)
	assert.Equal(t, 1, resp.Total)
	assert.Len(t, resp.Messages, 1)
	assert.Equal(t, "msg-1", resp.Messages[0].ID)
	assert.Equal(t, dlq.DlqStatusPending, resp.Messages[0].Status)
}

// ListMessagesがサーバーエラー時にDlqErrorを返すことを確認する。
func TestListMessages_ErrorStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		w.Write([]byte("internal server error"))
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	_, err := client.ListMessages(context.Background(), &dlq.ListDlqMessagesRequest{
		Topic:    "test",
		Page:     1,
		PageSize: 10,
	})

	require.Error(t, err)
	var dlqErr *dlq.DlqError
	require.True(t, errors.As(err, &dlqErr))
	assert.Equal(t, "list_messages", dlqErr.Op)
	assert.Equal(t, http.StatusInternalServerError, dlqErr.StatusCode)
}

// GetMessageが指定IDのDLQメッセージを正常に取得することを確認する。
func TestGetMessage_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodGet, r.Method)
		assert.Equal(t, "/api/v1/dlq/messages/msg-123", r.URL.Path)

		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(dlq.DlqMessage{
			ID:            "msg-123",
			OriginalTopic: "order-events",
			ErrorMessage:  "timeout",
			RetryCount:    1,
			MaxRetries:    3,
			Status:        dlq.DlqStatusRetrying,
		})
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	msg, err := client.GetMessage(context.Background(), "msg-123")

	require.NoError(t, err)
	assert.Equal(t, "msg-123", msg.ID)
	assert.Equal(t, "order-events", msg.OriginalTopic)
	assert.Equal(t, dlq.DlqStatusRetrying, msg.Status)
}

// GetMessageがメッセージ未存在時にDlqErrorを返すことを確認する。
func TestGetMessage_ErrorStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		w.Write([]byte("not found"))
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	_, err := client.GetMessage(context.Background(), "nonexistent")

	require.Error(t, err)
	var dlqErr *dlq.DlqError
	require.True(t, errors.As(err, &dlqErr))
	assert.Equal(t, "get_message", dlqErr.Op)
	assert.Equal(t, http.StatusNotFound, dlqErr.StatusCode)
}

// RetryMessageが指定IDのDLQメッセージのリトライを正常にリクエストすることを確認する。
func TestRetryMessage_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "/api/v1/dlq/messages/msg-456/retry", r.URL.Path)
		assert.Equal(t, "application/json", r.Header.Get("Content-Type"))

		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(dlq.RetryDlqMessageResponse{
			MessageID: "msg-456",
			Status:    dlq.DlqStatusRetrying,
		})
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	resp, err := client.RetryMessage(context.Background(), "msg-456")

	require.NoError(t, err)
	assert.Equal(t, "msg-456", resp.MessageID)
	assert.Equal(t, dlq.DlqStatusRetrying, resp.Status)
}

// RetryMessageがサーバーエラー時にDlqErrorを返すことを確認する。
func TestRetryMessage_ErrorStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusConflict)
		w.Write([]byte("message already resolved"))
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	_, err := client.RetryMessage(context.Background(), "msg-resolved")

	require.Error(t, err)
	var dlqErr *dlq.DlqError
	require.True(t, errors.As(err, &dlqErr))
	assert.Equal(t, "retry_message", dlqErr.Op)
	assert.Equal(t, http.StatusConflict, dlqErr.StatusCode)
}

// DeleteMessageが指定IDのDLQメッセージを正常に削除することを確認する。
func TestDeleteMessage_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodDelete, r.Method)
		assert.Equal(t, "/api/v1/dlq/messages/msg-789", r.URL.Path)
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	err := client.DeleteMessage(context.Background(), "msg-789")

	require.NoError(t, err)
}

// DeleteMessageがメッセージ未存在時にDlqErrorを返すことを確認する。
func TestDeleteMessage_ErrorStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		w.Write([]byte("not found"))
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	err := client.DeleteMessage(context.Background(), "nonexistent")

	require.Error(t, err)
	var dlqErr *dlq.DlqError
	require.True(t, errors.As(err, &dlqErr))
	assert.Equal(t, "delete_message", dlqErr.Op)
	assert.Equal(t, http.StatusNotFound, dlqErr.StatusCode)
}

// RetryAllが指定トピックの全DLQメッセージのリトライを正常にリクエストすることを確認する。
func TestRetryAll_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "/api/v1/dlq/order-events/retry-all", r.URL.Path)
		assert.Equal(t, "application/json", r.Header.Get("Content-Type"))
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	err := client.RetryAll(context.Background(), "order-events")

	require.NoError(t, err)
}

// RetryAllがサーバーエラー時にDlqErrorを返すことを確認する。
func TestRetryAll_ErrorStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		w.Write([]byte("retry failed"))
	}))
	defer server.Close()

	client := dlq.NewDlqClientWithHTTPClient(server.URL, server.Client())
	err := client.RetryAll(context.Background(), "order-events")

	require.Error(t, err)
	var dlqErr *dlq.DlqError
	require.True(t, errors.As(err, &dlqErr))
	assert.Equal(t, "retry_all", dlqErr.Op)
	assert.Equal(t, http.StatusInternalServerError, dlqErr.StatusCode)
}

// DlqStatusの定数値が期待する文字列と一致することを確認する。
func TestDlqStatus_Constants(t *testing.T) {
	assert.Equal(t, dlq.DlqStatus("PENDING"), dlq.DlqStatusPending)
	assert.Equal(t, dlq.DlqStatus("RETRYING"), dlq.DlqStatusRetrying)
	assert.Equal(t, dlq.DlqStatus("RESOLVED"), dlq.DlqStatusResolved)
	assert.Equal(t, dlq.DlqStatus("DEAD"), dlq.DlqStatusDead)
}

// DlqErrorのErrorメソッドがステータスコードを含む正しいエラーメッセージを返すことを確認する。
func TestDlqError_ErrorMessage(t *testing.T) {
	err := &dlq.DlqError{
		Op:         "list_messages",
		StatusCode: 500,
		Err:        errors.New("internal error"),
	}
	assert.Equal(t, "dlq list_messages: status 500: internal error", err.Error())
}

// DlqErrorのErrorメソッドがステータスコード未設定時にシンプルなエラーメッセージを返すことを確認する。
func TestDlqError_ErrorMessageWithoutStatusCode(t *testing.T) {
	err := &dlq.DlqError{
		Op:  "get_message",
		Err: errors.New("connection refused"),
	}
	assert.Equal(t, "dlq get_message: connection refused", err.Error())
}

// DlqErrorのUnwrapメソッドが内包するエラーを正しく返すことを確認する。
func TestDlqError_Unwrap(t *testing.T) {
	inner := errors.New("original error")
	err := &dlq.DlqError{
		Op:  "retry_message",
		Err: inner,
	}
	assert.Equal(t, inner, errors.Unwrap(err))
}
