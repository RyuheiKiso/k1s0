package webhookclient

import (
	"bytes"
	"context"
	"crypto/hmac"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"math/big"
	"net/http"
	"time"
)

// SignatureHeader は署名ヘッダー名。
const SignatureHeader = "X-K1s0-Signature"

// IdempotencyKeyHeader はべき等性キーのヘッダー名。
const IdempotencyKeyHeader = "Idempotency-Key"

// WebhookPayload はWebhookのペイロード。
type WebhookPayload struct {
	EventType string         `json:"event_type"`
	Timestamp string         `json:"timestamp"`
	Data      map[string]any `json:"data"`
}

// WebhookConfig はリトライ設定。
type WebhookConfig struct {
	MaxRetries       int
	InitialBackoffMs int
	MaxBackoffMs     int
}

// DefaultWebhookConfig はデフォルトのリトライ設定を返す。
func DefaultWebhookConfig() WebhookConfig {
	return WebhookConfig{
		MaxRetries:       3,
		InitialBackoffMs: 100,
		MaxBackoffMs:     10000,
	}
}

// MaxRetriesExceededError はリトライ上限到達エラー。
type MaxRetriesExceededError struct {
	Attempts       int
	LastStatusCode int
}

func (e *MaxRetriesExceededError) Error() string {
	return fmt.Sprintf("リトライ上限到達: %d回試行, 最終ステータス=%d", e.Attempts, e.LastStatusCode)
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

// generateUUIDv4 はUUID v4を生成する。
func generateUUIDv4() string {
	var uuid [16]byte
	_, err := rand.Read(uuid[:])
	if err != nil {
		// crypto/rand の失敗は致命的だが、ここでは空文字列を返さずフォールバック
		return fmt.Sprintf("%d", time.Now().UnixNano())
	}
	uuid[6] = (uuid[6] & 0x0f) | 0x40 // version 4
	uuid[8] = (uuid[8] & 0x3f) | 0x80 // variant 10
	return fmt.Sprintf("%08x-%04x-%04x-%04x-%012x",
		uuid[0:4], uuid[4:6], uuid[6:8], uuid[8:10], uuid[10:16])
}

// isRetryableStatus はリトライ対象のステータスコードかどうかを判定する。
func isRetryableStatus(statusCode int) bool {
	return statusCode == http.StatusTooManyRequests || statusCode >= 500
}

// calculateBackoff は指数バックオフ + ジッターの待機時間を計算する。
func calculateBackoff(attempt int, initialBackoffMs int, maxBackoffMs int) time.Duration {
	backoff := initialBackoffMs
	for i := 0; i < attempt; i++ {
		backoff *= 2
		if backoff > maxBackoffMs {
			backoff = maxBackoffMs
			break
		}
	}
	// ジッター: 0 ~ backoff/2 のランダム値を加算
	jitterMax := backoff / 2
	if jitterMax <= 0 {
		jitterMax = 1
	}
	jitterBig, err := rand.Int(rand.Reader, big.NewInt(int64(jitterMax)))
	jitter := 0
	if err == nil {
		jitter = int(jitterBig.Int64())
	}
	return time.Duration(backoff+jitter) * time.Millisecond
}

// sleepFunc はテストでオーバーライドするためのスリープ関数。
var sleepFunc = func(d time.Duration) {
	time.Sleep(d)
}

// WebhookClient はWebhookクライアントのインターフェース。
type WebhookClient interface {
	Send(ctx context.Context, url string, payload *WebhookPayload) (int, error)
}

// HTTPWebhookClient はHTTPベースのWebhookクライアント。
type HTTPWebhookClient struct {
	Secret     string
	Config     WebhookConfig
	HTTPClient *http.Client
}

// NewHTTPWebhookClient は新しい HTTPWebhookClient を生成する。
func NewHTTPWebhookClient(secret string) *HTTPWebhookClient {
	return &HTTPWebhookClient{
		Secret: secret,
		Config: DefaultWebhookConfig(),
		HTTPClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// NewHTTPWebhookClientWithConfig はリトライ設定付きの HTTPWebhookClient を生成する。
func NewHTTPWebhookClientWithConfig(secret string, config WebhookConfig) *HTTPWebhookClient {
	return &HTTPWebhookClient{
		Secret: secret,
		Config: config,
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

	signature := GenerateSignature(c.Secret, body)
	idempotencyKey := generateUUIDv4()
	maxAttempts := c.Config.MaxRetries + 1
	var lastStatusCode int

	for attempt := 0; attempt < maxAttempts; attempt++ {
		if attempt > 0 {
			delay := calculateBackoff(attempt-1, c.Config.InitialBackoffMs, c.Config.MaxBackoffMs)
			log.Printf("[webhook-client] リトライ attempt=%d/%d, 待機=%v, url=%s", attempt+1, maxAttempts, delay, url)
			sleepFunc(delay)
		} else {
			log.Printf("[webhook-client] 送信開始 url=%s, idempotency_key=%s", url, idempotencyKey)
		}

		req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
		if err != nil {
			return 0, fmt.Errorf("リクエスト作成に失敗: %w", err)
		}
		req.Header.Set("Content-Type", "application/json")
		req.Header.Set(SignatureHeader, signature)
		req.Header.Set(IdempotencyKeyHeader, idempotencyKey)

		resp, err := c.HTTPClient.Do(req)
		if err != nil {
			log.Printf("[webhook-client] 送信エラー attempt=%d/%d, error=%v", attempt+1, maxAttempts, err)
			if attempt == maxAttempts-1 {
				return 0, fmt.Errorf("Webhook送信に失敗: %w", err)
			}
			continue
		}
		resp.Body.Close()
		lastStatusCode = resp.StatusCode

		if !isRetryableStatus(lastStatusCode) {
			log.Printf("[webhook-client] 送信完了 status=%d, attempt=%d/%d", lastStatusCode, attempt+1, maxAttempts)
			return lastStatusCode, nil
		}

		log.Printf("[webhook-client] リトライ対象ステータス status=%d, attempt=%d/%d", lastStatusCode, attempt+1, maxAttempts)
	}

	return lastStatusCode, &MaxRetriesExceededError{
		Attempts:       maxAttempts,
		LastStatusCode: lastStatusCode,
	}
}
