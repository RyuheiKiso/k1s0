package middleware

import (
	"net/http"

	k1s0auth "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-auth"
)

// HTTPMiddleware creates HTTP authentication middleware.
type HTTPMiddleware struct {
	validator    k1s0auth.Validator
	errorHandler ErrorHandler
	skipper      Skipper
	audience     string
}

// ErrorHandler handles authentication errors.
type ErrorHandler func(w http.ResponseWriter, r *http.Request, err error)

// Skipper determines if authentication should be skipped for a request.
type Skipper func(r *http.Request) bool

// HTTPMiddlewareOption configures the HTTP middleware.
type HTTPMiddlewareOption func(*HTTPMiddleware)

// WithErrorHandler sets a custom error handler.
func WithErrorHandler(handler ErrorHandler) HTTPMiddlewareOption {
	return func(m *HTTPMiddleware) {
		m.errorHandler = handler
	}
}

// WithSkipper sets a skipper function.
func WithSkipper(skipper Skipper) HTTPMiddlewareOption {
	return func(m *HTTPMiddleware) {
		m.skipper = skipper
	}
}

// WithAudience sets the required audience.
func WithAudience(audience string) HTTPMiddlewareOption {
	return func(m *HTTPMiddleware) {
		m.audience = audience
	}
}

// NewHTTPMiddleware creates a new HTTP authentication middleware.
//
// Example:
//
//	validator := k1s0auth.NewJWTValidator(config)
//	authMiddleware := middleware.NewHTTPMiddleware(validator)
//
//	http.Handle("/api/", authMiddleware.Wrap(apiHandler))
func NewHTTPMiddleware(validator k1s0auth.Validator, opts ...HTTPMiddlewareOption) *HTTPMiddleware {
	m := &HTTPMiddleware{
		validator:    validator,
		errorHandler: defaultErrorHandler,
	}

	for _, opt := range opts {
		opt(m)
	}

	return m
}

// Wrap wraps an http.Handler with authentication.
func (m *HTTPMiddleware) Wrap(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Check skipper
		if m.skipper != nil && m.skipper(r) {
			next.ServeHTTP(w, r)
			return
		}

		// Extract token
		token, err := k1s0auth.ExtractBearerToken(r.Header.Get("Authorization"))
		if err != nil {
			m.errorHandler(w, r, err)
			return
		}

		// Validate token
		var claims *k1s0auth.Claims
		if m.audience != "" {
			claims, err = m.validator.ValidateWithAudience(token, m.audience)
		} else {
			claims, err = m.validator.Validate(token)
		}
		if err != nil {
			m.errorHandler(w, r, err)
			return
		}

		// Create principal and add to context
		principal := k1s0auth.NewPrincipal(claims)
		ctx := k1s0auth.ContextWithPrincipal(r.Context(), principal)
		ctx = k1s0auth.ContextWithToken(ctx, token)

		// Call next handler
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

// WrapFunc wraps an http.HandlerFunc with authentication.
func (m *HTTPMiddleware) WrapFunc(next http.HandlerFunc) http.HandlerFunc {
	return m.Wrap(next).ServeHTTP
}

// defaultErrorHandler is the default error handler.
func defaultErrorHandler(w http.ResponseWriter, r *http.Request, err error) {
	status := http.StatusUnauthorized
	message := "Unauthorized"

	switch err {
	case k1s0auth.ErrNotAuthenticated:
		message = "Authentication required"
	case k1s0auth.ErrTokenExpired:
		message = "Token expired"
	case k1s0auth.ErrTokenMalformed:
		message = "Invalid token format"
	case k1s0auth.ErrInvalidSignature:
		message = "Invalid token signature"
	}

	http.Error(w, message, status)
}

// RequireRole creates middleware that requires a specific role.
func RequireRole(role string) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			principal := k1s0auth.PrincipalFromContext(r.Context())
			if principal == nil {
				http.Error(w, "Unauthorized", http.StatusUnauthorized)
				return
			}

			if !principal.HasRole(role) {
				http.Error(w, "Forbidden", http.StatusForbidden)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// RequireAnyRole creates middleware that requires any of the specified roles.
func RequireAnyRole(roles ...string) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			principal := k1s0auth.PrincipalFromContext(r.Context())
			if principal == nil {
				http.Error(w, "Unauthorized", http.StatusUnauthorized)
				return
			}

			for _, role := range roles {
				if principal.HasRole(role) {
					next.ServeHTTP(w, r)
					return
				}
			}

			http.Error(w, "Forbidden", http.StatusForbidden)
		})
	}
}

// RequirePermission creates middleware that requires a specific permission.
func RequirePermission(permission string) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			principal := k1s0auth.PrincipalFromContext(r.Context())
			if principal == nil {
				http.Error(w, "Unauthorized", http.StatusUnauthorized)
				return
			}

			if !principal.HasPermission(permission) {
				http.Error(w, "Forbidden", http.StatusForbidden)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// SkipPaths returns a skipper that skips the given paths.
func SkipPaths(paths ...string) Skipper {
	pathSet := make(map[string]bool)
	for _, path := range paths {
		pathSet[path] = true
	}
	return func(r *http.Request) bool {
		return pathSet[r.URL.Path]
	}
}

// SkipHTTPMethods returns a skipper that skips the given HTTP methods.
func SkipHTTPMethods(methods ...string) Skipper {
	methodSet := make(map[string]bool)
	for _, method := range methods {
		methodSet[method] = true
	}
	return func(r *http.Request) bool {
		return methodSet[r.Method]
	}
}
