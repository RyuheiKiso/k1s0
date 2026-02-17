package telemetry

import (
	"context"
	"log/slog"
	"net/http"
	"time"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
)

// HTTPMiddleware は HTTP リクエストの分散トレーシングと構造化ログを提供するミドルウェアである。
// リクエストごとにスパンを生成し、メソッド・パス・ステータスコード・レイテンシをログに記録する。
func HTTPMiddleware(logger *slog.Logger) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ctx := r.Context()
			tracer := otel.Tracer("k1s0-http")

			ctx, span := tracer.Start(ctx, r.Method+" "+r.URL.Path,
				trace.WithAttributes(
					attribute.String("http.method", r.Method),
					attribute.String("http.url", r.URL.String()),
				),
			)
			defer span.End()

			rw := &responseWriter{ResponseWriter: w, statusCode: http.StatusOK}
			start := time.Now()

			next.ServeHTTP(rw, r.WithContext(ctx))

			duration := time.Since(start)
			span.SetAttributes(
				attribute.Int("http.status_code", rw.statusCode),
			)

			LogWithTrace(ctx, logger).Info("Request completed",
				slog.String("method", r.Method),
				slog.String("path", r.URL.Path),
				slog.Int("status", rw.statusCode),
				slog.Duration("duration", duration),
			)
		})
	}
}

// GRPCUnaryInterceptor は gRPC Unary RPC のトレーシングとログを提供するインターセプタ関数を返す。
func GRPCUnaryInterceptor(logger *slog.Logger) func(
	ctx context.Context,
	method string,
	req, reply interface{},
	invoker func(ctx context.Context, method string, req, reply interface{}) error,
) error {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		invoker func(ctx context.Context, method string, req, reply interface{}) error,
	) error {
		tracer := otel.Tracer("k1s0-grpc")
		ctx, span := tracer.Start(ctx, method,
			trace.WithAttributes(
				attribute.String("rpc.method", method),
			),
		)
		defer span.End()

		start := time.Now()
		err := invoker(ctx, method, req, reply)
		duration := time.Since(start)

		l := LogWithTrace(ctx, logger)
		if err != nil {
			l.Error("gRPC call failed",
				slog.String("method", method),
				slog.Duration("duration", duration),
				slog.String("error", err.Error()),
			)
		} else {
			l.Info("gRPC call completed",
				slog.String("method", method),
				slog.Duration("duration", duration),
			)
		}

		return err
	}
}

// responseWriter は HTTP ステータスコードをキャプチャするためのラッパーである。
type responseWriter struct {
	http.ResponseWriter
	statusCode int
}

func (rw *responseWriter) WriteHeader(code int) {
	rw.statusCode = code
	rw.ResponseWriter.WriteHeader(code)
}
