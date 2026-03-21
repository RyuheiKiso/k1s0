package authlib

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// mockServerStream は grpc.ServerStream の最小実装。
type mockServerStream struct {
	ctx context.Context
}

func (m *mockServerStream) SetHeader(metadata.MD) error  { return nil }
func (m *mockServerStream) SendHeader(metadata.MD) error { return nil }
func (m *mockServerStream) SetTrailer(metadata.MD)       {}
func (m *mockServerStream) Context() context.Context     { return m.ctx }
func (m *mockServerStream) SendMsg(any) error    { return nil }
func (m *mockServerStream) RecvMsg(any) error    { return nil }

// ctxWithMD は指定メタデータを含む incoming gRPC コンテキストを生成する。
func ctxWithMD(pairs ...string) context.Context {
	md := metadata.Pairs(pairs...)
	return metadata.NewIncomingContext(context.Background(), md)
}

func makeVerifier(t *testing.T) (*JWKSVerifier, string) {
	t.Helper()
	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)
	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)
	return verifier, tokenStr
}

// --- UnaryServerInterceptor テスト ---

// UnaryServerInterceptor がメタデータのないコンテキストで Unauthenticated エラーを返すことを確認する。
func TestUnaryInterceptor_NoMetadata(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)

	_, err := interceptor(context.Background(), nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req any) (any, error) {
			return "ok", nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

// UnaryServerInterceptor が authorization ヘッダーのないメタデータで Unauthenticated エラーを返すことを確認する。
func TestUnaryInterceptor_NoAuthorizationHeader(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("x-request-id", "req-1")

	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req any) (any, error) {
			return "ok", nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

// UnaryServerInterceptor が Bearer プレフィックスのない authorization ヘッダーで Unauthenticated エラーを返すことを確認する。
func TestUnaryInterceptor_NoBearerPrefix(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", tokenStr) // "Bearer " プレフィックスなし

	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req any) (any, error) {
			return "ok", nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

// UnaryServerInterceptor が無効なトークンで Unauthenticated エラーを返すことを確認する。
func TestUnaryInterceptor_InvalidToken(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "Bearer invalid-token")

	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req any) (any, error) {
			return "ok", nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

// UnaryServerInterceptor が有効なトークンを検証しクレームをコンテキストに設定してハンドラーへ渡すことを確認する。
func TestUnaryInterceptor_ValidToken_ClaimsInContext(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "Bearer "+tokenStr)

	var capturedClaims *Claims
	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req any) (any, error) {
			claims, ok := GetClaimsFromContext(ctx)
			if ok {
				capturedClaims = claims
			}
			return "ok", nil
		},
	)

	require.NoError(t, err)
	require.NotNil(t, capturedClaims)
	assert.Equal(t, "user-uuid-1234", capturedClaims.Sub)
	assert.Equal(t, testIssuer, capturedClaims.Iss)
}

// --- StreamServerInterceptor テスト ---

// StreamServerInterceptor がメタデータのないストリームコンテキストで Unauthenticated エラーを返すことを確認する。
func TestStreamInterceptor_NoMetadata(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := StreamServerInterceptor(verifier)
	ss := &mockServerStream{ctx: context.Background()}

	err := interceptor(nil, ss, &grpc.StreamServerInfo{},
		func(srv any, stream grpc.ServerStream) error {
			return nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

// StreamServerInterceptor が有効なトークンを検証しクレームをストリームコンテキストに設定することを確認する。
func TestStreamInterceptor_ValidToken_ClaimsInContext(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := StreamServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "Bearer "+tokenStr)
	ss := &mockServerStream{ctx: ctx}

	var capturedClaims *Claims
	err := interceptor(nil, ss, &grpc.StreamServerInfo{},
		func(srv any, stream grpc.ServerStream) error {
			claims, ok := GetClaimsFromContext(stream.Context())
			if ok {
				capturedClaims = claims
			}
			return nil
		},
	)

	require.NoError(t, err)
	require.NotNil(t, capturedClaims)
	assert.Equal(t, "user-uuid-1234", capturedClaims.Sub)
}

// StreamServerInterceptor が無効なトークンで Unauthenticated エラーを返すことを確認する。
func TestStreamInterceptor_InvalidToken(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := StreamServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "Bearer invalid-token")
	ss := &mockServerStream{ctx: ctx}

	err := interceptor(nil, ss, &grpc.StreamServerInfo{},
		func(srv any, stream grpc.ServerStream) error {
			return nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

// UnaryServerInterceptor が小文字の "bearer " プレフィックスでも有効なトークンを受け入れることを確認する。
// RFC 7235 は認証スキーム名を case-insensitive と規定している。
func TestUnaryInterceptor_LowercaseBearerPrefix(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "bearer "+tokenStr) // 小文字の "bearer"

	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req any) (any, error) {
			return "ok", nil
		},
	)

	require.NoError(t, err)
}

// UnaryServerInterceptor が大文字の "BEARER " プレフィックスでも有効なトークンを受け入れることを確認する。
func TestUnaryInterceptor_UppercaseBearerPrefix(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "BEARER "+tokenStr) // 大文字の "BEARER"

	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req any) (any, error) {
			return "ok", nil
		},
	)

	require.NoError(t, err)
}

// StreamServerInterceptor が小文字の "bearer " プレフィックスでも有効なトークンを受け入れることを確認する。
func TestStreamInterceptor_LowercaseBearerPrefix(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := StreamServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "bearer "+tokenStr) // 小文字の "bearer"
	ss := &mockServerStream{ctx: ctx}

	err := interceptor(nil, ss, &grpc.StreamServerInfo{},
		func(srv any, stream grpc.ServerStream) error {
			return nil
		},
	)

	require.NoError(t, err)
}

// StreamServerInterceptor が大文字の "BEARER " プレフィックスでも有効なトークンを受け入れることを確認する。
func TestStreamInterceptor_UppercaseBearerPrefix(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := StreamServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "BEARER "+tokenStr) // 大文字の "BEARER"
	ss := &mockServerStream{ctx: ctx}

	err := interceptor(nil, ss, &grpc.StreamServerInfo{},
		func(srv any, stream grpc.ServerStream) error {
			return nil
		},
	)

	require.NoError(t, err)
}

// --- wrappedServerStream テスト ---

// wrappedServerStream の Context メソッドが注入された新しいコンテキストを返すことを確認する。
func TestWrappedServerStream_Context(t *testing.T) {
	originalCtx := context.Background()
	newCtx := context.WithValue(originalCtx, ClaimsContextKey, &Claims{Sub: "test"})
	ss := &mockServerStream{ctx: originalCtx}
	wrapped := &wrappedServerStream{ServerStream: ss, ctx: newCtx}

	assert.Equal(t, newCtx, wrapped.Context())
	assert.NotEqual(t, originalCtx, wrapped.Context())
}
