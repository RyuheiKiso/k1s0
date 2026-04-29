// 本ファイルは tier1 facade の interceptor チェイン込みの end-to-end 結合テスト。
//
// 検証する組み合わせ（runtime.go の本番構成と同じ chain）:
//   1. AuthInterceptor（HMAC mode）: JWT 検証 + L1 テナント上書き
//   2. ObservabilityInterceptor: OTel 1 span 発行 + RED メトリクス
//   3. StateService handler: requireTenantID で TenantContext を必須化
//   4. Dapr StateAdapter: L2 物理キー prefix（<tenant_id>/<key>）
//   5. inMemoryDapr backend: 3 階層 map（tenant / store / key）でテナント分離
//
// 検証観点:
//   - 認証 → handler → adapter → backend の全層が連動して期待通り動作
//   - JWT.tenant_id と TenantContext.tenant_id 不一致は PermissionDenied で adapter まで到達しない
//   - JWT 不在は Unauthenticated で handler まで到達しない
//   - L2 prefix 適用後、別テナントが同一論理キーで Set しても物理隔離される
//   - 共通 interceptor が StateService の挙動を破壊しない（既存テストと同等の挙動）

package state

import (
	"context"
	"net"
	"testing"
	"time"

	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
)

// startServerWithInterceptors は runtime.go と同じ interceptor chain で StateService を起動する。
// 認証は HMAC mode で起動し、test ごとに JWT を発行する。
func startServerWithInterceptors(t *testing.T, hmacSecret []byte) (statev1.StateServiceClient, func()) {
	t.Helper()
	lis := bufconn.Listen(1024 * 1024)
	client := dapr.NewClientWithInMemoryBackends()
	deps := NewDepsFromClient(client)

	// runtime.go と同じ chain（AuthInterceptor → ObservabilityInterceptor）。
	authCfg := common.AuthInterceptorConfig{
		Mode:       common.AuthModeHMAC,
		HMACSecret: hmacSecret,
		// SkipMethods は LoadAuthConfigFromEnv の既定値と等価にする。
		SkipMethods: map[string]bool{
			"/grpc.health.v1.Health/Check":                  true,
			"/k1s0.tier1.health.v1.HealthService/Liveness":  true,
			"/k1s0.tier1.health.v1.HealthService/Readiness": true,
		},
	}
	srv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			common.AuthInterceptor(authCfg),
			common.ObservabilityInterceptor(),
		),
	)
	Register(deps)(srv)
	go func() { _ = srv.Serve(lis) }()

	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	conn, err := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("grpc.NewClient: %v", err)
	}
	cleanup := func() {
		_ = conn.Close()
		srv.Stop()
		_ = lis.Close()
	}
	return statev1.NewStateServiceClient(conn), cleanup
}

// helper: HS256 JWT を生成する。
func mintJWT(t *testing.T, secret []byte, tenantID, subject string) string {
	t.Helper()
	signer, err := jose.NewSigner(jose.SigningKey{Algorithm: jose.HS256, Key: secret}, (&jose.SignerOptions{}).WithType("JWT"))
	if err != nil {
		t.Fatalf("signer: %v", err)
	}
	tok, err := jwt.Signed(signer).Claims(common.AuthClaims{
		TenantID: tenantID,
		Subject:  subject,
		Claims: jwt.Claims{
			IssuedAt: jwt.NewNumericDate(time.Now()),
			Expiry:   jwt.NewNumericDate(time.Now().Add(5 * time.Minute)),
		},
	}).Serialize()
	if err != nil {
		t.Fatalf("sign: %v", err)
	}
	return tok
}

// helper: JWT を Authorization metadata で ctx に詰める。
func ctxWithJWT(token string) context.Context {
	return metadata.NewOutgoingContext(context.Background(),
		metadata.Pairs("authorization", "Bearer "+token))
}

// 全層連動: 正規 JWT + 整合 TenantContext で Set → Get が成功する。
func TestFullStack_HappyPath(t *testing.T) {
	secret := []byte("integration-test-secret-32-bytes")
	c, cleanup := startServerWithInterceptors(t, secret)
	defer cleanup()
	ctx := ctxWithJWT(mintJWT(t, secret, "T-alpha", "user-1"))

	if _, err := c.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default", Key: "k1", Data: []byte("v1"), Context: makeTenantCtx("T-alpha"),
	}); err != nil {
		t.Fatalf("Set: %v", err)
	}
	resp, err := c.Get(ctx, &statev1.GetRequest{
		Store: "valkey-default", Key: "k1", Context: makeTenantCtx("T-alpha"),
	})
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if string(resp.GetData()) != "v1" {
		t.Fatalf("data: got %q want %q", resp.GetData(), "v1")
	}
}

// JWT 不在は Unauthenticated で handler / adapter に到達しない。
func TestFullStack_NoJWT_Unauthenticated(t *testing.T) {
	c, cleanup := startServerWithInterceptors(t, []byte("integration-test-secret-32-bytes"))
	defer cleanup()
	_, err := c.Get(context.Background(), &statev1.GetRequest{
		Store: "valkey-default", Key: "k1", Context: makeTenantCtx("T-alpha"),
	})
	if status.Code(err) != grpccodes.Unauthenticated {
		t.Fatalf("expected Unauthenticated, got %v", err)
	}
}

// JWT.tenant_id != TenantContext.tenant_id は PermissionDenied で adapter に到達しない。
func TestFullStack_TenantMismatch_PermissionDenied(t *testing.T) {
	secret := []byte("integration-test-secret-32-bytes")
	c, cleanup := startServerWithInterceptors(t, secret)
	defer cleanup()
	// JWT は T-alpha だが Context は T-bravo を指定する。
	ctx := ctxWithJWT(mintJWT(t, secret, "T-alpha", "user-1"))
	_, err := c.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default", Key: "k1", Data: []byte("v1"), Context: makeTenantCtx("T-bravo"),
	})
	if status.Code(err) != grpccodes.PermissionDenied {
		t.Fatalf("expected PermissionDenied, got %v", err)
	}
}

// L2 物理 prefix が adapter 層で適用され、テナント A の値はテナント B から見えない。
// interceptor chain と adapter が連動して NFR-E-AC-003 を満たすことを確認する。
func TestFullStack_CrossTenantIsolation_L2(t *testing.T) {
	secret := []byte("integration-test-secret-32-bytes")
	c, cleanup := startServerWithInterceptors(t, secret)
	defer cleanup()

	// tenant A が "secret-A" を保存する。
	ctxA := ctxWithJWT(mintJWT(t, secret, "T-alpha", "u1"))
	if _, err := c.Set(ctxA, &statev1.SetRequest{
		Store: "valkey-default", Key: "shared", Data: []byte("secret-A"), Context: makeTenantCtx("T-alpha"),
	}); err != nil {
		t.Fatalf("Set A: %v", err)
	}
	// tenant B が同一論理キーに "secret-B" を保存する。
	ctxB := ctxWithJWT(mintJWT(t, secret, "T-bravo", "u2"))
	if _, err := c.Set(ctxB, &statev1.SetRequest{
		Store: "valkey-default", Key: "shared", Data: []byte("secret-B"), Context: makeTenantCtx("T-bravo"),
	}); err != nil {
		t.Fatalf("Set B: %v", err)
	}

	// tenant A の Get は "secret-A"。
	rA, err := c.Get(ctxA, &statev1.GetRequest{
		Store: "valkey-default", Key: "shared", Context: makeTenantCtx("T-alpha"),
	})
	if err != nil {
		t.Fatalf("Get A: %v", err)
	}
	if string(rA.GetData()) != "secret-A" {
		t.Fatalf("Get A leak: got %q want %q", rA.GetData(), "secret-A")
	}
	// tenant B の Get は "secret-B"（A の値が見えない）。
	rB, err := c.Get(ctxB, &statev1.GetRequest{
		Store: "valkey-default", Key: "shared", Context: makeTenantCtx("T-bravo"),
	})
	if err != nil {
		t.Fatalf("Get B: %v", err)
	}
	if string(rB.GetData()) != "secret-B" {
		t.Fatalf("Get B leak: got %q want %q", rB.GetData(), "secret-B")
	}
}

// SkipMethods（health）は JWT 不在でも素通り（K8s probe 互換）。
// 本テストは StateService 経由ではなく Health 経由で検証するため、ここでは pass-through 確認のみとする。
// （Health のテストは internal/health/health_test.go に既存あり。）

// JWT 期限切れは Unauthenticated。
func TestFullStack_ExpiredJWT_Unauthenticated(t *testing.T) {
	secret := []byte("integration-test-secret-32-bytes")
	c, cleanup := startServerWithInterceptors(t, secret)
	defer cleanup()

	// 1 時間前に期限切れの JWT。
	signer, err := jose.NewSigner(jose.SigningKey{Algorithm: jose.HS256, Key: secret}, (&jose.SignerOptions{}).WithType("JWT"))
	if err != nil {
		t.Fatalf("signer: %v", err)
	}
	tok, err := jwt.Signed(signer).Claims(common.AuthClaims{
		TenantID: "T",
		Claims: jwt.Claims{
			IssuedAt: jwt.NewNumericDate(time.Now().Add(-2 * time.Hour)),
			Expiry:   jwt.NewNumericDate(time.Now().Add(-1 * time.Hour)),
		},
	}).Serialize()
	if err != nil {
		t.Fatalf("sign: %v", err)
	}
	ctx := metadata.NewOutgoingContext(context.Background(),
		metadata.Pairs("authorization", "Bearer "+tok))
	_, err = c.Get(ctx, &statev1.GetRequest{
		Store: "valkey-default", Key: "k1", Context: makeTenantCtx("T"),
	})
	if status.Code(err) != grpccodes.Unauthenticated {
		t.Fatalf("expected Unauthenticated, got %v", err)
	}
}
