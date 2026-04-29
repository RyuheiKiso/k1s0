// 本ファイルは ETag 必須化（共通規約 §「Dapr 互換性マトリクス」: First-Write-Wins）の検証。
//
// docs 規定:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「Dapr 互換性マトリクス」
//     "ETag 必須化（Dapr は任意）"
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「エラー型 K1s0Error」
//     "AlreadyExists / Conflict — ETag 不一致、冪等性キー衝突"
//
// 検証する挙動:
//   1. 新規キーへの Set（ETag 空）は成功する
//   2. 既存キーへの Set（ETag 空）は AlreadyExists（First-Write 違反）
//   3. 既存キーへの Set（正しい ETag）は ETag 必須化のために Get → Set サイクルが必要だが、
//      in-memory backend が ETag を返さないため、ここでは Get → ETag 取得 → Set でもう一度確認
//   4. 既存キーへの Set（誤った ETag）は AlreadyExists

package state

import (
	"context"
	"testing"

	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// First write は成功、second write（ETag 空）は AlreadyExists。
func TestETagEnforcement_SecondWriteWithoutETag_AlreadyExists(t *testing.T) {
	c, cleanup := startStateServerWithInMemoryDapr(t)
	defer cleanup()
	ctx := context.Background()

	// 1. 新規キーへの Set: 成功する。
	if _, err := c.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default", Key: "etag-test/k1", Data: []byte("v1"),
		Context: makeTenantCtx("T"),
	}); err != nil {
		t.Fatalf("first Set should succeed: %v", err)
	}

	// 2. 既存キーへの Set（ETag 空）: AlreadyExists が返る。
	_, err := c.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default", Key: "etag-test/k1", Data: []byte("v2"),
		Context: makeTenantCtx("T"),
	})
	if status.Code(err) != grpccodes.AlreadyExists {
		t.Fatalf("second Set without ETag should be AlreadyExists, got %v", err)
	}
}

// 既存キーへの Set（誤った ETag）は AlreadyExists / Conflict。
func TestETagEnforcement_WrongETag_AlreadyExists(t *testing.T) {
	c, cleanup := startStateServerWithInMemoryDapr(t)
	defer cleanup()
	ctx := context.Background()

	// 1. 新規 Set。
	if _, err := c.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default", Key: "etag-test/k2", Data: []byte("v1"),
		Context: makeTenantCtx("T"),
	}); err != nil {
		t.Fatalf("first Set: %v", err)
	}

	// 2. 誤った ETag で Set: AlreadyExists / Conflict。
	_, err := c.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default", Key: "etag-test/k2", Data: []byte("v2"),
		ExpectedEtag: "wrong-etag",
		Context:      makeTenantCtx("T"),
	})
	if status.Code(err) != grpccodes.AlreadyExists {
		t.Fatalf("wrong ETag Set should be AlreadyExists, got %v", err)
	}
}

// 正しい ETag で Set すると更新が成功する。
// （adapter は SaveStateWithETag 経由、in-memory backend が etag を比較する）
func TestETagEnforcement_CorrectETag_Succeeds(t *testing.T) {
	c, cleanup := startStateServerWithInMemoryDapr(t)
	defer cleanup()
	ctx := context.Background()

	// 1. 新規 Set。
	if _, err := c.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default", Key: "etag-test/k3", Data: []byte("v1"),
		Context: makeTenantCtx("T"),
	}); err != nil {
		t.Fatalf("first Set: %v", err)
	}
	// 2. Get で ETag 取得。
	got, err := c.Get(ctx, &statev1.GetRequest{
		Store: "valkey-default", Key: "etag-test/k3", Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if got.GetEtag() == "" {
		t.Fatalf("expected non-empty etag after first Set")
	}
	// 3. 正しい ETag で更新: 成功する。
	if _, err := c.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default", Key: "etag-test/k3", Data: []byte("v2"),
		ExpectedEtag: got.GetEtag(),
		Context:      makeTenantCtx("T"),
	}); err != nil {
		t.Fatalf("Set with correct ETag should succeed: %v", err)
	}
	// 4. Get で v2 が返る。
	got2, err := c.Get(ctx, &statev1.GetRequest{
		Store: "valkey-default", Key: "etag-test/k3", Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Get after update: %v", err)
	}
	if string(got2.GetData()) != "v2" {
		t.Fatalf("expected updated data v2, got %q", got2.GetData())
	}
}
