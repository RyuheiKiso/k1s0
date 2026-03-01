package webhookclient_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"

	webhookclient "github.com/k1s0-platform/system-library-go-webhook-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGenerateSignature_Deterministic(t *testing.T) {
	secret := "my-secret"
	body := []byte(`{"event_type":"test"}`)
	sig1 := webhookclient.GenerateSignature(secret, body)
	sig2 := webhookclient.GenerateSignature(secret, body)
	assert.Equal(t, sig1, sig2)
	assert.NotEmpty(t, sig1)
}

func TestVerifySignature_Valid(t *testing.T) {
	secret := "my-secret"
	body := []byte(`{"event_type":"test"}`)
	sig := webhookclient.GenerateSignature(secret, body)
	assert.True(t, webhookclient.VerifySignature(secret, body, sig))
}

func TestVerifySignature_WrongSecret(t *testing.T) {
	body := []byte(`{"event_type":"test"}`)
	sig := webhookclient.GenerateSignature("secret1", body)
	assert.False(t, webhookclient.VerifySignature("secret2", body, sig))
}

func TestVerifySignature_TamperedBody(t *testing.T) {
	secret := "my-secret"
	body := []byte(`{"event_type":"test"}`)
	sig := webhookclient.GenerateSignature(secret, body)
	tampered := []byte(`{"event_type":"hacked"}`)
	assert.False(t, webhookclient.VerifySignature(secret, tampered, sig))
}

func TestVerifySignature_EmptyBody(t *testing.T) {
	secret := "my-secret"
	body := []byte{}
	sig := webhookclient.GenerateSignature(secret, body)
	assert.True(t, webhookclient.VerifySignature(secret, body, sig))
}

func TestGenerateSignature_DifferentBodies(t *testing.T) {
	secret := "my-secret"
	sig1 := webhookclient.GenerateSignature(secret, []byte("body1"))
	sig2 := webhookclient.GenerateSignature(secret, []byte("body2"))
	assert.NotEqual(t, sig1, sig2)
}

func newTestPayload() *webhookclient.WebhookPayload {
	return &webhookclient.WebhookPayload{
		EventType: "test.event",
		Timestamp: "2026-01-01T00:00:00Z",
		Data:      map[string]any{"key": "value"},
	}
}

func TestSend_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "application/json", r.Header.Get("Content-Type"))
		assert.NotEmpty(t, r.Header.Get(webhookclient.SignatureHeader))
		assert.NotEmpty(t, r.Header.Get(webhookclient.IdempotencyKeyHeader))
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := webhookclient.NewHTTPWebhookClient("test-secret")
	statusCode, err := client.Send(context.Background(), server.URL, newTestPayload())
	require.NoError(t, err)
	assert.Equal(t, http.StatusOK, statusCode)
}

func TestSend_SignatureHeader_IsK1s0(t *testing.T) {
	var receivedHeader string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		receivedHeader = r.Header.Get(webhookclient.SignatureHeader)
		// X-Webhook-Signature は送信されないことを確認
		assert.Empty(t, r.Header.Get("X-Webhook-Signature"))
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := webhookclient.NewHTTPWebhookClient("test-secret")
	_, err := client.Send(context.Background(), server.URL, newTestPayload())
	require.NoError(t, err)
	assert.NotEmpty(t, receivedHeader)
}

func TestSend_IdempotencyKey_IsSentAsUUID(t *testing.T) {
	var receivedKey string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		receivedKey = r.Header.Get(webhookclient.IdempotencyKeyHeader)
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := webhookclient.NewHTTPWebhookClient("test-secret")
	_, err := client.Send(context.Background(), server.URL, newTestPayload())
	require.NoError(t, err)
	assert.NotEmpty(t, receivedKey)
	// UUID v4 フォーマット: 8-4-4-4-12
	assert.Len(t, receivedKey, 36)
	assert.Equal(t, byte('-'), receivedKey[8])
	assert.Equal(t, byte('-'), receivedKey[13])
	assert.Equal(t, byte('-'), receivedKey[18])
	assert.Equal(t, byte('-'), receivedKey[23])
}

func TestSend_IdempotencyKey_SameAcrossRetries(t *testing.T) {
	var keys []string
	var callCount int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		keys = append(keys, r.Header.Get(webhookclient.IdempotencyKeyHeader))
		count := atomic.AddInt32(&callCount, 1)
		if count <= 2 {
			w.WriteHeader(http.StatusInternalServerError)
			return
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	// スリープをスキップしてテストを高速化
	origSleep := webhookclient.ExportSleepFunc()
	webhookclient.SetSleepFunc(func(d time.Duration) {})
	defer webhookclient.SetSleepFunc(origSleep)

	config := webhookclient.WebhookConfig{
		MaxRetries:       3,
		InitialBackoffMs: 10,
		MaxBackoffMs:     100,
	}
	client := webhookclient.NewHTTPWebhookClientWithConfig("test-secret", config)
	statusCode, err := client.Send(context.Background(), server.URL, newTestPayload())
	require.NoError(t, err)
	assert.Equal(t, http.StatusOK, statusCode)
	assert.Equal(t, 3, len(keys))
	// 全リトライで同一のIdempotency-Key
	assert.Equal(t, keys[0], keys[1])
	assert.Equal(t, keys[1], keys[2])
}

func TestSend_Retry_On5xx(t *testing.T) {
	var callCount int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		count := atomic.AddInt32(&callCount, 1)
		if count <= 2 {
			w.WriteHeader(http.StatusServiceUnavailable) // 503
			return
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	origSleep := webhookclient.ExportSleepFunc()
	webhookclient.SetSleepFunc(func(d time.Duration) {})
	defer webhookclient.SetSleepFunc(origSleep)

	config := webhookclient.WebhookConfig{
		MaxRetries:       3,
		InitialBackoffMs: 10,
		MaxBackoffMs:     100,
	}
	client := webhookclient.NewHTTPWebhookClientWithConfig("test-secret", config)
	statusCode, err := client.Send(context.Background(), server.URL, newTestPayload())
	require.NoError(t, err)
	assert.Equal(t, http.StatusOK, statusCode)
	assert.Equal(t, int32(3), atomic.LoadInt32(&callCount))
}

func TestSend_Retry_On429(t *testing.T) {
	var callCount int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		count := atomic.AddInt32(&callCount, 1)
		if count <= 1 {
			w.WriteHeader(http.StatusTooManyRequests) // 429
			return
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	origSleep := webhookclient.ExportSleepFunc()
	webhookclient.SetSleepFunc(func(d time.Duration) {})
	defer webhookclient.SetSleepFunc(origSleep)

	config := webhookclient.WebhookConfig{
		MaxRetries:       2,
		InitialBackoffMs: 10,
		MaxBackoffMs:     100,
	}
	client := webhookclient.NewHTTPWebhookClientWithConfig("test-secret", config)
	statusCode, err := client.Send(context.Background(), server.URL, newTestPayload())
	require.NoError(t, err)
	assert.Equal(t, http.StatusOK, statusCode)
	assert.Equal(t, int32(2), atomic.LoadInt32(&callCount))
}

func TestSend_NoRetry_On4xx(t *testing.T) {
	var callCount int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		atomic.AddInt32(&callCount, 1)
		w.WriteHeader(http.StatusBadRequest) // 400
	}))
	defer server.Close()

	config := webhookclient.WebhookConfig{
		MaxRetries:       3,
		InitialBackoffMs: 10,
		MaxBackoffMs:     100,
	}
	client := webhookclient.NewHTTPWebhookClientWithConfig("test-secret", config)
	statusCode, err := client.Send(context.Background(), server.URL, newTestPayload())
	require.NoError(t, err)
	assert.Equal(t, http.StatusBadRequest, statusCode)
	assert.Equal(t, int32(1), atomic.LoadInt32(&callCount)) // リトライなし
}

func TestSend_MaxRetriesExceeded(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError) // 常に500
	}))
	defer server.Close()

	origSleep := webhookclient.ExportSleepFunc()
	webhookclient.SetSleepFunc(func(d time.Duration) {})
	defer webhookclient.SetSleepFunc(origSleep)

	config := webhookclient.WebhookConfig{
		MaxRetries:       2,
		InitialBackoffMs: 10,
		MaxBackoffMs:     100,
	}
	client := webhookclient.NewHTTPWebhookClientWithConfig("test-secret", config)
	statusCode, err := client.Send(context.Background(), server.URL, newTestPayload())
	require.Error(t, err)
	assert.Equal(t, http.StatusInternalServerError, statusCode)

	var maxRetriesErr *webhookclient.MaxRetriesExceededError
	assert.True(t, errors.As(err, &maxRetriesErr))
	assert.Equal(t, 3, maxRetriesErr.Attempts)
	assert.Equal(t, http.StatusInternalServerError, maxRetriesErr.LastStatusCode)
}

func TestSend_DefaultConfig(t *testing.T) {
	config := webhookclient.DefaultWebhookConfig()
	assert.Equal(t, 3, config.MaxRetries)
	assert.Equal(t, 100, config.InitialBackoffMs)
	assert.Equal(t, 10000, config.MaxBackoffMs)
}
