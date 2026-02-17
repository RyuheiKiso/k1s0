package auth

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// --- Device Flow テスト ---

func TestDeviceAuthClient_RequestDeviceCode_Success(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "POST", r.Method)
		assert.Equal(t, "application/x-www-form-urlencoded", r.Header.Get("Content-Type"))
		require.NoError(t, r.ParseForm())
		assert.Equal(t, "test-client", r.FormValue("client_id"))
		assert.Equal(t, "openid profile", r.FormValue("scope"))

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(DeviceCodeResponse{
			DeviceCode:              "device-code-123",
			UserCode:                "ABCD-EFGH",
			VerificationURI:         "https://auth.example.com/device",
			VerificationURIComplete: "https://auth.example.com/device?user_code=ABCD-EFGH",
			ExpiresIn:               600,
			Interval:                5,
		})
	}))
	defer server.Close()

	client := NewDeviceAuthClient(server.URL, server.URL)

	resp, err := client.RequestDeviceCode(context.Background(), "test-client", "openid profile")
	require.NoError(t, err)
	assert.Equal(t, "device-code-123", resp.DeviceCode)
	assert.Equal(t, "ABCD-EFGH", resp.UserCode)
	assert.Equal(t, "https://auth.example.com/device", resp.VerificationURI)
	assert.Equal(t, "https://auth.example.com/device?user_code=ABCD-EFGH", resp.VerificationURIComplete)
	assert.Equal(t, 600, resp.ExpiresIn)
	assert.Equal(t, 5, resp.Interval)
}

func TestDeviceAuthClient_PollToken_AuthorizationPendingThenSuccess(t *testing.T) {
	var callCount int32

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		require.NoError(t, r.ParseForm())
		assert.Equal(t, "urn:ietf:params:oauth:grant-type:device_code", r.FormValue("grant_type"))
		assert.Equal(t, "device-code-123", r.FormValue("device_code"))
		assert.Equal(t, "test-client", r.FormValue("client_id"))

		count := atomic.AddInt32(&callCount, 1)
		if count <= 2 {
			w.WriteHeader(http.StatusBadRequest)
			json.NewEncoder(w).Encode(map[string]string{
				"error": "authorization_pending",
			})
			return
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(TokenResult{
			AccessToken:  "access-token-xyz",
			RefreshToken: "refresh-token-xyz",
			TokenType:    "Bearer",
			ExpiresIn:    900,
		})
	}))
	defer server.Close()

	client := NewDeviceAuthClient(server.URL, server.URL)

	result, err := client.PollToken(context.Background(), "test-client", "device-code-123", 0)
	require.NoError(t, err)
	assert.Equal(t, "access-token-xyz", result.AccessToken)
	assert.Equal(t, "refresh-token-xyz", result.RefreshToken)
	assert.Equal(t, "Bearer", result.TokenType)
	assert.Equal(t, 900, result.ExpiresIn)
	assert.GreaterOrEqual(t, atomic.LoadInt32(&callCount), int32(3))
}

func TestDeviceAuthClient_PollToken_SlowDown(t *testing.T) {
	var callCount int32

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		count := atomic.AddInt32(&callCount, 1)
		if count == 1 {
			w.WriteHeader(http.StatusBadRequest)
			json.NewEncoder(w).Encode(map[string]string{
				"error": "slow_down",
			})
			return
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(TokenResult{
			AccessToken: "access-token",
			TokenType:   "Bearer",
			ExpiresIn:   900,
		})
	}))
	defer server.Close()

	client := NewDeviceAuthClient(server.URL, server.URL)

	start := time.Now()
	result, err := client.PollToken(context.Background(), "test-client", "device-code-123", 0)
	elapsed := time.Since(start)

	require.NoError(t, err)
	assert.Equal(t, "access-token", result.AccessToken)
	// slow_down を受け取った後、interval が増加しているはず（少なくとも 5 秒以上待つ）
	assert.GreaterOrEqual(t, elapsed, 5*time.Second)
}

func TestDeviceAuthClient_PollToken_ExpiredToken(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(map[string]string{
			"error": "expired_token",
		})
	}))
	defer server.Close()

	client := NewDeviceAuthClient(server.URL, server.URL)

	_, err := client.PollToken(context.Background(), "test-client", "device-code-123", 0)
	require.Error(t, err)
	var dfe *DeviceFlowError
	require.ErrorAs(t, err, &dfe)
	assert.Equal(t, "expired_token", dfe.ErrorCode)
}

func TestDeviceAuthClient_PollToken_AccessDenied(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(map[string]string{
			"error": "access_denied",
		})
	}))
	defer server.Close()

	client := NewDeviceAuthClient(server.URL, server.URL)

	_, err := client.PollToken(context.Background(), "test-client", "device-code-123", 0)
	require.Error(t, err)
	var dfe *DeviceFlowError
	require.ErrorAs(t, err, &dfe)
	assert.Equal(t, "access_denied", dfe.ErrorCode)
}

func TestDeviceAuthClient_DeviceFlow_Integration(t *testing.T) {
	var tokenCallCount int32

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		require.NoError(t, r.ParseForm())

		// デバイスコードリクエスト
		if r.FormValue("client_id") != "" && r.FormValue("grant_type") == "" {
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(DeviceCodeResponse{
				DeviceCode:              "device-code-flow",
				UserCode:                "WXYZ-1234",
				VerificationURI:         "https://auth.example.com/device",
				VerificationURIComplete: "https://auth.example.com/device?user_code=WXYZ-1234",
				ExpiresIn:               600,
				Interval:                1,
			})
			return
		}

		// トークンリクエスト
		count := atomic.AddInt32(&tokenCallCount, 1)
		if count <= 1 {
			w.WriteHeader(http.StatusBadRequest)
			json.NewEncoder(w).Encode(map[string]string{
				"error": "authorization_pending",
			})
			return
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(TokenResult{
			AccessToken:  "flow-access-token",
			RefreshToken: "flow-refresh-token",
			TokenType:    "Bearer",
			ExpiresIn:    900,
		})
	}))
	defer server.Close()

	client := NewDeviceAuthClient(server.URL, server.URL)

	var receivedUserCode string
	var receivedVerificationURI string

	result, err := client.DeviceFlow(
		context.Background(),
		"test-client",
		"openid",
		func(resp *DeviceCodeResponse) {
			receivedUserCode = resp.UserCode
			receivedVerificationURI = resp.VerificationURI
		},
	)

	require.NoError(t, err)
	assert.Equal(t, "flow-access-token", result.AccessToken)
	assert.Equal(t, "flow-refresh-token", result.RefreshToken)
	assert.Equal(t, "WXYZ-1234", receivedUserCode)
	assert.Equal(t, "https://auth.example.com/device", receivedVerificationURI)
}

func TestDeviceAuthClient_PollToken_ContextCancellation(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(map[string]string{
			"error": "authorization_pending",
		})
	}))
	defer server.Close()

	client := NewDeviceAuthClient(server.URL, server.URL)

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	_, err := client.PollToken(ctx, "test-client", "device-code-123", 1)
	require.Error(t, err)
}
