package auth

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
func (m *mockServerStream) SendMsg(interface{}) error    { return nil }
func (m *mockServerStream) RecvMsg(interface{}) error    { return nil }

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

func TestUnaryInterceptor_NoMetadata(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)

	_, err := interceptor(context.Background(), nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req interface{}) (interface{}, error) {
			return "ok", nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

func TestUnaryInterceptor_NoAuthorizationHeader(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("x-request-id", "req-1")

	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req interface{}) (interface{}, error) {
			return "ok", nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

func TestUnaryInterceptor_NoBearerPrefix(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", tokenStr) // "Bearer " プレフィックスなし

	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req interface{}) (interface{}, error) {
			return "ok", nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

func TestUnaryInterceptor_InvalidToken(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "Bearer invalid-token")

	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req interface{}) (interface{}, error) {
			return "ok", nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

func TestUnaryInterceptor_ValidToken_ClaimsInContext(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := UnaryServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "Bearer "+tokenStr)

	var capturedClaims *Claims
	_, err := interceptor(ctx, nil, &grpc.UnaryServerInfo{},
		func(ctx context.Context, req interface{}) (interface{}, error) {
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

func TestStreamInterceptor_NoMetadata(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := StreamServerInterceptor(verifier)
	ss := &mockServerStream{ctx: context.Background()}

	err := interceptor(nil, ss, &grpc.StreamServerInfo{},
		func(srv interface{}, stream grpc.ServerStream) error {
			return nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

func TestStreamInterceptor_ValidToken_ClaimsInContext(t *testing.T) {
	verifier, tokenStr := makeVerifier(t)
	interceptor := StreamServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "Bearer "+tokenStr)
	ss := &mockServerStream{ctx: ctx}

	var capturedClaims *Claims
	err := interceptor(nil, ss, &grpc.StreamServerInfo{},
		func(srv interface{}, stream grpc.ServerStream) error {
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

func TestStreamInterceptor_InvalidToken(t *testing.T) {
	verifier, _ := makeVerifier(t)
	interceptor := StreamServerInterceptor(verifier)
	ctx := ctxWithMD("authorization", "Bearer invalid-token")
	ss := &mockServerStream{ctx: ctx}

	err := interceptor(nil, ss, &grpc.StreamServerInfo{},
		func(srv interface{}, stream grpc.ServerStream) error {
			return nil
		},
	)

	require.Error(t, err)
	st, _ := status.FromError(err)
	assert.Equal(t, codes.Unauthenticated, st.Code())
}

// --- wrappedServerStream テスト ---

func TestWrappedServerStream_Context(t *testing.T) {
	originalCtx := context.Background()
	newCtx := context.WithValue(originalCtx, ClaimsContextKey, &Claims{Sub: "test"})
	ss := &mockServerStream{ctx: originalCtx}
	wrapped := &wrappedServerStream{ServerStream: ss, ctx: newCtx}

	assert.Equal(t, newCtx, wrapped.Context())
	assert.NotEqual(t, originalCtx, wrapped.Context())
}
