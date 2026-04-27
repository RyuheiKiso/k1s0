// BFF サービス用の OTel 初期化共通実装（tier2 と同パターンの no-op fallback）。

// Package otel は tier3 BFF 用の OpenTelemetry 初期化ヘルパ。
package otel

// 標準 import。
import (
	"context"
	"errors"
	"os"
	"sync"
	"time"
)

// Config は OTel 初期化の最小設定。
type Config struct {
	ServiceName     string
	ServiceVersion  string
	Environment     string
	OTLPEndpoint    string
	ShutdownTimeout time.Duration
}

// ShutdownFunc は Init が返す shutdown ハンドル。
type ShutdownFunc func(ctx context.Context) error

// noopShutdown は OTLPEndpoint 未設定時の no-op。
func noopShutdown(_ context.Context) error { return nil }

func (c *Config) applyDefaults() {
	if c.ShutdownTimeout <= 0 {
		c.ShutdownTimeout = 5 * time.Second
	}
	if c.ServiceName == "" {
		c.ServiceName = "unknown-bff"
	}
}

// Init は SDK 初期化を行い、shutdown ハンドルを返す。
//
// リリース時点 は OTLPEndpoint が空なら no-op、設定有なら multi-call safe な shutdown のみ提供。
// 実 SDK 統合は tier3 BFF 観測性整備（plan 04-02）で追加する。
func Init(ctx context.Context, cfg Config) (ShutdownFunc, error) {
	cfg.applyDefaults()
	if endpoint := os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT"); endpoint != "" {
		cfg.OTLPEndpoint = endpoint
	}
	if cfg.OTLPEndpoint == "" {
		return noopShutdown, nil
	}
	once := &sync.Once{}
	return func(shutdownCtx context.Context) error {
		var firstErr error
		once.Do(func() {
			_, cancel := context.WithTimeout(shutdownCtx, cfg.ShutdownTimeout)
			defer cancel()
			firstErr = errors.Join(firstErr)
		})
		return firstErr
	}, nil
}
