package notificationclient_test

import (
	"context"
	"testing"

	"github.com/k1s0-platform/system-library-go-notification-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// SendがEmailチャンネルへの通知リクエストに対して正しいレスポンスを返すことを確認する。
func TestSend_ReturnsResponse(t *testing.T) {
	c := notificationclient.NewInMemoryClient()
	resp, err := c.Send(context.Background(), notificationclient.NotificationRequest{
		ID:        "n-1",
		Channel:   notificationclient.ChannelEmail,
		Recipient: "user@example.com",
		Subject:   "Test",
		Body:      "Hello",
	})
	require.NoError(t, err)
	assert.Equal(t, "n-1", resp.ID)
	assert.Equal(t, "sent", resp.Status)
	assert.Equal(t, "n-1-msg", resp.MessageID)
}

// 複数回Sendを呼び出した際に送信済みリクエストが全件記録されることを確認する。
func TestSend_RecordsSentRequests(t *testing.T) {
	c := notificationclient.NewInMemoryClient()
	ctx := context.Background()

	_, _ = c.Send(ctx, notificationclient.NotificationRequest{
		ID: "n-1", Channel: notificationclient.ChannelEmail, Recipient: "a@b.com", Body: "hi",
	})
	_, _ = c.Send(ctx, notificationclient.NotificationRequest{
		ID: "n-2", Channel: notificationclient.ChannelSMS, Recipient: "+1234", Body: "sms",
	})

	sent := c.SentRequests()
	assert.Len(t, sent, 2)
	assert.Equal(t, "n-1", sent[0].ID)
	assert.Equal(t, "n-2", sent[1].ID)
}

// 初期状態でSentRequestsが空スライスを返すことを確認する。
func TestSentRequests_Empty(t *testing.T) {
	c := notificationclient.NewInMemoryClient()
	assert.Empty(t, c.SentRequests())
}

// SendがPushチャンネルへの通知を正常に送信できることを確認する。
func TestSend_Push(t *testing.T) {
	c := notificationclient.NewInMemoryClient()
	resp, err := c.Send(context.Background(), notificationclient.NotificationRequest{
		ID:      "n-3",
		Channel: notificationclient.ChannelPush,
		Body:    "push notification",
	})
	require.NoError(t, err)
	assert.Equal(t, "sent", resp.Status)
	assert.Equal(t, notificationclient.ChannelPush, c.SentRequests()[0].Channel)
}

// SendがSlackチャンネルへの通知を正常に送信できることを確認する。
func TestSend_Slack(t *testing.T) {
	c := notificationclient.NewInMemoryClient()
	resp, err := c.Send(context.Background(), notificationclient.NotificationRequest{
		ID:        "n-4",
		Channel:   notificationclient.ChannelSlack,
		Recipient: "#general",
		Body:      "slack notification",
	})
	require.NoError(t, err)
	assert.Equal(t, "sent", resp.Status)
	assert.Equal(t, notificationclient.ChannelSlack, c.SentRequests()[0].Channel)
}

// SendがWebhookチャンネルへの通知を正常に送信できることを確認する。
func TestSend_Webhook(t *testing.T) {
	c := notificationclient.NewInMemoryClient()
	resp, err := c.Send(context.Background(), notificationclient.NotificationRequest{
		ID:        "n-5",
		Channel:   notificationclient.ChannelWebhook,
		Recipient: "https://example.com/webhook",
		Body:      "webhook payload",
	})
	require.NoError(t, err)
	assert.Equal(t, "sent", resp.Status)
	assert.Equal(t, notificationclient.ChannelWebhook, c.SentRequests()[0].Channel)
}

// SendBatchが複数の通知リクエストをまとめて送信し全レスポンスを返すことを確認する。
func TestSendBatch_ReturnsResponses(t *testing.T) {
	c := notificationclient.NewInMemoryClient()
	ctx := context.Background()

	inputs := []notificationclient.SendNotificationInput{
		{
			ID:        "n-b1",
			Channel:   notificationclient.ChannelEmail,
			Recipient: "u1@example.com",
			Body:      "hello",
		},
		{
			ID:        "n-b2",
			Channel:   notificationclient.ChannelSMS,
			Recipient: "+819000000000",
			Body:      "sms",
		},
	}

	out, err := c.SendBatch(ctx, inputs)
	require.NoError(t, err)
	assert.Len(t, out, 2)
	assert.Equal(t, "n-b1", out[0].ID)
	assert.Equal(t, "sent", out[0].Status)
	assert.Equal(t, "n-b2", out[1].ID)
	assert.Len(t, c.SentRequests(), 2)
}
