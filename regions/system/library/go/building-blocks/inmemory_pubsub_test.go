package buildingblocks

import (
	"context"
	"testing"
	"time"
)

// InMemoryPubSub の Init 前後でステータスが Uninitialized → Ready に遷移することを確認する。
func TestInMemoryPubSub_InitAndStatus(t *testing.T) {
	ps := NewInMemoryPubSub()
	ctx := context.Background()

	if ps.Status(ctx) != StatusUninitialized {
		t.Errorf("expected StatusUninitialized, got %s", ps.Status(ctx))
	}
	if err := ps.Init(ctx, Metadata{}); err != nil {
		t.Fatalf("Init failed: %v", err)
	}
	if ps.Status(ctx) != StatusReady {
		t.Errorf("expected StatusReady, got %s", ps.Status(ctx))
	}
}

// InMemoryPubSub の Name と Version が正しい値を返すことを確認する。
func TestInMemoryPubSub_Name(t *testing.T) {
	ps := NewInMemoryPubSub()
	if ps.Name() != "inmemory-pubsub" {
		t.Errorf("unexpected Name: %q", ps.Name())
	}
	if ps.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", ps.Version())
	}
}

// InMemoryPubSub で Publish したメッセージが Subscribe したチャネルから受信できることを確認する。
func TestInMemoryPubSub_PublishSubscribe(t *testing.T) {
	ps := NewInMemoryPubSub()
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch, err := ps.Subscribe(ctx, "events")
	if err != nil {
		t.Fatalf("Subscribe failed: %v", err)
	}

	msg := &Message{Topic: "events", Data: []byte("hello"), ID: "1"}
	if err := ps.Publish(ctx, msg); err != nil {
		t.Fatalf("Publish failed: %v", err)
	}

	select {
	case received := <-ch:
		if received.ID != "1" {
			t.Errorf("expected ID '1', got %q", received.ID)
		}
		if string(received.Data) != "hello" {
			t.Errorf("expected Data 'hello', got %q", received.Data)
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for message")
	}
}

// InMemoryPubSub の Publish が受信メッセージの Timestamp を自動的に設定することを確認する。
func TestInMemoryPubSub_PublishSetsTimestamp(t *testing.T) {
	ps := NewInMemoryPubSub()
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch, _ := ps.Subscribe(ctx, "t")
	msg := &Message{Topic: "t", ID: "x"}
	_ = ps.Publish(ctx, msg)

	select {
	case received := <-ch:
		if received.Timestamp.IsZero() {
			t.Error("expected Timestamp to be set")
		}
	case <-time.After(time.Second):
		t.Fatal("timed out")
	}
}

// InMemoryPubSub が同一トピックの複数サブスクライバー全員にメッセージを配信することを確認する。
func TestInMemoryPubSub_MultipleSubscribers(t *testing.T) {
	ps := NewInMemoryPubSub()
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch1, _ := ps.Subscribe(ctx, "topic")
	ch2, _ := ps.Subscribe(ctx, "topic")

	_ = ps.Publish(ctx, &Message{Topic: "topic", ID: "m"})

	for _, ch := range []<-chan *Message{ch1, ch2} {
		select {
		case <-ch:
		case <-time.After(time.Second):
			t.Fatal("timed out waiting for subscriber")
		}
	}
}

// InMemoryPubSub が異なるトピックのサブスクライバーへメッセージを配信しないことを確認する。
func TestInMemoryPubSub_NoDeliveryToOtherTopics(t *testing.T) {
	ps := NewInMemoryPubSub()
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch, _ := ps.Subscribe(ctx, "other")
	_ = ps.Publish(ctx, &Message{Topic: "events", ID: "m"})

	select {
	case <-ch:
		t.Error("should not receive message for different topic")
	case <-time.After(50 * time.Millisecond):
	}
}

// InMemoryPubSub の Close がステータスを Closed に遷移させサブスクライバーチャネルを閉じることを確認する。
func TestInMemoryPubSub_Close(t *testing.T) {
	ps := NewInMemoryPubSub()
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch, _ := ps.Subscribe(ctx, "t")
	if err := ps.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if ps.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", ps.Status(ctx))
	}

	// channel should be closed
	_, ok := <-ch
	if ok {
		t.Error("expected channel to be closed")
	}
}
