package webhookclient

import (
	"bytes"
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

// WebhookPayload はWebhookのペイロード。
type WebhookPayload struct {
	EventType string         `json:"event_type"`
	Timestamp string         `json:"timestamp"`
	Data      map[string]any `json:"data"`
}

// GenerateSignature はHMAC-SHA256署名を生成する。
func GenerateSignature(secret string, body []byte) string {
	mac := hmac.New(sha256.New, []byte(secret))
	mac.Write(body)
	return hex.EncodeToString(mac.Sum(nil))
}

// VerifySignature はHMAC-SHA256署名を検証する。
func VerifySignature(secret string, body []byte, sig string) bool {
	expected := GenerateSignature(secret, body)
	return hmac.Equal([]byte(expected), []byte(sig))
}

// WebhookClient はWebhookクライアントのインターフェース。
type WebhookClient interface {
	Send(ctx context.Context, url string, payload *WebhookPayload) (int, error)
}

// HTTPWebhookClient はHTTPベースのWebhookクライアント。
type HTTPWebhookClient struct {
	Secret     string
	HTTPClient *http.Client
}

// NewHTTPWebhookClient は新しい HTTPWebhookClient を生成する。
func NewHTTPWebhookClient(secret string) *HTTPWebhookClient {
	return &HTTPWebhookClient{
		Secret: secret,
		HTTPClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (c *HTTPWebhookClient) Send(ctx context.Context, url string, payload *WebhookPayload) (int, error) {
	body, err := json.Marshal(payload)
	if err != nil {
		return 0, fmt.Errorf("ペイロードのJSON変換に失敗: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return 0, fmt.Errorf("リクエスト作成に失敗: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Webhook-Signature", GenerateSignature(c.Secret, body))

	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return 0, fmt.Errorf("Webhook送信に失敗: %w", err)
	}
	defer resp.Body.Close()
	return resp.StatusCode, nil
}
