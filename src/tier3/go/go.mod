// k1s0 tier3 Go module（state lib 等価強度実装）
module github.com/k1s0/tier3/go

// Go バージョン
go 1.22

require (
	// k1s0-hlc Go lib（wall-clock TTL 禁止規律に従い HLC を使用する）
	// src/CLAUDE.md §wall-clock TTL 禁止: deadline 計算は HLC ベース必須
	github.com/k1s0/hlc-lib-go v0.0.0
	// OpenTelemetry: client_purge_event を span event として audit emit する（R3-5）
	go.opentelemetry.io/otel v1.32.0
)

// replace: ローカル path 依存（モノレポ内の hlc_lib を参照する）
replace github.com/k1s0/hlc-lib-go => ../../client/hlc_lib/go
