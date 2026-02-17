package auth

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

// DeviceCodeResponse はデバイス認可リクエストのレスポンス（RFC 8628）。
type DeviceCodeResponse struct {
	DeviceCode              string `json:"device_code"`
	UserCode                string `json:"user_code"`
	VerificationURI         string `json:"verification_uri"`
	VerificationURIComplete string `json:"verification_uri_complete,omitempty"`
	ExpiresIn               int    `json:"expires_in"`
	Interval                int    `json:"interval"`
}

// TokenResult はトークンエンドポイントのレスポンス。
type TokenResult struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token,omitempty"`
	TokenType    string `json:"token_type"`
	ExpiresIn    int    `json:"expires_in"`
}

// DeviceFlowError は Device Authorization Grant フローのエラー。
type DeviceFlowError struct {
	ErrorCode   string `json:"error"`
	Description string `json:"error_description,omitempty"`
}

func (e *DeviceFlowError) Error() string {
	if e.Description != "" {
		return fmt.Sprintf("device flow error: %s (%s)", e.ErrorCode, e.Description)
	}
	return fmt.Sprintf("device flow error: %s", e.ErrorCode)
}

// DeviceCodeCallback はユーザーにデバイスコード情報を表示するためのコールバック。
type DeviceCodeCallback func(resp *DeviceCodeResponse)

// DeviceAuthClient は Device Authorization Grant フロー（RFC 8628）のクライアント。
type DeviceAuthClient struct {
	deviceEndpoint string
	tokenEndpoint  string
	httpClient     *http.Client
}

// NewDeviceAuthClient は DeviceAuthClient を生成する。
func NewDeviceAuthClient(deviceEndpoint, tokenEndpoint string) *DeviceAuthClient {
	return &DeviceAuthClient{
		deviceEndpoint: deviceEndpoint,
		tokenEndpoint:  tokenEndpoint,
		httpClient:     &http.Client{Timeout: 30 * time.Second},
	}
}

// RequestDeviceCode はデバイス認可リクエストを送信し、デバイスコード情報を返す。
func (c *DeviceAuthClient) RequestDeviceCode(ctx context.Context, clientID, scope string) (*DeviceCodeResponse, error) {
	data := url.Values{
		"client_id": {clientID},
	}
	if scope != "" {
		data.Set("scope", scope)
	}

	req, err := http.NewRequestWithContext(ctx, "POST", c.deviceEndpoint, strings.NewReader(data.Encode()))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("device code request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("device code request failed with status %d: %s", resp.StatusCode, string(body))
	}

	var result DeviceCodeResponse
	if err := json.Unmarshal(body, &result); err != nil {
		return nil, fmt.Errorf("failed to parse device code response: %w", err)
	}

	return &result, nil
}

// PollToken は device_code を使ってトークンエンドポイントをポーリングする。
// interval が 0 の場合はデフォルトの 5 秒を使用する。
func (c *DeviceAuthClient) PollToken(ctx context.Context, clientID, deviceCode string, interval int) (*TokenResult, error) {
	if interval <= 0 {
		interval = 5
	}

	for {
		data := url.Values{
			"grant_type":  {"urn:ietf:params:oauth:grant-type:device_code"},
			"device_code": {deviceCode},
			"client_id":   {clientID},
		}

		req, err := http.NewRequestWithContext(ctx, "POST", c.tokenEndpoint, strings.NewReader(data.Encode()))
		if err != nil {
			return nil, fmt.Errorf("failed to create request: %w", err)
		}
		req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

		resp, err := c.httpClient.Do(req)
		if err != nil {
			return nil, fmt.Errorf("token request failed: %w", err)
		}

		body, err := io.ReadAll(resp.Body)
		resp.Body.Close()
		if err != nil {
			return nil, fmt.Errorf("failed to read response: %w", err)
		}

		// 成功レスポンス
		if resp.StatusCode == http.StatusOK {
			var result TokenResult
			if err := json.Unmarshal(body, &result); err != nil {
				return nil, fmt.Errorf("failed to parse token response: %w", err)
			}
			return &result, nil
		}

		// エラーレスポンスをパース
		var errResp struct {
			Error            string `json:"error"`
			ErrorDescription string `json:"error_description"`
		}
		if err := json.Unmarshal(body, &errResp); err != nil {
			return nil, fmt.Errorf("failed to parse error response: %w", err)
		}

		switch errResp.Error {
		case "authorization_pending":
			// 待機して再ポーリング
		case "slow_down":
			// interval を 5 秒増加（RFC 8628 Section 3.5）
			interval += 5
		case "expired_token":
			return nil, &DeviceFlowError{ErrorCode: "expired_token", Description: errResp.ErrorDescription}
		case "access_denied":
			return nil, &DeviceFlowError{ErrorCode: "access_denied", Description: errResp.ErrorDescription}
		default:
			return nil, &DeviceFlowError{ErrorCode: errResp.Error, Description: errResp.ErrorDescription}
		}

		// interval 秒待機
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-time.After(time.Duration(interval) * time.Second):
		}
	}
}

// DeviceFlow は Device Authorization Grant フロー全体を実行する統合メソッド。
// onUserCode コールバックでユーザーにデバイスコード情報を通知する。
func (c *DeviceAuthClient) DeviceFlow(ctx context.Context, clientID, scope string, onUserCode DeviceCodeCallback) (*TokenResult, error) {
	deviceResp, err := c.RequestDeviceCode(ctx, clientID, scope)
	if err != nil {
		return nil, fmt.Errorf("failed to request device code: %w", err)
	}

	if onUserCode != nil {
		onUserCode(deviceResp)
	}

	return c.PollToken(ctx, clientID, deviceResp.DeviceCode, deviceResp.Interval)
}
