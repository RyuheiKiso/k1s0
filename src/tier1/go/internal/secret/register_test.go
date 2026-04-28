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
	getFn     func(ctx context.Context, req openbao.SecretGetRequest) (openbao.SecretGetResponse, error)
	bulkGetFn func(ctx context.Context, names []string, tenantID string) (map[string]openbao.SecretGetResponse, error)
	rotateFn  func(ctx context.Context, req openbao.SecretRotateRequest) (openbao.SecretGetResponse, error)
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
	_, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{Name: "x"})
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
	_, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{Name: "x"})
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status: got %v want Internal", got)
	}
}

// adapter 未注入時は Unimplemented。
func TestSecretHandler_Get_NoAdapter(t *testing.T) {
	h := &secretHandler{deps: Deps{}}
	_, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{Name: "x"})
	if got := status.Code(err); got != codes.Unimplemented {
		t.Fatalf("status: got %v want Unimplemented", got)
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
	_, err := h.Get(context.Background(), &secretsv1.GetSecretRequest{
		Name:    "x",
		Version: &v,
	})
	if err != nil {
		t.Fatalf("Get error: %v", err)
	}
	if observed != 3 {
		t.Fatalf("version not propagated: got %d", observed)
	}
}

// Rotate の正常系: 新バージョンが返る。
func TestSecretHandler_Rotate_OK(t *testing.T) {
	a := &fakeSecretsAdapter{
		rotateFn: func(_ context.Context, _ openbao.SecretRotateRequest) (openbao.SecretGetResponse, error) {
			return openbao.SecretGetResponse{Version: 8}, nil
		},
	}
	h := &secretHandler{deps: Deps{SecretsAdapter: a}}
	resp, err := h.Rotate(context.Background(), &secretsv1.RotateSecretRequest{Name: "db/master"})
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
	resp, err := client.Get(context.Background(), &secretsv1.GetSecretRequest{Name: "k"})
	if err != nil {
		t.Fatalf("Get over gRPC: %v", err)
	}
	if resp.GetValues()["k"] != "v" {
		t.Fatalf("value mismatch: %v", resp.GetValues())
	}
}
