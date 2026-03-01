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
	ID        string  `json:"id"`
	Channel   Channel `json:"channel"`
	Recipient string  `json:"recipient"`
	Subject   string  `json:"subject,omitempty"`
	Body      string  `json:"body"`
}

// NotificationResponse は通知レスポンス。
type NotificationResponse struct {
	ID        string `json:"id"`
	Status    string `json:"status"`
	MessageID string `json:"message_id,omitempty"`
}

// NotificationClient は通知クライアントのインターフェース。
type NotificationClient interface {
	Send(ctx context.Context, req NotificationRequest) (NotificationResponse, error)
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

// SentRequests は送信済みリクエストを返す。
func (c *InMemoryClient) SentRequests() []NotificationRequest {
	c.mu.Lock()
	defer c.mu.Unlock()
	result := make([]NotificationRequest, len(c.sent))
	copy(result, c.sent)
	return result
}
