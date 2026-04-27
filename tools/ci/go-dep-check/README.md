# go-dep-check

k1s0 リポジトリの Go ソース全体を走査し、`import` 宣言が **tier3 → tier2 → sdk → tier1** 一方向ルール（[`IMP-DIR-ROOT-002`](../../../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md)）に従っているかを検証する自作 Go linter。

`<ProjectReference>` レベルの違反は go.mod / go module 解決そのものが拒否しないので、本 linter が **個別 import 文単位** で逆方向参照を検出する。

## 検出ルール

| ソース tier | 禁止 import prefix |
|---|---|
| `src/contracts/` | （なし、独立） |
| `src/sdk/go/` | `github.com/k1s0/k1s0/src/tier1/`, `.../src/tier2/`, `.../src/tier3/` |
| `src/tier1/go/` | `github.com/k1s0/k1s0/src/tier2/`, `.../src/tier3/`, `github.com/k1s0/sdk-go` |
| `src/tier2/go/` | `github.com/k1s0/k1s0/src/tier1/`, `.../src/tier3/` |
| `src/tier3/bff/` | `github.com/k1s0/k1s0/src/tier1/` |

`src/platform/` はテンプレート経由の間接参照のみ許容（直接 import は他 tier へ可）。

## 利用方法

```bash
cd tools/ci/go-dep-check
go run ./cmd/go-dep-check                       # リポジトリ root を自動解決
go run ./cmd/go-dep-check --root /path/to/repo  # path 明示
go run ./cmd/go-dep-check --json                # 違反を JSON で出力（PR コメント用）
```

違反検出時は exit code 1。違反なし時は exit code 0。

## CI 連携

`.github/workflows/_reusable-lint.yml` の Go job から呼ぶ想定。違反は PR を block する（`tools/ci/dep-check/` がコメント投稿、[`05_依存方向ルール.md`](../../../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md) 違反検出フロー参照）。

## アーキテクチャ

- `cmd/go-dep-check/main.go` — CLI entry
- `internal/checker/rules.go` — tier 識別 + 禁止 prefix 表
- `internal/checker/checker.go` — `go/parser` で import 列挙 + 検証
- `internal/checker/checker_test.go` — golden test
