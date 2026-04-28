// 本ファイルは daprPubSubAdapter の単体テスト。
// Dapr SDK との結合点である daprPubSubClient を fake で差し替え、
// adapter が SDK へ渡すメソッド・引数・metadata を直接検証する。

package dapr

import (
	"context"
	"errors"
	"testing"

	daprclient "github.com/dapr/go-sdk/client"
)

// fakePubSubClient は daprPubSubClient の最小 fake 実装。
type fakePubSubClient struct {
	publishFn func(ctx context.Context, pubsubName, topicName string, data interface{}, opts ...daprclient.PublishEventOption) error
}

func (f *fakePubSubClient) PublishEvent(ctx context.Context, pubsubName, topicName string, data interface{}, opts ...daprclient.PublishEventOption) error {
	return f.publishFn(ctx, pubsubName, topicName, data, opts...)
}

func newPubSubAdapterWithFake(t *testing.T, fake *fakePubSubClient) PubSubAdapter {
	t.Helper()
	cli := NewWithPubSubClient("test://noop", fake)
	return NewPubSubAdapter(cli)
}

// Publish が SDK に正しい component / topic / data を渡すことを検証する。
func TestPubSubAdapter_Publish_OK(t *testing.T) {
	called := 0
	var observedComp, observedTopic string
	var observedData []byte
	fake := &fakePubSubClient{
		publishFn: func(_ context.Context, pubsubName, topicName string, data interface{}, _ ...daprclient.PublishEventOption) error {
			called++
			observedComp = pubsubName
			observedTopic = topicName
			if d, ok := data.([]byte); ok {
				observedData = d
			}
			return nil
		},
	}
	a := newPubSubAdapterWithFake(t, fake)
	resp, err := a.Publish(context.Background(), PublishRequest{
		Component:   "pubsub-kafka",
		Topic:       "k1s0.events.user-created",
		Data:        []byte(`{"user_id":"42"}`),
		ContentType: "application/json",
		TenantID:    "tenant-A",
	})
	if err != nil {
		t.Fatalf("Publish error: %v", err)
	}
	if called != 1 {
		t.Fatalf("PublishEvent not called once: %d", called)
	}
	if observedComp != "pubsub-kafka" || observedTopic != "k1s0.events.user-created" {
		t.Fatalf("component/topic mismatch: %s / %s", observedComp, observedTopic)
	}
	if string(observedData) != `{"user_id":"42"}` {
		t.Fatalf("data mismatch: got %q", observedData)
	}
	// SDK は offset を返さないので 0 確定。
	if resp.Offset != 0 {
		t.Fatalf("expected offset=0, got %d", resp.Offset)
	}
}

// Publish が SDK エラーを上位に透過することを検証する。
func TestPubSubAdapter_Publish_SDKError(t *testing.T) {
	want := errors.New("rpc deadline exceeded")
	fake := &fakePubSubClient{
		publishFn: func(_ context.Context, _, _ string, _ interface{}, _ ...daprclient.PublishEventOption) error {
			return want
		},
	}
	a := newPubSubAdapterWithFake(t, fake)
	_, err := a.Publish(context.Background(), PublishRequest{Component: "c", Topic: "t", Data: []byte("d")})
	if !errors.Is(err, want) {
		t.Fatalf("error not transparent: got %v", err)
	}
}

// metadata 合成: 利用側 metadata + tenantId + idempotencyKey の優先順位検証。
func TestBuildPubSubMeta(t *testing.T) {
	tests := []struct {
		name           string
		tenantID       string
		idempotencyKey string
		extra          map[string]string
		want           map[string]string
	}{
		{name: "all empty", want: nil},
		{
			name:     "tenant only",
			tenantID: "T",
			want:     map[string]string{"tenantId": "T"},
		},
		{
			name:           "all three",
			tenantID:       "T",
			idempotencyKey: "ID-1",
			extra:          map[string]string{"partitionKey": "P"},
			want: map[string]string{
				"tenantId":       "T",
				"idempotencyKey": "ID-1",
				"partitionKey":   "P",
			},
		},
		{
			name:     "extra tenantId is overridden",
			tenantID: "T",
			extra:    map[string]string{"tenantId": "WRONG", "partitionKey": "P"},
			want: map[string]string{
				"tenantId":     "T",
				"partitionKey": "P",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := buildPubSubMeta(tt.tenantID, tt.idempotencyKey, tt.extra)
			if len(got) != len(tt.want) {
				t.Fatalf("len mismatch: got=%v want=%v", got, tt.want)
			}
			for k, v := range tt.want {
				if got[k] != v {
					t.Fatalf("key %q: got %q want %q", k, got[k], v)
				}
			}
		})
	}
}
