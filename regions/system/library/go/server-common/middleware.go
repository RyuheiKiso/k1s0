package servercommon

import (
	"log/slog"
	"net/http"
	"runtime/debug"

	"github.com/google/uuid"
)

// MiddlewareFunc は HTTP ミドルウェアの型エイリアス。
type MiddlewareFunc func(http.Handler) http.Handler

// Chain は複数のミドルウェアをチェーンする。
// 引数の順序で適用される（最初が最外側）。
func Chain(middlewares ...MiddlewareFunc) MiddlewareFunc {
	return func(next http.Handler) http.Handler {
		for i := len(middlewares) - 1; i >= 0; i-- {
			next = middlewares[i](next)
		}
		return next
	}
}

// RequestIDMiddleware はリクエストに X-Request-ID ヘッダーを付与するミドルウェア。
// 既に X-Request-ID が存在する場合はそのまま使用する。
func RequestIDMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		requestID := r.Header.Get("X-Request-ID")
		if requestID == "" {
			requestID = generateID()
		}
		w.Header().Set("X-Request-ID", requestID)
		next.ServeHTTP(w, r)
	})
}

// CORSMiddleware は CORS ヘッダーを付与するミドルウェア。
// OPTIONS プリフライトリクエストは即座に 204 を返す。
func CORSMiddleware(allowedOrigins []string) MiddlewareFunc {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			origin := r.Header.Get("Origin")
			for _, allowed := range allowedOrigins {
				if allowed == "*" || allowed == origin {
					w.Header().Set("Access-Control-Allow-Origin", origin)
					break
				}
			}
			w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
			w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Request-ID")

			if r.Method == http.MethodOptions {
				w.WriteHeader(http.StatusNoContent)
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}

// RecoveryMiddleware はパニックをリカバリして 500 を返すミドルウェア。
// パニック値とスタックトレースを構造化ログに出力し、障害調査を容易にする。
func RecoveryMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			if rec := recover(); rec != nil {
				// パニックの詳細とスタックトレースを構造化ログに出力する
				requestID := r.Header.Get("X-Request-ID")
				slog.Error("handler panic recovered",
					"panic", rec,
					"stack", string(debug.Stack()),
					"request_id", requestID,
					"method", r.Method,
					"path", r.URL.Path,
				)
				http.Error(w, "Internal Server Error", http.StatusInternalServerError)
			}
		}()
		next.ServeHTTP(w, r)
	})
}

// generateID は UUID v4 を使用して一意な ID を生成する。
func generateID() string {
	return uuid.New().String()
}
