package auth

import (
	"context"
	"strings"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// UnaryServerInterceptor は gRPC Unary RPC 用の JWT 認証インターセプターを返す。
// Authorization メタデータから Bearer トークンを取得し、JWKS 検証を行う。
// 検証成功時は Claims をコンテキストに格納する。
func UnaryServerInterceptor(verifier *JWKSVerifier) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		claims, err := extractAndVerifyGRPCToken(ctx, verifier)
		if err != nil {
			return nil, err
		}
		ctx = context.WithValue(ctx, ClaimsContextKey, claims)
		return handler(ctx, req)
	}
}

// StreamServerInterceptor は gRPC Streaming RPC 用の JWT 認証インターセプターを返す。
// Authorization メタデータから Bearer トークンを取得し、JWKS 検証を行う。
// 検証成功時は Claims をコンテキストに格納する。
func StreamServerInterceptor(verifier *JWKSVerifier) grpc.StreamServerInterceptor {
	return func(srv interface{}, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
		claims, err := extractAndVerifyGRPCToken(ss.Context(), verifier)
		if err != nil {
			return err
		}
		return handler(srv, &wrappedServerStream{
			ServerStream: ss,
			ctx:          context.WithValue(ss.Context(), ClaimsContextKey, claims),
		})
	}
}

// extractAndVerifyGRPCToken は gRPC メタデータから Bearer トークンを取り出して検証する。
func extractAndVerifyGRPCToken(ctx context.Context, verifier *JWKSVerifier) (*Claims, error) {
	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return nil, status.Error(codes.Unauthenticated, "認証が必要です")
	}

	values := md.Get("authorization")
	if len(values) == 0 {
		return nil, status.Error(codes.Unauthenticated, "認証が必要です")
	}

	raw := values[0]
	trimmed := strings.TrimPrefix(raw, "Bearer ")
	if trimmed == raw {
		return nil, status.Error(codes.Unauthenticated, "Bearer トークンが必要です")
	}
	tokenString := strings.TrimSpace(trimmed)
	if tokenString == "" {
		return nil, status.Error(codes.Unauthenticated, "認証が必要です")
	}

	claims, err := verifier.VerifyToken(ctx, tokenString)
	if err != nil {
		return nil, status.Error(codes.Unauthenticated, "トークンが無効です")
	}
	return claims, nil
}

// wrappedServerStream は Claims を格納したコンテキストを持つ ServerStream のラッパー。
type wrappedServerStream struct {
	grpc.ServerStream
	ctx context.Context
}

func (w *wrappedServerStream) Context() context.Context {
	return w.ctx
}
