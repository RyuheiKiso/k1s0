// go-k1s0-analysis — k1s0 tier1 Go analysis analyzer モジュール定義
// tier1/CLAUDE.md §lint ツール「golangci-lint depguard + go-apidiff + 自製 go/analysis」の
// 自製 go/analysis 実体を定義する。
module k1s0.dev/tools/go-k1s0-analysis

// Go バージョン: 1.23 を使用する
go 1.23.0

// golang.org/x/tools: go/analysis framework に使用する
require golang.org/x/tools v0.28.0

require (
	golang.org/x/mod v0.22.0 // indirect
	golang.org/x/sync v0.10.0 // indirect
)
