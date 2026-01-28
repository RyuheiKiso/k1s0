package middleware

import (
	"context"

	k1s0auth "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-auth"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// GRPCInterceptor creates gRPC authentication interceptors.
type GRPCInterceptor struct {
	validator    k1s0auth.Validator
	skipper      GRPCSkipper
	audience     string
	tokenHeader  string
	errorHandler GRPCErrorHandler
}

// GRPCSkipper determines if authentication should be skipped for a method.
type GRPCSkipper func(ctx context.Context, method string) bool

// GRPCErrorHandler handles authentication errors for gRPC.
type GRPCErrorHandler func(err error) error

// GRPCInterceptorOption configures the gRPC interceptor.
type GRPCInterceptorOption func(*GRPCInterceptor)

// WithGRPCSkipper sets a skipper function.
func WithGRPCSkipper(skipper GRPCSkipper) GRPCInterceptorOption {
	return func(i *GRPCInterceptor) {
		i.skipper = skipper
	}
}

// WithGRPCAudience sets the required audience.
func WithGRPCAudience(audience string) GRPCInterceptorOption {
	return func(i *GRPCInterceptor) {
		i.audience = audience
	}
}

// WithTokenHeader sets the metadata key for the token.
// Default is "authorization".
func WithTokenHeader(header string) GRPCInterceptorOption {
	return func(i *GRPCInterceptor) {
		i.tokenHeader = header
	}
}

// WithGRPCErrorHandler sets a custom error handler.
func WithGRPCErrorHandler(handler GRPCErrorHandler) GRPCInterceptorOption {
	return func(i *GRPCInterceptor) {
		i.errorHandler = handler
	}
}

// NewGRPCInterceptor creates a new gRPC authentication interceptor.
//
// Example:
//
//	validator := k1s0auth.NewJWTValidator(config)
//	authInterceptor := middleware.NewGRPCInterceptor(validator)
//
//	server := grpc.NewServer(
//	    grpc.UnaryInterceptor(authInterceptor.Unary()),
//	    grpc.StreamInterceptor(authInterceptor.Stream()),
//	)
func NewGRPCInterceptor(validator k1s0auth.Validator, opts ...GRPCInterceptorOption) *GRPCInterceptor {
	i := &GRPCInterceptor{
		validator:    validator,
		tokenHeader:  "authorization",
		errorHandler: defaultGRPCErrorHandler,
	}

	for _, opt := range opts {
		opt(i)
	}

	return i
}

// Unary returns the unary interceptor.
func (i *GRPCInterceptor) Unary() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		// Check skipper
		if i.skipper != nil && i.skipper(ctx, info.FullMethod) {
			return handler(ctx, req)
		}

		// Authenticate
		newCtx, err := i.authenticate(ctx)
		if err != nil {
			return nil, i.errorHandler(err)
		}

		return handler(newCtx, req)
	}
}

// Stream returns the stream interceptor.
func (i *GRPCInterceptor) Stream() grpc.StreamServerInterceptor {
	return func(
		srv interface{},
		ss grpc.ServerStream,
		info *grpc.StreamServerInfo,
		handler grpc.StreamHandler,
	) error {
		ctx := ss.Context()

		// Check skipper
		if i.skipper != nil && i.skipper(ctx, info.FullMethod) {
			return handler(srv, ss)
		}

		// Authenticate
		newCtx, err := i.authenticate(ctx)
		if err != nil {
			return i.errorHandler(err)
		}

		// Wrap stream with new context
		wrapped := &wrappedServerStream{ServerStream: ss, ctx: newCtx}
		return handler(srv, wrapped)
	}
}

// authenticate extracts and validates the token from context.
func (i *GRPCInterceptor) authenticate(ctx context.Context) (context.Context, error) {
	// Extract token from metadata
	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return nil, k1s0auth.ErrNotAuthenticated
	}

	values := md.Get(i.tokenHeader)
	if len(values) == 0 {
		return nil, k1s0auth.ErrNotAuthenticated
	}

	token, err := k1s0auth.ExtractBearerToken(values[0])
	if err != nil {
		return nil, err
	}

	// Validate token
	var claims *k1s0auth.Claims
	if i.audience != "" {
		claims, err = i.validator.ValidateWithAudience(token, i.audience)
	} else {
		claims, err = i.validator.Validate(token)
	}
	if err != nil {
		return nil, err
	}

	// Create principal and add to context
	principal := k1s0auth.NewPrincipal(claims)
	ctx = k1s0auth.ContextWithPrincipal(ctx, principal)
	ctx = k1s0auth.ContextWithToken(ctx, token)

	return ctx, nil
}

// defaultGRPCErrorHandler converts authentication errors to gRPC status errors.
func defaultGRPCErrorHandler(err error) error {
	switch err {
	case k1s0auth.ErrNotAuthenticated:
		return status.Error(codes.Unauthenticated, "authentication required")
	case k1s0auth.ErrTokenExpired:
		return status.Error(codes.Unauthenticated, "token expired")
	case k1s0auth.ErrTokenMalformed:
		return status.Error(codes.Unauthenticated, "invalid token format")
	case k1s0auth.ErrInvalidSignature:
		return status.Error(codes.Unauthenticated, "invalid token signature")
	case k1s0auth.ErrInvalidIssuer:
		return status.Error(codes.Unauthenticated, "invalid token issuer")
	case k1s0auth.ErrInvalidAudience:
		return status.Error(codes.Unauthenticated, "invalid token audience")
	default:
		return status.Error(codes.Unauthenticated, err.Error())
	}
}

// wrappedServerStream wraps a server stream with a custom context.
type wrappedServerStream struct {
	grpc.ServerStream
	ctx context.Context
}

// Context returns the wrapped context.
func (w *wrappedServerStream) Context() context.Context {
	return w.ctx
}

// SkipMethods returns a skipper that skips the given methods.
func SkipMethods(methods ...string) GRPCSkipper {
	methodSet := make(map[string]bool)
	for _, method := range methods {
		methodSet[method] = true
	}
	return func(ctx context.Context, method string) bool {
		return methodSet[method]
	}
}

// SkipReflection returns a skipper that skips gRPC reflection methods.
func SkipReflection() GRPCSkipper {
	return func(ctx context.Context, method string) bool {
		return method == "/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo" ||
			method == "/grpc.reflection.v1.ServerReflection/ServerReflectionInfo"
	}
}

// SkipHealthCheck returns a skipper that skips gRPC health check methods.
func SkipHealthCheck() GRPCSkipper {
	return func(ctx context.Context, method string) bool {
		return method == "/grpc.health.v1.Health/Check" ||
			method == "/grpc.health.v1.Health/Watch"
	}
}

// CombineSkippers combines multiple skippers with OR logic.
func CombineSkippers(skippers ...GRPCSkipper) GRPCSkipper {
	return func(ctx context.Context, method string) bool {
		for _, skipper := range skippers {
			if skipper(ctx, method) {
				return true
			}
		}
		return false
	}
}

// RequireRoleInterceptor creates an interceptor that requires a specific role.
func RequireRoleInterceptor(role string) grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		principal := k1s0auth.PrincipalFromContext(ctx)
		if principal == nil {
			return nil, status.Error(codes.Unauthenticated, "authentication required")
		}

		if !principal.HasRole(role) {
			return nil, status.Error(codes.PermissionDenied, "insufficient permissions")
		}

		return handler(ctx, req)
	}
}

// RequirePermissionInterceptor creates an interceptor that requires a specific permission.
func RequirePermissionInterceptor(permission string) grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		principal := k1s0auth.PrincipalFromContext(ctx)
		if principal == nil {
			return nil, status.Error(codes.Unauthenticated, "authentication required")
		}

		if !principal.HasPermission(permission) {
			return nil, status.Error(codes.PermissionDenied, "insufficient permissions")
		}

		return handler(ctx, req)
	}
}
