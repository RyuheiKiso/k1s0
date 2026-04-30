// 本ファイルは G2 regression test。
//
// G2 bug: Dapr Go SDK の SaveState gRPC 応答 (Empty) には new ETag が含まれない。
// 旧実装は `StateSetResponse{NewEtag: ""}` を常に返していたため、共通規約 §「Dapr
// 互換性マトリクス」が要求する SetResponse.new_etag を満たしていなかった。
// 修正: Set 成功後に GetState を 1 回追加発行して新 ETag を取得する経路に変更。

package dapr

import (
	"context"
	"testing"

	daprclient "github.com/dapr/go-sdk/client"
)

// G2 regression: Set 成功後 GetState で取得した etag が NewEtag に詰められる。
// 直前の修正前は、SaveState 後の Get を呼ばず常に "" を返していた。
func TestStateAdapter_Set_ReturnsEtagFromGetAfterSave(t *testing.T) {
	saveCalled := 0
	getCalled := 0
	fake := &fakeStateClient{
		saveFn: func(_ context.Context, _, _ string, _ []byte, _ map[string]string, _ ...daprclient.StateOption) error {
			saveCalled++
			return nil
		},
		getFn: func(_ context.Context, _, _ string, _ map[string]string) (*daprclient.StateItem, error) {
			getCalled++
			return &daprclient.StateItem{Etag: "etag-v42"}, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Set(context.Background(), StateSetRequest{
		Store: "kv", Key: "k", Data: []byte("v"),
	})
	if err != nil {
		t.Fatalf("Set: %v", err)
	}
	if saveCalled != 1 {
		t.Fatalf("SaveState should be called exactly once, got %d", saveCalled)
	}
	if getCalled != 1 {
		t.Fatalf("post-save Get should be called exactly once, got %d", getCalled)
	}
	if resp.NewEtag != "etag-v42" {
		t.Fatalf("NewEtag should be propagated from post-save Get: got %q want etag-v42", resp.NewEtag)
	}
}

// G2 (b): GetState が失敗 / NotFound の場合、empty NewEtag に fallback する
// (race condition: 他 writer が直後に削除した可能性)。Set 自体は成功扱い。
func TestStateAdapter_Set_FallsBackToEmptyEtagOnGetFailure(t *testing.T) {
	fake := &fakeStateClient{
		saveFn: func(_ context.Context, _, _ string, _ []byte, _ map[string]string, _ ...daprclient.StateOption) error {
			return nil
		},
		getFn: func(_ context.Context, _, _ string, _ map[string]string) (*daprclient.StateItem, error) {
			return nil, nil // simulate "deleted between save and get"
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Set(context.Background(), StateSetRequest{
		Store: "kv", Key: "k", Data: []byte("v"),
	})
	if err != nil {
		t.Fatalf("Set should succeed even when post-save Get returns nil: %v", err)
	}
	if resp.NewEtag != "" {
		t.Fatalf("NewEtag should be empty fallback, got %q", resp.NewEtag)
	}
}
