# 01. Go module 分離戦略

本ファイルは k1s0 モノレポにおける Go の `go.mod` 境界を確定する。Go には Cargo のような workspace 内 members 機構の明確な代替が無く（`go.work` は開発時のみの union 機構であり build 単位を変えない）、module 分割をどこで切るかがビルド時間・依存方向の成否を直接決める。ADR-TIER1-001（Go + Rust ハイブリッド）と ADR-DIR-001（contracts 昇格）の帰結を受け、本ファイルでは Uber の monorepo 運用知見（`02_世界トップ企業事例比較.md`）を引きつつ、tier 境界と所有権境界を二軸とした **5 module 分離** を固定する。

![Go module 5 分割と replace 経路](img/go_module_5分割.svg)

`00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md` は tier1 Go 単独の `go.mod` 内部構造を確定済である。本ファイルはその上位層、つまり「モノレポ全体でいくつ `go.mod` を置くか」と「どう依存方向を強制するか」を固定する役割を持つ。

## なぜ単一 module を選ばないか

Go には歴史的に「1 リポジトリ 1 module」が推奨された時期があったが、Uber（`02_世界トップ企業事例比較.md` 参照）や Google、Cloudflare は早期に複数 module へ切り替えている。単一 module の破綻点は以下である。

- tier3 BFF が追加する `echo` や `fiber` のような Web フレームワーク依存が tier1 Pod の `go.mod` に indirect 依存として混入し、tier1 の脆弱性面が広がる
- SDK Go を `crates.io` 相当の `go get` 外部公開したい Phase 1b で、`go.mod` を途中分割すると既存利用者の import path が破壊される
- `go mod tidy` が tier1 変更でも tier3 依存を再整理し、CI の diff が意味のない indirect dep 変化で膨らむ
- `go test ./...` がリポジトリ全体を走査してテスト時間が線形に悪化する

Uber の事例では 100+ module で運用されており、本プロジェクトはそこまでの粒度は要らないが、**tier 境界と所有権境界** を超える単一 module は早期に破綻するため採用しない。

## 5 module の境界定義

5 module 構成を IMP-BUILD-POL-002（ワークスペース境界 = tier 境界）の具体適用として確定する。境界は以下である。所有権・リリース周期・依存方向のすべてが独立している 5 つを module として切り出し、それ以外の細分化は Phase 1b 以降の事例観察で再評価する。

| module path | 用途 | module name | 所有 |
|---|---|---|---|
| `src/tier1/go/go.mod` | Dapr ファサード 3 Pod（state / secret / workflow）| `github.com/k1s0/k1s0/src/tier1/go` | `@k1s0/tier1-go` |
| `src/tier2/go/go.mod` | tier2 ドメイン共通ロジック（Go サービス群）| `github.com/k1s0/k1s0/src/tier2/go` | `@k1s0/tier2-team` |
| `src/tier3/bff/go.mod` | Backend For Frontend（Web / Native 向けアグリゲータ）| `github.com/k1s0/k1s0/src/tier3/bff` | `@k1s0/tier3-web` |
| `src/sdk/go/go.mod` | 外部公開 SDK（tier2 / tier3 / 社外クライアント向け）| `github.com/k1s0/sdk-go` | `@k1s0/platform-team` |
| `tests/go.mod` | E2E / Contract / Integration テストスイート | `github.com/k1s0/k1s0/tests` | `@k1s0/qa-team` |

tier2 Go は Uber 的には「サービスごとに `go.mod` を切る」選択もあり得るが、Phase 1a 時点の tier2 Go サービス数は数件の想定であり、1 module に集約してサービスを `services/<name>/` サブディレクトリで分離するほうが `go mod tidy` / `go build ./...` の運用コストが低い。tier2 Go サービス数が 10 を超えた段階で分割を ADR で再評価する。

SDK Go の module name だけリポジトリ path と異なるのは、crates.io 相当の外部公開時に `go get github.com/k1s0/sdk-go` 形式を維持するため。リポジトリ物理配置は `src/sdk/go/` だが、`go.mod` の宣言は `module github.com/k1s0/sdk-go` とし、go proxy / goproxy.io 経由のパブリッシュ経路を意識する。

## replace ディレクティブによる内部 module 参照

5 module の間で内部依存が必要なケースは以下の 3 経路のみ許容する。それ以外は原則 IMP-BUILD-POL-003（逆流拒否）で CI lint 拒否。

- `src/tier2/go/` → `src/sdk/go/`：tier2 が SDK を経由して tier1 を呼ぶ
- `src/tier3/bff/` → `src/sdk/go/`：BFF が SDK を経由して tier1 を呼ぶ
- `tests/` → 全 module：E2E テストは全 module を横断

内部 module 参照は `replace` ディレクティブで相対 path 指定する。これにより、ローカル開発で SDK 変更が tier2 / BFF に即時反映され、かつ CI は `replace` を検出して「内部 module のみへの依存」であることを lint で検証できる。

```go
// src/tier2/go/go.mod
module github.com/k1s0/k1s0/src/tier2/go

go 1.22

require (
    github.com/k1s0/sdk-go v0.0.0
    // 外部依存...
)

replace github.com/k1s0/sdk-go => ../../sdk/go
```

Phase 1b で SDK Go を外部公開する際は `replace` を外した CI ジョブを追加し、go proxy 経由でも build が通ることを検証する。`replace` に依存した build が外部公開版で通らないケースを事前に検出する。

## 依存方向逆流の独自 linter

`tools/ci/go-layer-check/` に独自 linter を置く。`go/ast` を使って全 import を走査し、以下の違反を検出する。

- `src/tier1/go/` 配下のファイルが `src/tier2/go/` / `src/tier3/` / `src/sdk/go/` を import
- `src/tier2/go/` 配下のファイルが `src/tier3/` を import
- `src/sdk/go/` 配下のファイルが `src/tier1/` / `src/tier2/` / `src/tier3/` を import（SDK は contracts からの生成物にのみ依存）
- `src/tier3/bff/` 配下のファイルが `src/tier1/` を直接 import（SDK 経由必須）

linter は Go のパッケージ解析 API（`golang.org/x/tools/go/packages`）で 5 module を独立解析し、import graph の頂点が定義された依存方向に沿うことを検証する。違反は `30_CI_CD設計/` の reusable workflow の lint 段で fail し、PR merge をブロックする。

```go
// tools/ci/go-layer-check/main.go（概念）
// 各 module の root を引数に取り、import graph の依存方向を検証する
func checkLayer(modulePath string, allowedLayers []string) error {
    // packages.Load で該当 module の全 package を読み込む
    pkgs, err := packages.Load(&packages.Config{
        Mode: packages.NeedImports | packages.NeedFiles | packages.NeedName,
        Dir:  modulePath,
    }, "./...")
    if err != nil {
        return err
    }
    // 全 import を検査し、allowedLayers 以外への依存があれば error
    for _, pkg := range pkgs {
        for imp := range pkg.Imports {
            if !isAllowed(imp, allowedLayers) {
                return fmt.Errorf("layer violation: %s imports %s", pkg.PkgPath, imp)
            }
        }
    }
    return nil
}
```

## go.sum 整合と go mod tidy 検証

CI の lint 段では各 module で `go mod tidy -diff` を実行し、commit された `go.sum` / `go.mod` と `tidy` 結果が一致することを検証する。差分がある PR は「依存ドリフトが含まれる」として拒否。`-diff` フラグは Go 1.23 以降で利用可能であり、本プロジェクトは `go 1.22` を最低とするが、CI 実行環境では Go 1.23+ を使って lint する運用とする（開発者ローカルは 1.22 で可）。

```bash
# CI lint 段の雛形（30_CI_CD設計/ reusable workflow 内）
for mod in src/tier1/go src/tier2/go src/tier3/bff src/sdk/go tests; do
    # 各 module ディレクトリで tidy diff を検証
    (cd "$mod" && go mod tidy -diff) || exit 1
done
```

`go mod verify` も同ジョブで呼び、module cache の hash 整合を検証する。`GOFLAGS=-mod=readonly` を CI 全体で強制し、ビルド中に go.mod を暗黙変更させない。

## 選択ビルドと path-filter 連動

IMP-BUILD-POL-004（path-filter 選択ビルド）の Go 面実装は以下。

- `src/tier1/go/**` 変更 → tier1 Go のみ `go build ./...` + `go test ./...`
- `src/tier2/go/**` 変更 → tier2 Go のみ
- `src/tier3/bff/**` 変更 → BFF のみ
- `src/sdk/go/**` 変更 → SDK Go + 下流（tier2 Go / BFF）を replace 経由で連動ビルド
- `src/contracts/**` 変更 → 全 Go module をビルド（契約変更は横断伝播）
- `tests/**` 変更 → E2E のみ

各 module 内での選択ビルドは Go が `go build ./...` で module 全体を build するため module 単位が最小粒度となる。Uber のような `go build //path/to/pkg:target` 単位の細粒度選択は Bazel が必要であり、本プロジェクトでは Phase 1c の再評価対象（IMP-BUILD-POL-001）。

## GOCACHE のリモート化

Go は Rust の sccache に相当する公式リモートキャッシュ機構を持たないため、IMP-BUILD-POL-005（3 層キャッシュ）の第 3 層は CI の GitHub Actions cache（第 2 層相当）に留まる。ただし GOCACHE 自体は `GOCACHE=$GITHUB_WORKSPACE/.cache/go-build` に設定し、`actions/cache` でキャッシュする。キーは `${{ runner.os }}-go-build-${{ hashFiles('**/go.sum') }}` の形式で module 間共有可能とする。

Go 1.24 で実験導入された `GOCACHEPROG`（外部キャッシュプログラム連携）が stable 化した段階で、sccache / buildbuddy 連携の ADR を起票する。Phase 1c の再評価対象。

## gopls / IDE 設定

5 module を同一リポジトリで扱うため、VSCode / Goland の gopls は workspace root ではなく各 module root を認識する必要がある。`.vscode/settings.json` で明示する。

```json
{
  "go.useLanguageServer": true,
  "gopls": {
    "experimentalWorkspaceModule": true,
    "expandWorkspaceToModule": true
  },
  "go.toolsEnvVars": {
    "GOFLAGS": "-mod=mod"
  }
}
```

`go.work` は **Phase 1a 時点では採用しない**。`go.work` は開発時の複数 module 統合を楽にするが、`go.work` が commit されると CI と開発者で build 単位がズレる事故が起きる。Uber も `go.work` を使わず replace 運用を維持しており、本プロジェクトも同方針。

## ディレクトリ配置まとめ

5 module の物理配置と各 module 固有設定は以下で確定する。

| path | 役割 | 主要設定 |
|---|---|---|
| `src/tier1/go/go.mod` | tier1 Dapr ファサード 3 Pod | `dapr/go-sdk`、`google.golang.org/grpc` |
| `src/tier2/go/go.mod` | tier2 Go ドメインサービス群 | `replace github.com/k1s0/sdk-go` |
| `src/tier3/bff/go.mod` | BFF（Web / Native 向け）| `replace github.com/k1s0/sdk-go`、`echo` / `fiber` 等 |
| `src/sdk/go/go.mod` | SDK Go（外部公開）| 最小依存、`module github.com/k1s0/sdk-go` |
| `tests/go.mod` | E2E / Contract / Integration | `replace` で全 module 参照 |
| `tools/ci/go-layer-check/` | 独自依存方向 linter | `golang.org/x/tools/go/packages` |

## 対応 IMP-BUILD ID

- `IMP-BUILD-GM-020` : 5 module 分離（tier1 Go / tier2 Go / BFF / SDK Go / tests）
- `IMP-BUILD-GM-021` : 内部 module 参照の `replace` ディレクティブ運用
- `IMP-BUILD-GM-022` : 独自 linter による依存方向逆流検出
- `IMP-BUILD-GM-023` : `go mod tidy -diff` による go.sum ドリフト検出
- `IMP-BUILD-GM-024` : path-filter 選択ビルドの Go 面実装
- `IMP-BUILD-GM-025` : GOCACHE の actions/cache リモート化
- `IMP-BUILD-GM-026` : `go.work` 不採用（commit 事故防止）
- `IMP-BUILD-GM-027` : SDK Go module name の外部公開路径（`github.com/k1s0/sdk-go`）固定

## 対応 ADR / DS-SW-COMP / NFR

- ADR-TIER1-001（Go + Rust ハイブリッド）/ ADR-TIER1-003（内部言語不可視）/ ADR-DIR-001（contracts 昇格）
- DS-SW-COMP-003（Dapr / Rust 二分）/ 121（contracts 配置）/ 124（tier1 Go 内部）/ 130（SDK 生成）
- NFR-B-PERF-001（性能基盤）/ NFR-C-NOP-004（ビルド所要時間運用）/ NFR-C-MNT-003（API 互換方針）
