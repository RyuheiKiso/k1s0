// 本ファイルは tier1 Go の OpenTelemetry 共通初期化ユーティリティ。
//
// 設計: docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/05_モジュール依存関係.md
//       （DS-SW-COMP-109: k1s0-otel 共通ライブラリ、tracer / meter / logger / propagator 集約）
//       docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//       （DS-SW-COMP-037: stdout JSON Lines / OTel Collector / Loki 集約）
//       （DS-SW-COMP-038: Metrics Emitter）
// 関連 ID: ADR-OBS-001 / ADR-OBS-002 / ADR-OBS-003 / IMP-OBS-* / NFR-B-PERF-*
//
// 役割:
//   tier1 Pod の Log / Metric / Trace の 3 信号送出経路を環境変数で切替えて構築する。
//
//   - `OTEL_EXPORTER_OTLP_ENDPOINT` 未設定: stdout JSON Lines 経路（DS-SW-COMP-037
//     の第 1 段。fluentbit / fluentd 経由で Loki に集約される運用前提）。
//   - `OTEL_EXPORTER_OTLP_ENDPOINT` 設定済: OTLP gRPC で OTel Collector に直送
//     （DS-SW-COMP-037 第 2 段。Tempo / Mimir / Loki に振り分けられる）。
//
//   呼出側は本パッケージの NewBundle(ctx) を 1 度だけ呼び、3 emitter を取り出して
//   handler に注入する。Bundle.Shutdown は graceful shutdown で必ず呼ぶこと。

// Package otel は tier1 Go の OpenTelemetry 初期化と span 生成ユーティリティを集約する。
//
// docs 正典: `internal/otel/`（DS-SW-COMP-109、k1s0-otel 共通ライブラリ）。
package otel

import (
	// 起動・shutdown のキャンセル制御に context を使う。
	"context"
	// 失敗ログを stderr に書き出す。
	"log"
	// 環境変数読出。
	"os"
	// shutdown timeout 制御。
	"time"

	// OTel Logs SDK と gRPC exporter。
	otlploggrpc "go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc"
	// OTel Metrics SDK 用 OTLP gRPC exporter。
	otlpmetricgrpc "go.opentelemetry.io/otel/exporters/otlp/otlpmetric/otlpmetricgrpc"
	// OTel Traces SDK 用 OTLP gRPC exporter。
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace"
	otlptracegrpc "go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	// OTel SDK Logger Provider 構築。
	sdklog "go.opentelemetry.io/otel/sdk/log"
	// OTel SDK Meter Provider 構築。
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	// OTel SDK Tracer Provider 構築。
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
)

// Bundle は Log / Metric / Trace の 3 emitter と shutdown 関数を 1 つに束ねる。
// cmd 側は本構造体のフィールドを Deps に注入して使う。
type Bundle struct {
	// Log emitter（LogService.Send / BulkSend が直接呼ぶ）。
	LogEmitter LogEmitter
	// Metric emitter（Telemetry.EmitMetric が直接呼ぶ）。
	MetricEmitter MetricEmitter
	// Trace emitter（Telemetry.EmitSpan が直接呼ぶ）。
	TraceEmitter TraceEmitter
	// graceful shutdown 用関数。Pod 終了時に最大 5 秒で flush + close する。
	Shutdown func(context.Context) error
}

// NewBundle は環境変数を判定して stdout / OTLP gRPC のどちらかの経路で Bundle を構築する。
//
// 判定:
//   - `OTEL_EXPORTER_OTLP_ENDPOINT` が空 → stdout JSON Lines（dev / CI / fluentbit 経路）
//   - `OTEL_EXPORTER_OTLP_ENDPOINT` が設定済 → OTLP gRPC 直送
//
// OTLP gRPC 接続失敗時は fail-soft で stdout にフォールバックする（pod 起動を妨げない）。
func NewBundle(ctx context.Context) Bundle {
	// stdout JSON Lines のフォールバック構築（共通の安全装置として常に保持）。
	stdoutBundle := NewStdoutBundle(os.Stdout)
	// OTLP endpoint 未設定時は stdout 経路を即返す。
	endpoint := os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT")
	// 空文字は OTLP 無効として扱う。
	if endpoint == "" {
		// shutdown は no-op（stdout は flush 不要）。
		return Bundle{
			LogEmitter:    stdoutBundle.LogEmitter,
			MetricEmitter: stdoutBundle.MetricEmitter,
			TraceEmitter:  stdoutBundle.TraceEmitter,
			Shutdown:      func(_ context.Context) error { return nil },
		}
	}
	// OTLP gRPC 経路を構築する。失敗時は stdout にフォールバックする。
	logProvider, metricProvider, traceProvider, err := buildOTLPProviders(ctx)
	// 構築失敗は stderr に記録して stdout 経路で返す（fail-soft、起動を妨げない）。
	if err != nil {
		log.Printf("tier1/otel: OTLP gRPC init failed (%v), falling back to stdout JSON Lines", err)
		// stdout 経路を返す。
		return Bundle{
			LogEmitter:    stdoutBundle.LogEmitter,
			MetricEmitter: stdoutBundle.MetricEmitter,
			TraceEmitter:  stdoutBundle.TraceEmitter,
			Shutdown:      func(_ context.Context) error { return nil },
		}
	}
	// OTLP backed emitter を構築する（既存の OTel SDK ラッパー constructor を使う）。
	logEmitter := NewLogEmitter(logProvider.Logger("tier1"))
	metricEmitter := NewMetricEmitter(metricProvider.Meter("tier1"))
	traceEmitter := NewTraceEmitter(traceProvider.Tracer("tier1"))
	// 統合 shutdown は 3 provider の Shutdown を順次呼ぶ。
	shutdown := func(c context.Context) error {
		// timeout を 5 秒で囲んで flush + close する。
		shutdownCtx, cancel := context.WithTimeout(c, 5*time.Second)
		defer cancel()
		// 1 つでもエラーがあっても残りも呼ぶ（fail-soft）。
		var firstErr error
		// LoggerProvider 終了。
		if e := logProvider.Shutdown(shutdownCtx); e != nil && firstErr == nil {
			firstErr = e
		}
		// MeterProvider 終了。
		if e := metricProvider.Shutdown(shutdownCtx); e != nil && firstErr == nil {
			firstErr = e
		}
		// TracerProvider 終了。
		if e := traceProvider.Shutdown(shutdownCtx); e != nil && firstErr == nil {
			firstErr = e
		}
		return firstErr
	}
	return Bundle{
		LogEmitter:    logEmitter,
		MetricEmitter: metricEmitter,
		TraceEmitter:  traceEmitter,
		Shutdown:      shutdown,
	}
}

// buildOTLPProviders は OTLP gRPC exporter を構築し、3 provider を返す。
// 失敗時は途中で確保した resource を Shutdown してから error を返す。
func buildOTLPProviders(ctx context.Context) (*sdklog.LoggerProvider, *sdkmetric.MeterProvider, *sdktrace.TracerProvider, error) {
	// Logs OTLP gRPC exporter。
	logExp, err := otlploggrpc.New(ctx)
	// 失敗時は早期 return。
	if err != nil {
		return nil, nil, nil, err
	}
	// LogProvider に BatchProcessor で wrap する。
	logProvider := sdklog.NewLoggerProvider(
		sdklog.WithProcessor(sdklog.NewBatchProcessor(logExp)),
	)
	// Metrics OTLP gRPC exporter。
	metricExp, err := otlpmetricgrpc.New(ctx)
	// 失敗時は LogProvider を解放してから return。
	if err != nil {
		_ = logProvider.Shutdown(ctx)
		return nil, nil, nil, err
	}
	// MeterProvider に PeriodicReader で wrap する。
	metricProvider := sdkmetric.NewMeterProvider(
		sdkmetric.WithReader(sdkmetric.NewPeriodicReader(metricExp)),
	)
	// Traces OTLP gRPC exporter。
	traceExp, err := otlptrace.New(ctx, otlptracegrpc.NewClient())
	// 失敗時は前 2 provider を解放してから return。
	if err != nil {
		_ = logProvider.Shutdown(ctx)
		_ = metricProvider.Shutdown(ctx)
		return nil, nil, nil, err
	}
	// TracerProvider に BatchSpanProcessor で wrap する。
	traceProvider := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(traceExp),
	)
	return logProvider, metricProvider, traceProvider, nil
}
