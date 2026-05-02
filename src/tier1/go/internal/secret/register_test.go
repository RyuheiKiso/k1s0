// 本ファイルは SecretsService handler の単体テスト。
// fake openbao.SecretsAdapter で SDK 結合点を切り離し、handler の責務を検証する。

package secret

import (
	"context"
	"errors"
	"net"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	secretsv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/secrets/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
)

const bufSize = 1024 * 1024

// fakeSecretsAdapter は openbao.SecretsAdapter の最小 fake 実装。
type fakeSecretsAdapter struct {
	getFn         func(ctx context.Context, req openbao.SecretGetRequest) (openbao.SecretGetResponse, error)
	bulkGetFn     func(ctx context.Context, names []string, tenantID string) (map[string]openbao.SecretGetResponse, error)
	listAndGetFn  func(ctx context.Context, tenantID string) (map[string]openbao.SecretGetResponse, error)
	rotateFn      func(ctx context.Context, req openbao.SecretRotateRequest) (openbao.SecretGetResponse, error)
}

func (f *fakeSecretsAdapter) Get(ctx context.Context, req openbao.SecretGetRequest) (openbao.SecretGetResponse, error) {
	return f.getFn(ctx, req)
}
func (f *fakeSecretsAdapter) BulkGet(ctx context.Context, names []string, tenantID string) (map[string]openbao.SecretGetResponse, error) {
	if f.bulkGetFn == nil {
		return map[string]openbao.SecretGetResponse{}, nil
	}
	return f.bulkGetFn(ctx, names, tenantID)
}
func (f *fakeSecretsAdapter) ListAndGet(ctx context.Context, tenantID string) (map[string]openbao.SecretGetResponse, error) {
	// 未注入時は空 map を返す（既存テストでは BulkGet を直接呼ばないため互換維持目的）。
	if f.listAndGetFn == nil {
		// 空 map を返す。
		return map[string]openbao.SecretGetResponse{}, nil
	}
	// 注入関数に委譲する。
	return f.listAndGetFn(ctx, tenantID)
}
func (f *fakeSecretsAdapter) Rotate(ctx context.Context, req openbao.SecretRotateRequest) (openbao.SecretGetResponse, error) {
	return f.rotateFn(ctx, req)
}

// Get の正常系: adapter から取得した値が proto 応答に詰められる。
func TestSecretHandler_Get_OK(t *testing.T) {
	a := &fakeSecretsAdapter{
		getFn: func(_ context.Context, req openbao.SecretGetRequest) (openbao.SecretGetResponse, error) {
			if req.Name != "db/master" || req.TenantID != "tenant-A" {
				t.Fatalf("args mismatch: %+v", req)
			}
			return openbao.SecretGetResponse{
				Values:  map[string]string{"password": "p", "username": "u"},
				Version: 7,
			}, nil
		},
	}
	h := &secretHandler{deps: Deps{SecretsAdapter: a}}
	resp, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{
		Name:    "db/master",
		Context: &commonv1.TenantContext{TenantId: "tenant-A"},
	})
	if err != nil {
		t.Fatalf("Get error: %v", err)
	}
	if resp.GetVersion() != 7 {
		t.Fatalf("version mismatch: %d", resp.GetVersion())
	}
	if resp.GetValues()["password"] != "p" {
		t.Fatalf("values mismatch")
	}
}

// adapter が ErrSecretNotFound を返した時 NotFound に翻訳される。
func TestSecretHandler_Get_NotFound(t *testing.T) {
	a := &fakeSecretsAdapter{
		getFn: func(_ context.Context, _ openbao.SecretGetRequest) (openbao.SecretGetResponse, error) {
			return openbao.SecretGetResponse{}, openbao.ErrSecretNotFound
		},
	}
	h := &secretHandler{deps: Deps{SecretsAdapter: a}}
	// NFR-E-AC-003: TenantContext は必須。
	_, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{Name: "x", Context: &commonv1.TenantContext{TenantId: "tenant-A"}})
	if got := status.Code(err); got != codes.NotFound {
		t.Fatalf("status: got %v want NotFound", got)
	}
}

// adapter エラーが Internal に翻訳される。
func TestSecretHandler_Get_AdapterError(t *testing.T) {
	a := &fakeSecretsAdapter{
		getFn: func(_ context.Context, _ openbao.SecretGetRequest) (openbao.SecretGetResponse, error) {
			return openbao.SecretGetResponse{}, errors.New("openbao 500")
		},
	}
	h := &secretHandler{deps: Deps{SecretsAdapter: a}}
	// NFR-E-AC-003: TenantContext は必須。
	_, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{Name: "x", Context: &commonv1.TenantContext{TenantId: "tenant-A"}})
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status: got %v want Internal", got)
	}
}

// Version 指定時に adapter req.Version に伝搬する。
func TestSecretHandler_Get_WithVersion(t *testing.T) {
	var observed int
	a := &fakeSecretsAdapter{
		getFn: func(_ context.Context, req openbao.SecretGetRequest) (openbao.SecretGetResponse, error) {
			observed = req.Version
			return openbao.SecretGetResponse{Values: map[string]string{}, Version: int32(req.Version)}, nil
		},
	}
	h := &secretHandler{deps: Deps{SecretsAdapter: a}}
	v := int32(3)
	// NFR-E-AC-003: TenantContext は必須。
	_, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{
		Name:    "x",
		Version: &v,
		Context: &commonv1.TenantContext{TenantId: "tenant-A"},
	})
	if err != nil {
		t.Fatalf("Get error: %v", err)
	}
	if observed != 3 {
		t.Fatalf("version not propagated: got %d", observed)
	}
}

// Rotate の正常系: 新バージョンが返る（NFR-E-AC-003 で TenantContext 必須）。
func TestSecretHandler_Rotate_OK(t *testing.T) {
	a := &fakeSecretsAdapter{
		rotateFn: func(_ context.Context, _ openbao.SecretRotateRequest) (openbao.SecretGetResponse, error) {
			return openbao.SecretGetResponse{Version: 8}, nil
		},
	}
	h := &secretHandler{deps: Deps{SecretsAdapter: a}}
	// TenantContext は NFR-E-AC-003 のため必須。
	resp, err := h.Rotate(context.Background(), &secretsv1.RotateSecretRequest{
		Name:    "db/master",
		Context: &commonv1.TenantContext{TenantId: "tenant-A"},
	})
	if err != nil {
		t.Fatalf("Rotate error: %v", err)
	}
	if resp.GetNewVersion() != 8 {
		t.Fatalf("new_version mismatch: %d", resp.GetNewVersion())
	}
}

// in-process gRPC で Get round-trip が動くことを検証する。
func TestSecretsService_Get_OverGRPC(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	a := &fakeSecretsAdapter{
		getFn: func(_ context.Context, _ openbao.SecretGetRequest) (openbao.SecretGetResponse, error) {
			return openbao.SecretGetResponse{Values: map[string]string{"k": "v"}, Version: 1}, nil
		},
	}
	srv := grpc.NewServer()
	Register(Deps{SecretsAdapter: a})(srv)
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}
	conn, err := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("grpc.NewClient: %v", err)
	}
	defer conn.Close()
	client := secretsv1.NewSecretsServiceClient(conn)
	// NFR-E-AC-003: TenantContext は必須。
	resp, err := client.Get(context.Background(), &secretsv1.GetSecretRequest{
		Name:    "k",
		Context: &commonv1.TenantContext{TenantId: "tenant-A"},
	})
	if err != nil {
		t.Fatalf("Get over gRPC: %v", err)
	}
	if resp.GetValues()["k"] != "v" {
		t.Fatalf("value mismatch: %v", resp.GetValues())
	}
}

// NFR-E-AC-003: tenant_id 空 / nil context は InvalidArgument で弾かれる。
// Get / Rotate のテナント検証回帰テスト。
func TestSecretHandler_TenantValidation(t *testing.T) {
	// adapter は呼ばれない想定（handler 側で短絡）。
	a := &fakeSecretsAdapter{
		getFn: func(_ context.Context, _ openbao.SecretGetRequest) (openbao.SecretGetResponse, error) {
			t.Fatalf("adapter should not be called when tenant_id is empty")
			return openbao.SecretGetResponse{}, nil
		},
		rotateFn: func(_ context.Context, _ openbao.SecretRotateRequest) (openbao.SecretGetResponse, error) {
			t.Fatalf("adapter should not be called when tenant_id is empty")
			return openbao.SecretGetResponse{}, nil
		},
	}
	h := &secretHandler{deps: Deps{SecretsAdapter: a}}

	// Get: nil context で InvalidArgument。
	if _, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{Name: "x"}); status.Code(err) != codes.InvalidArgument {
		t.Fatalf("Get nil context: want InvalidArgument got %v", err)
	}
	// Get: 空 tenant_id で InvalidArgument。
	if _, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{Name: "x", Context: &commonv1.TenantContext{}}); status.Code(err) != codes.InvalidArgument {
		t.Fatalf("Get empty tenant: want InvalidArgument got %v", err)
	}
	// Get: 空 name で InvalidArgument（tenant 有効でも name 空は不正）。
	if _, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{Context: &commonv1.TenantContext{TenantId: "tenant-A"}}); status.Code(err) != codes.InvalidArgument {
		t.Fatalf("Get empty name: want InvalidArgument got %v", err)
	}
	// Rotate: nil context で InvalidArgument。
	if _, err := h.Rotate(context.Background(), &secretsv1.RotateSecretRequest{Name: "x"}); status.Code(err) != codes.InvalidArgument {
		t.Fatalf("Rotate nil context: want InvalidArgument got %v", err)
	}
	// Rotate: 空 tenant_id で InvalidArgument。
	if _, err := h.Rotate(context.Background(), &secretsv1.RotateSecretRequest{Name: "x", Context: &commonv1.TenantContext{}}); status.Code(err) != codes.InvalidArgument {
		t.Fatalf("Rotate empty tenant: want InvalidArgument got %v", err)
	}
	// Rotate: 空 name で InvalidArgument。
	if _, err := h.Rotate(context.Background(), &secretsv1.RotateSecretRequest{Context: &commonv1.TenantContext{TenantId: "tenant-A"}}); status.Code(err) != codes.InvalidArgument {
		t.Fatalf("Rotate empty name: want InvalidArgument got %v", err)
	}
}
