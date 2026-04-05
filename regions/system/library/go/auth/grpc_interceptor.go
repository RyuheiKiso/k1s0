package authlib

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
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
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
	return func(srv any, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
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

	// LOW-018 監査対応: 複数の authorization ヘッダーはヘッダーインジェクション攻撃等の
	// 不正アクセスの兆候であるため明示的に拒否する。
	// RFC 7235 では Authorization ヘッダーは1つのみ送信することが規定されている。
	// gRPC メタデータは同一キーに複数値を持てるが、それを許可するとヘッダースマグリング等の
	// リスクが生じるためエラーとして扱う。
	if len(values) > 1 {
		return nil, status.Error(codes.Unauthenticated, "Authorization ヘッダーは1つのみ許可されます")
	}
	raw := values[0]
	// "Bearer " プレフィックスを大文字小文字を区別せずに確認する（RFC 7235 準拠）
	const bearerPrefix = "bearer "
	if len(raw) < len(bearerPrefix) || !strings.EqualFold(raw[:len(bearerPrefix)], bearerPrefix) {
		return nil, status.Error(codes.Unauthenticated, "Bearer トークンが必要です")
	}
	tokenString := strings.TrimSpace(raw[len(bearerPrefix):])
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
