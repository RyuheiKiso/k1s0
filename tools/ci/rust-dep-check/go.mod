// k1s0 Rust workspace 依存方向 linter（Go 実装）。
// 設計正典: docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md（IMP-DIR-ROOT-002）
//
// Rust の Cargo.toml [dependencies] に tier 越境 path 依存が混入していないか検証する。
// 実装は Go（go-dep-check と同じ runtime / CI 連携を共有、Cargo.toml は標準 TOML）。
module github.com/k1s0/k1s0/tools/ci/rust-dep-check

go 1.22

require github.com/BurntSushi/toml v1.4.0
