package notificationclient_test

import (
	"context"
	"testing"

	"github.com/k1s0-platform/system-library-go-notification-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

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

func TestSentRequests_Empty(t *testing.T) {
	c := notificationclient.NewInMemoryClient()
	assert.Empty(t, c.SentRequests())
}

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
