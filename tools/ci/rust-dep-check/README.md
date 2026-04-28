# rust-dep-check

k1s0 リポジトリの Rust workspace（`src/tier1/rust/`、`src/sdk/rust/`、`src/platform/scaffold/`、`tests/fuzz/rust/`）配下の `Cargo.toml` を走査し、`[dependencies]` / `[dev-dependencies]` / `[build-dependencies]` の `path = "..."` が依存方向ルール（[`IMP-DIR-ROOT-002`](../../../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md)）に違反していないかを検証する自作 linter。

## 検出ルール

`Cargo.toml` の `path` 依存が、自身の crate が属する tier の **同 tier 内** または **依存方向ルールで許容された下位 tier** に閉じているかを検証する:

| ソース crate の tier | 許容される `path` 解決先 | 違反例 |
|---|---|---|
| `src/contracts/` | （独立） | sdk / tier1 / tier2 / tier3 への path |
| `src/sdk/rust/` | 同 workspace 内のみ | tier1 / tier2 / tier3 配下の crate を `path` 参照 |
| `src/tier1/rust/` | 同 workspace 内のみ | sdk / tier2 / tier3 配下を `path` 参照 |
| `src/platform/scaffold/` | 同 crate / contracts | tier1〜3 への path |

実装上の判定ロジック:

- `Cargo.toml` の絶対 path → 自身の tier を SourceTierByPath で判定
- 各 dep の `path = "../../foo"` を絶対 path に解決 → 解決先の tier を判定
- 自身の tier から解決先 tier への参照が dep direction で許容されるかを `IsAllowed` で検証

## 利用方法

```bash
cd tools/ci/rust-dep-check
go run ./cmd/rust-dep-check                       # リポジトリ root を git で解決
go run ./cmd/rust-dep-check --root /path/to/repo
go run ./cmd/rust-dep-check --json                # JSON 形式（CI PR コメント用）
```

違反検出時は exit 1。違反なし時は exit 0。

## 実装言語選択

本 linter の実装は Go（[`tools/ci/go-dep-check/`](../go-dep-check/) と同じ）。理由:

- Cargo.toml は標準 TOML フォーマットで、`github.com/BurntSushi/toml` で簡潔に parse できる
- CI runner で setup-go が単一であれば go-dep-check / rust-dep-check 両方を実行できる
- `cargo deny check` の `bans` セクションは別途設定し、本 linter とは責任分界する（cargo deny: 外部 crate 禁止リスト / rust-dep-check: 内部 path 依存方向）

## アーキテクチャ

- `cmd/rust-dep-check/main.go` — CLI entry
- `internal/checker/rules.go` — tier 判定 + 許容方向
- `internal/checker/checker.go` — Cargo.toml 走査 + path 依存解決
- `internal/checker/checker_test.go` — golden test
