// 本ファイルは daprBindingAdapter の単体テスト。
// Dapr SDK InvokeBinding を fake で差し替え、
// adapter が SDK へ渡すメソッド・引数・metadata を直接検証する。

package dapr

import (
	"context"
	"errors"
	"testing"

	daprclient "github.com/dapr/go-sdk/client"
)

// fakeBindingClient は daprBindingClient の最小 fake 実装。
type fakeBindingClient struct {
	invokeFn func(ctx context.Context, in *daprclient.InvokeBindingRequest) (*daprclient.BindingEvent, error)
}

func (f *fakeBindingClient) InvokeBinding(ctx context.Context, in *daprclient.InvokeBindingRequest) (*daprclient.BindingEvent, error) {
	return f.invokeFn(ctx, in)
}

func newBindingAdapterWithFake(t *testing.T, fake *fakeBindingClient) BindingAdapter {
	t.Helper()
	return NewBindingAdapter(NewWithBindingClient("test://noop", fake))
}

// 正常系: SDK に Name / Operation / Data / Metadata（tenant 含む）を渡し、応答を返すことを検証。
func TestBindingAdapter_OK(t *testing.T) {
	fake := &fakeBindingClient{
		invokeFn: func(_ context.Context, in *daprclient.InvokeBindingRequest) (*daprclient.BindingEvent, error) {
			if in.Name != "s3-archive" || in.Operation != "create" {
				t.Fatalf("name/op mismatch: %s / %s", in.Name, in.Operation)
			}
			if string(in.Data) != "payload" {
				t.Fatalf("data mismatch: %s", in.Data)
			}
			if in.Metadata["tenantId"] != "tenant-A" {
				t.Fatalf("tenant metadata not propagated: %v", in.Metadata)
			}
			if in.Metadata["bucket"] != "k1s0-archive" {
				t.Fatalf("user metadata not propagated: %v", in.Metadata)
			}
			return &daprclient.BindingEvent{Data: []byte("ack"), Metadata: map[string]string{"etag": "abc123"}}, nil
		},
	}
	a := newBindingAdapterWithFake(t, fake)
	resp, err := a.Invoke(context.Background(), BindingRequest{
		Name:      "s3-archive",
		Operation: "create",
		Data:      []byte("payload"),
		Metadata:  map[string]string{"bucket": "k1s0-archive"},
		TenantID:  "tenant-A",
	})
	if err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	if string(resp.Data) != "ack" {
		t.Fatalf("response data mismatch: %s", resp.Data)
	}
	if resp.Metadata["etag"] != "abc123" {
		t.Fatalf("response metadata mismatch: %v", resp.Metadata)
	}
}

// SDK が nil event を返した場合、空 BindingResponse を返すことを検証。
func TestBindingAdapter_NilEvent(t *testing.T) {
	fake := &fakeBindingClient{
		invokeFn: func(_ context.Context, _ *daprclient.InvokeBindingRequest) (*daprclient.BindingEvent, error) {
			return nil, nil
		},
	}
	a := newBindingAdapterWithFake(t, fake)
	resp, err := a.Invoke(context.Background(), BindingRequest{Name: "n", Operation: "send"})
	if err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	if resp.Data != nil || resp.Metadata != nil {
		t.Fatalf("expected empty response, got %+v", resp)
	}
}

// SDK エラーが透過されることを検証。
func TestBindingAdapter_SDKError(t *testing.T) {
	want := errors.New("smtp send failed")
	fake := &fakeBindingClient{
		invokeFn: func(_ context.Context, _ *daprclient.InvokeBindingRequest) (*daprclient.BindingEvent, error) {
			return nil, want
		},
	}
	a := newBindingAdapterWithFake(t, fake)
	_, err := a.Invoke(context.Background(), BindingRequest{Name: "n", Operation: "send"})
	if !errors.Is(err, want) {
		t.Fatalf("error not transparent: %v", err)
	}
}

// 呼出元 metadata map が破壊されないことを検証（adapter が内部コピーを作るべき）。
func TestBindingAdapter_PreservesCallerMetadata(t *testing.T) {
	original := map[string]string{"k1": "v1"}
	fake := &fakeBindingClient{
		invokeFn: func(_ context.Context, in *daprclient.InvokeBindingRequest) (*daprclient.BindingEvent, error) {
			// adapter 側で tenantId が追加されるため、コピーが渡されるはず。
			in.Metadata["mutated"] = "by-fake"
			return &daprclient.BindingEvent{}, nil
		},
	}
	a := newBindingAdapterWithFake(t, fake)
	if _, err := a.Invoke(context.Background(), BindingRequest{
		Name:     "n",
		Metadata: original,
		TenantID: "T",
	}); err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	// 元 map が変化していないことを確認。
	if _, ok := original["mutated"]; ok {
		t.Fatalf("caller metadata was mutated by adapter")
	}
	if _, ok := original["tenantId"]; ok {
		t.Fatalf("caller metadata was extended with tenantId")
	}
}
