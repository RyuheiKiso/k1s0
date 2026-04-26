// 本ファイルは tier1 Go の OpenTelemetry 共通初期化ユーティリティ。
//
// 設計: docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/05_モジュール依存関係.md
//       （DS-SW-COMP-109: k1s0-otel 共通ライブラリ、tracer / meter / logger / propagator 集約）
//       plan/04_tier1_Goファサード実装/02_共通基盤.md（plan 側は `internal/observability/` 表記、docs 正典は `internal/otel/`）
// 関連 ID: ADR-OBS-001 / ADR-OBS-002 / ADR-OBS-003 / IMP-OBS-* / NFR-B-PERF-*
//
// scope（リリース時点最小骨格）:
//   - パッケージ宣言と Init / Shutdown のシグネチャ確定のみ
//   - OTel SDK の TracerProvider / MeterProvider / LoggerProvider 初期化、OTLP-gRPC エクスポータ、
//     W3C Trace Context propagator 設定は次セッションで実装
//
// 未実装（plan 04-02 主作業 1〜4 で追加、次セッション以降）:
//   - go.opentelemetry.io/otel / sdk / exporters/otlp/otlpgrpc / propagation 依存の追加と初期化
//   - slog ベース logger（trace_id / span_id を log record に attach）
//   - W3C Trace Context + Baggage propagator 設定
//   - fail-soft（collector 接続失敗時はログ出力で継続、panic 禁止）

// Package otel は tier1 Go の OpenTelemetry 初期化と span 生成ユーティリティを集約する。
//
// docs 正典: `internal/otel/`（DS-SW-COMP-109、k1s0-otel 共通ライブラリ）。
// 各 Pod の cmd は本パッケージの Init を起動時に呼び、Shutdown を defer で登録する。
package otel

// 標準ライブラリのみ import。OTel SDK 依存は次セッションで追加（go.mod 安定優先）。
import (
	// 起動・shutdown のキャンセル制御に context を使う。
	"context"
)

// Init は tier1 Pod 起動時に Trace / Metric / Log の 3 シグナル送信路を初期化する。
//
// scope（リリース時点）: シグネチャのみ確定。実装は次セッション。
// 戻り値の cleanup func は呼び出し側で defer すること（graceful shutdown 時に flush）。
func Init(_ context.Context, _ string) (cleanup func(context.Context) error, err error) {
	// fail-soft 方針: collector 接続失敗時は err を返しつつ no-op cleanup を提供する。
	// caller は err をログに記録した上で起動を継続できる。
	cleanup = func(context.Context) error {
		// TODO(plan 04-02): TracerProvider / MeterProvider / LoggerProvider の Shutdown を呼ぶ。
		return nil
		// 関数リテラルを閉じる。
	}
	// 現状は no-op で nil error を返す（最小骨格）。
	return cleanup, nil
	// Init 関数を閉じる。
}
