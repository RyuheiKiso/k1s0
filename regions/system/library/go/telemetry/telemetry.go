package telemetry

import (
	"context"
	"log/slog"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
)

// TelemetryConfig は telemetry ライブラリの初期化設定を保持する。
type TelemetryConfig struct {
	ServiceName   string
	Version       string
	Tier          string
	Environment   string
	TraceEndpoint string
	SampleRate    float64
	LogLevel      string
	LogFormat     string
}

// Provider は TracerProvider と Logger を保持し、シャットダウンを管理する。
type Provider struct {
	tracerProvider *sdktrace.TracerProvider
	logger         *slog.Logger
}

// IsDevEnvironment は環境名が開発環境（development / dev / local）かを判定する。
// 本番環境との区別に使用し、開発環境のみ WithInsecure() を有効にする。
func IsDevEnvironment(env string) bool {
	switch env {
	case "development", "dev", "local":
		return true
	default:
		return false
	}
}

// InitTelemetry は OpenTelemetry TracerProvider と構造化ロガーを初期化する。
// 開発環境（development / dev / local）のみ WithInsecure() を使用し、
// 本番環境では TLS 通信を強制して平文送信を防ぐ。
func InitTelemetry(ctx context.Context, cfg TelemetryConfig) (*Provider, error) {
	var tp *sdktrace.TracerProvider

	if cfg.TraceEndpoint != "" {
		// 開発環境のみ TLS を無効化し、本番では TLS 接続を使用する
		opts := []otlptracegrpc.Option{
			otlptracegrpc.WithEndpoint(cfg.TraceEndpoint),
		}
		if IsDevEnvironment(cfg.Environment) {
			opts = append(opts, otlptracegrpc.WithInsecure())
		}
		exporter, err := otlptracegrpc.New(ctx, opts...)
		if err != nil {
			return nil, err
		}
		tp = sdktrace.NewTracerProvider(
			sdktrace.WithBatcher(exporter),
			sdktrace.WithSampler(sdktrace.TraceIDRatioBased(cfg.SampleRate)),
			sdktrace.WithResource(resource.NewWithAttributes(
				semconv.SchemaURL,
				semconv.ServiceNameKey.String(cfg.ServiceName),
				semconv.ServiceVersionKey.String(cfg.Version),
			)),
		)
		otel.SetTracerProvider(tp)
	}

	logger := NewLogger(cfg)

	return &Provider{tracerProvider: tp, logger: logger}, nil
}

// Shutdown は TracerProvider をシャットダウンする。
func (p *Provider) Shutdown(ctx context.Context) error {
	if p.tracerProvider != nil {
		return p.tracerProvider.Shutdown(ctx)
	}
	return nil
}

// Logger は構造化ロガーを返す。
func (p *Provider) Logger() *slog.Logger {
	return p.logger
}
