// go.mod — k1s0-hlc Go モジュール定義
// wall-clock TTL 禁止規律（src/CLAUDE.md §wall-clock TTL 禁止）に従い、
// 全 deadline 計算を HLC ベースで行う基盤 Go パッケージを定義する。
module github.com/k1s0/hlc-lib-go

// Go 1.22 以上を要求する（generics / range-over-int の安定版）
go 1.22
