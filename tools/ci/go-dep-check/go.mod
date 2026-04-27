// k1s0 Go 依存方向 linter（自作）。
// 設計正典: docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md（IMP-DIR-ROOT-002）
//
// `import` 宣言の path prefix が tier3 → tier2 → sdk → tier1 一方向に従っていることを検証する。
module github.com/k1s0/k1s0/tools/ci/go-dep-check

go 1.22
