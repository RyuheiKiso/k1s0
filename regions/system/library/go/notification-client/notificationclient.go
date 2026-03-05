package notificationclient

import (
	"context"
	"sync"
)

// Channel は通知チャンネル。
type Channel string

const (
	ChannelEmail   Channel = "email"
	ChannelSMS     Channel = "sms"
	ChannelPush    Channel = "push"
	ChannelSlack   Channel = "slack"
	ChannelWebhook Channel = "webhook"
)

// NotificationRequest は通知リクエスト。
type NotificationRequest struct {
	ID        string                 `json:"id"`
	Channel   Channel                `json:"channel"`
	Recipient string                 `json:"recipient"`
	Subject   string                 `json:"subject,omitempty"`
	Body      string                 `json:"body"`
	Metadata  map[string]interface{} `json:"metadata,omitempty"`
}

// NotificationResponse は通知レスポンス。
type NotificationResponse struct {
	ID        string `json:"id"`
	Status    string `json:"status"`
	MessageID string `json:"message_id,omitempty"`
}

// SendNotificationInput は単一送信入力のエイリアス。
type SendNotificationInput = NotificationRequest

// SendNotificationOutput は単一送信出力のエイリアス。
type SendNotificationOutput = NotificationResponse

// NotificationClient は通知クライアントのインターフェース。
type NotificationClient interface {
	Send(ctx context.Context, req NotificationRequest) (NotificationResponse, error)
	SendBatch(ctx context.Context, reqs []SendNotificationInput) ([]SendNotificationOutput, error)
}

// InMemoryClient はメモリ内の通知クライアント。
type InMemoryClient struct {
	mu   sync.Mutex
	sent []NotificationRequest
	seq  int
}

// NewInMemoryClient は新しい InMemoryClient を生成する。
func NewInMemoryClient() *InMemoryClient {
	return &InMemoryClient{}
}

func (c *InMemoryClient) Send(_ context.Context, req NotificationRequest) (NotificationResponse, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.sent = append(c.sent, req)
	c.seq++
	return NotificationResponse{
		ID:        req.ID,
		Status:    "sent",
		MessageID: req.ID + "-msg",
	}, nil
}

func (c *InMemoryClient) SendBatch(
	_ context.Context,
	reqs []SendNotificationInput,
) ([]SendNotificationOutput, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	results := make([]SendNotificationOutput, 0, len(reqs))
	for _, req := range reqs {
		c.sent = append(c.sent, req)
		c.seq++
		results = append(results, SendNotificationOutput{
			ID:        req.ID,
			Status:    "sent",
			MessageID: req.ID + "-msg",
		})
	}
	return results, nil
}

// SentRequests は送信済みリクエストを返す。
func (c *InMemoryClient) SentRequests() []NotificationRequest {
	c.mu.Lock()
	defer c.mu.Unlock()
	result := make([]NotificationRequest, len(c.sent))
	copy(result, c.sent)
	return result
}
