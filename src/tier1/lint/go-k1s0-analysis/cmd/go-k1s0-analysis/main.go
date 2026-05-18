// main.go — k1s0 tier1 go-k1s0-analysis: go/analysis analyzer 実行バイナリ
// tier1/CLAUDE.md §lint ツール「自製 go/analysis」の CLI エントリポイント。
// multichecker で k1s0lint analyzer を実行する。

// パッケージ名: main（go-k1s0-analysis バイナリのエントリポイント）
package main

import (
	// golang.org/x/tools/go/analysis/multichecker: 複数 analyzer をまとめて実行するフレームワーク
	"golang.org/x/tools/go/analysis/multichecker"
	// k1s0.dev/tools/go-k1s0-analysis: k1s0 tier1 固有 analyzer のインポート
	gok1s0analysis "k1s0.dev/tools/go-k1s0-analysis"
)

// main は go-k1s0-analysis バイナリのエントリポイント
// multichecker.Main で k1s0lint analyzer を実行する
func main() {
	// multichecker.Main で k1s0lint analyzer を実行する
	// 引数なしで実行した場合は標準 go/analysis ヘルプを表示する
	multichecker.Main(
		// k1s0lint Analyzer を登録する
		gok1s0analysis.Analyzer,
	)
}
