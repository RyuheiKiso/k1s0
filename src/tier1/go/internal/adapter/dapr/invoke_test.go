// 本ファイルは daprInvokeAdapter の単体テスト。
// Dapr SDK の InvokeMethodWithCustomContent を fake で差し替え、
// adapter が SDK へ渡すメソッド・引数を直接検証する。

package dapr

import (
	"context"
	"errors"
	"testing"
)

// fakeInvokeClient は daprInvokeClient の最小 fake 実装。
type fakeInvokeClient struct {
	invokeFn func(ctx context.Context, appID, method, verb, contentType string, content interface{}) ([]byte, error)
}

func (f *fakeInvokeClient) InvokeMethodWithCustomContent(ctx context.Context, appID, method, verb, contentType string, content interface{}) ([]byte, error) {
	return f.invokeFn(ctx, appID, method, verb, contentType, content)
}

func newInvokeAdapterWithFake(t *testing.T, fake *fakeInvokeClient) InvokeAdapter {
	t.Helper()
	return NewInvokeAdapter(NewWithInvokeClient("test://noop", fake))
}

// 正常系: SDK に正しい AppID / Method / verb / ContentType / Data が渡ることを検証。
func TestInvokeAdapter_OK(t *testing.T) {
	fake := &fakeInvokeClient{
		invokeFn: func(_ context.Context, appID, method, verb, ct string, content interface{}) ([]byte, error) {
			if appID != "tier2-tax-calculator" || method != "calculate" {
				t.Fatalf("appid/method mismatch: %s / %s", appID, method)
			}
			if verb != "POST" {
				t.Fatalf("verb expected POST, got %s", verb)
			}
			if ct != "application/json" {
				t.Fatalf("content-type mismatch: %s", ct)
			}
			if d, ok := content.([]byte); !ok || string(d) != `{"amount":100}` {
				t.Fatalf("content mismatch: %v", content)
			}
			return []byte(`{"tax":10}`), nil
		},
	}
	a := newInvokeAdapterWithFake(t, fake)
	resp, err := a.Invoke(context.Background(), InvokeRequest{
		AppID:       "tier2-tax-calculator",
		Method:      "calculate",
		Data:        []byte(`{"amount":100}`),
		ContentType: "application/json",
	})
	if err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	if string(resp.Data) != `{"tax":10}` {
		t.Fatalf("response data mismatch: %s", resp.Data)
	}
	if resp.Status != 200 {
		t.Fatalf("status mismatch: %d", resp.Status)
	}
	if resp.ContentType != "application/json" {
		t.Fatalf("content-type echo mismatch: %s", resp.ContentType)
	}
}

// ContentType 空時に application/octet-stream が補完されることを検証。
func TestInvokeAdapter_DefaultContentType(t *testing.T) {
	var observedCT string
	fake := &fakeInvokeClient{
		invokeFn: func(_ context.Context, _, _, _, ct string, _ interface{}) ([]byte, error) {
			observedCT = ct
			return []byte("ok"), nil
		},
	}
	a := newInvokeAdapterWithFake(t, fake)
	if _, err := a.Invoke(context.Background(), InvokeRequest{AppID: "x", Method: "m", Data: []byte("d")}); err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	if observedCT != "application/octet-stream" {
		t.Fatalf("default content-type not applied: %q", observedCT)
	}
}

// SDK エラーが透過されることを検証。
func TestInvokeAdapter_SDKError(t *testing.T) {
	want := errors.New("upstream unavailable")
	fake := &fakeInvokeClient{
		invokeFn: func(_ context.Context, _, _, _, _ string, _ interface{}) ([]byte, error) {
			return nil, want
		},
	}
	a := newInvokeAdapterWithFake(t, fake)
	_, err := a.Invoke(context.Background(), InvokeRequest{AppID: "x", Method: "m"})
	if !errors.Is(err, want) {
		t.Fatalf("error not transparent: %v", err)
	}
}
