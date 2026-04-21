# 07. 開発ツールと CI/CD 配置

本ファイルは `tools/` / `scripts/` / `.github/workflows/` / `.devcontainer/` 配下の詳細ファイル配置を方式として固定化する。上流は概要設計 [70_開発者体験方式設計/01_CI_CD方式](../../04_概要設計/70_開発者体験方式設計/01_CI_CD方式.md)・[70_開発者体験方式設計/02_ローカル開発環境方式](../../04_概要設計/70_開発者体験方式設計/02_ローカル開発環境方式.md)・[50_開発者体験方式設計/03_Feature_Management方式](../../04_概要設計/50_開発者体験方式設計/03_Feature_Management方式.md)で、本ファイルは開発支援ツール・CI ワークフロー・Dev Container の配置を確定させる。

## 本ファイルの位置付け

[01_リポジトリルート構成.md](01_リポジトリルート構成.md) の `DS-IMPL-DIR-007` 〜 `DS-IMPL-DIR-010` で `tools/` と `scripts/` の配置原則を、`DS-IMPL-DIR-011` 〜 `DS-IMPL-DIR-013` で `.github/` と `.devcontainer/` の配置原則を固定した。本ファイルはその 1 階層下を詳細化する。具体的には、(a) `tools/<name>/` 配下の個別ツール構造、(b) `scripts/*.sh` 各スクリプトの責務、(c) `.github/workflows/` の PR / main / release / nightly 4 系統のジョブ分割、(d) `.devcontainer/devcontainer.json` と `Dockerfile` の構成、(e) `CODEOWNERS` のチーム割り当てを確定する。

配置が曖昧なまま Phase 1a に入ると、「このスクリプトはどこに置く？」「この CI ジョブはどのワークフローに書く？」を毎回判断することになり、PR レビュー時の議論が発散する。また、CI ワークフローは変更頻度が高いが、所有者が曖昧だと壊れた時の対応が遅れる。本章は配置と所有権を同時に固定することで、Phase 1a 以降の iteration を高速化する。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-IMPL-DIR-181` 〜 `DS-IMPL-DIR-200` の 20 件である。

## tools/ 配下の全体構造

`tools/` の完全な構造は以下のとおり。

```
tools/
├── README.md                             # tools/ の使い方と各ツールの関係
├── proto-gen/                            # buf CLI ラッパ（Rust）
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   └── src/main.rs
├── check-deps/                           # コンポーネント依存検査（Go）
│   ├── go.mod
│   ├── go.sum
│   ├── main.go
│   └── internal/
│       ├── analyzer/
│       └── rules/
├── check-rust-deps/                      # Rust path 依存の静的検査（04 章 DS-IMPL-DIR-114 用、Phase 1a）
│   ├── Cargo.toml
│   └── src/main.rs
├── k1s0-cli/                             # 雛形生成 CLI（Rust）
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── new.rs                    # `k1s0 new <template>`
│   │   │   ├── dev.rs                    # `k1s0 dev up / down`
│   │   │   └── doctor.rs                 # `k1s0 doctor`
│   │   └── templates/                    # 雛形テンプレート本体
│   └── tests/
├── mock-server/                          # tier1 API モック（Go）
│   ├── go.mod
│   ├── go.sum
│   └── main.go
├── backstage/                            # Backstage TechDocs / Catalog 設定（Phase 1c）
│   ├── mkdocs.yml
│   └── catalog-info.yaml
└── backstage-templates/                  # Backstage Software Template（Phase 1b〜）
    ├── README.md                         # テンプレート全体の使い方と更新手順
    ├── tier2/                            # tier2 マイクロサービス雛形
    │   ├── go/                           # Go + Dapr client + 4 層（10 章 DS-IMPL-DIR-241〜245）
    │   │   ├── template.yaml             # Backstage Software Template エントリ
    │   │   ├── skeleton/                 # 生成されるリポジトリの雛形
    │   │   └── README.md
    │   └── csharp/                       # .NET 8 + Dapr client + 4 層（10 章 DS-IMPL-DIR-246〜250）
    │       ├── template.yaml
    │       ├── skeleton/
    │       └── README.md
    └── tier3/                            # tier3 エンドユーザーアプリ雛形
        ├── webapp-typescript/            # Next.js + React + i18n（11 章 DS-IMPL-DIR-261〜266）
        │   ├── template.yaml
        │   ├── skeleton/
        │   └── README.md
        ├── mobile-maui/                  # .NET MAUI（11 章 DS-IMPL-DIR-267〜272）
        │   ├── template.yaml
        │   ├── skeleton/
        │   └── README.md
        └── bff-typescript/               # Fastify BFF（11 章 DS-IMPL-DIR-273〜278）
            ├── template.yaml
            ├── skeleton/
            └── README.md
```

各ツールは独立したサブディレクトリで `go.mod` または `Cargo.toml` を個別に持つ。これは [DS-IMPL-DIR-008](01_リポジトリルート構成.md) で確定した独立モジュール原則の適用である。ツール間の依存は共有せず、本当に共有したいライブラリがあれば `tools/common/` を別途作るルールとする（Phase 1a 時点ではそのような需要がないため作らない）。

`backstage-templates/` は tier2/tier3 の 2 階層で分類し、その下で変種別（Go/C#、Web/Mobile/BFF）に分ける。フラット配置（例: `tools/backstage-templates/tier2-microservice-go/`）ではなく 2 階層にする理由は、Phase 2 以降で tier2 の新変種（例: `tier2/python/`）や tier3 の新変種（例: `tier3/desktop-tauri/`）を追加する際、`tier2/` / `tier3/` のどこに置くかが機械的に決まり、命名の一貫性が保たれるためである。また CODEOWNERS や CI path filter も `tools/backstage-templates/tier2/**` / `tools/backstage-templates/tier3/**` として階層でまとめて指定できる。

## DS-IMPL-DIR-181 tools/proto-gen/ の配置

`tools/proto-gen/` は buf CLI のラッパを Rust で実装する。単なる buf 呼び出しではなく、`proto` ファイルのスキーマ検証・lint・breaking change 検出・Go / Rust 両言語への生成を 1 コマンドで実行する責務を持つ。

構造は以下。

- `tools/proto-gen/Cargo.toml`: clap / tokio / `buf` コマンド呼び出しライブラリ（またはシェルアウト）
- `tools/proto-gen/src/main.rs`: CLI エントリポイント
- `tools/proto-gen/rust-toolchain.toml`: `src/tier1/rust/` と同一バージョン（1.85）を pin

利用側は `scripts/proto-gen.sh` 経由で呼び出す。CI ワークフロー（`.github/workflows/pr.yml`）の contracts 変更検出ジョブも本ツール経由で実行する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、FR-T1-\*（契約生成）。**上流**: DS-IMPL-DIR-007、DS-IMPL-DIR-021（contracts）。

## DS-IMPL-DIR-182 tools/check-deps/ の配置

`tools/check-deps/` は概要設計 [DS-SW-COMP-113](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/05_モジュール依存関係.md) で宣言された `tools/check-deps.sh` を Go に移植したものである。シェルスクリプトでは Go / Rust の静的解析を両立しにくいため、Go 実装として統一する。

責務は 4 つ。

1. Go の import を解析し、layer 逸脱（handler → repository 直接呼び出し等）および `cmd → pods → shared` の 3 層を跨ぐ逆流、Pod 間 import（`pods/state/` から `pods/secret/` を参照する等）を検出
2. Rust の `Cargo.toml` `[dependencies]` を解析し、domain crate が adapter crate に依存していないか、`crates/pods/` 同士の path 依存が発生していないかを検証（Rust 側の詳細検査は `tools/check-rust-deps/` に委譲し、本ツールは cross-language の層分離のみを担当）
3. tier2 / tier3 領域が tier1 内部 API（`src/tier1/go/internal/pods/` / `src/tier1/go/internal/shared/` / `src/tier1/rust/crates/pods/*` / `src/tier1/rust/crates/shared/*` の private 型）を import していないか検証
4. **docs ↔ 実体の双方向整合検証（Phase 1b 追加）**: `docs/05_実装/00_ディレクトリ設計/**/*.md` 内の全 `DS-IMPL-DIR-NNN` 宣言と、実ディレクトリ（`src/tier1/` / `tests/` / `tools/` / `.github/` / `.devcontainer/`）の整合を双方向に検査する。具体的には (a) 本章内で宣言された各 DS-IMPL-DIR-NNN が言及する実ファイル／ディレクトリ（`src/tier1/go/cmd/<pod>/main.go`、`src/tier1/rust/crates/pods/<pod>/`、`tools/<name>/` 等）の存在確認、(b) `src/tier1/` 配下の第 2 階層（`go/internal/pods/<pod>/`、`rust/crates/pods/<crate>/`、`infra/{delivery,runtime,governance,platform}/<subdir>/`）が対応する DS-IMPL-DIR-NNN に紐付けられているかの確認、(c) ID 飛び番の検出（本章 README.md は飛び番禁止を宣言）。違反は PR ブロックではなく**週次 digest** として `@k1s0/tier1-architects` に通知する（PR ブロック扱いは false positive 率が高いため）。長期運用（5 年超）で docs と実体が乖離する drift を継続的に可視化する狙い。

構造は以下。

- `tools/check-deps/main.go`: CLI エントリ
- `tools/check-deps/internal/analyzer/go.go`: Go AST 解析（`golang.org/x/tools/go/packages`）
- `tools/check-deps/internal/analyzer/rust.go`: Rust 解析（`cargo metadata` の JSON 出力を parse）
- `tools/check-deps/internal/analyzer/docs.go`: Markdown 解析（`goldmark` AST から `DS-IMPL-DIR-NNN` 見出しと参照パス文字列を抽出、Phase 1b 追加）
- `tools/check-deps/internal/rules/layer.go`: 層間依存ルール（go-cleanarch 設定と整合、03 章 DS-IMPL-DIR-074）
- `tools/check-deps/internal/rules/pod_isolation.go`: Pod 間 import 禁止ルール（Go `pods/<pod>/` 横断と Rust `crates/pods/<crate>/` 横断、03 章 DS-IMPL-DIR-074 / 04 章 DS-IMPL-DIR-114）
- `tools/check-deps/internal/rules/root.go`: root 禁止物（`go.mod` / `Cargo.toml` 等、01 章 DS-IMPL-DIR-019）の検出
- `tools/check-deps/internal/rules/shared_residency.go`: shared/ 追加・撤収判定基準（Go / Rust の利用 Pod 数と類似度）の週次検査（03 章 DS-IMPL-DIR-050 / 04 章 DS-IMPL-DIR-088、Phase 1b 追加）
- `tools/check-deps/internal/rules/ds_mapping.go`: 責務 4 の docs ↔ 実体整合検証（Phase 1b 追加）

CI（`.github/workflows/pr.yml`）で PR ごとに実行し、違反を検出したら merge block する（ただし責務 4 は週次デジタルのみで PR ブロックしない）。

**確定フェーズ**: Phase 1a（責務 1〜3）、Phase 1b（責務 4 の docs 整合検証 + shared residency 週次ジョブ追加）。**対応要件**: DX-CICD-\*、NFR-C-NOP-001、NFR-C-NOP-002。**上流**: DS-SW-COMP-113、DS-IMPL-DIR-007、DS-IMPL-DIR-019、DS-IMPL-DIR-050、DS-IMPL-DIR-088。

## DS-IMPL-DIR-183 tools/k1s0-cli/ のコマンド配置

`tools/k1s0-cli/` は開発者体験の中核ツールで、Rust 製の CLI である。概要設計 [DS-DEVX-LOCAL-\*] で「`k1s0 dev up` でローカル環境を起動」「`k1s0 new <template>` で雛形生成」が宣言されており、本ツールはその実装を担う。

コマンド配置（`src/commands/<cmd>.rs`）は以下。

- `new.rs`: 雛形生成の入口（後述のサブコマンド分割）。テンプレートは `src/templates/<kind>/` に同梱
- `dev.rs`: ローカルクラスタ起動・停止（`k1s0 dev up --tier1=real`、`k1s0 dev up --tier1=mock`）
- `doctor.rs`: ローカル環境の診断（kind / docker / mise が正しくインストールされているか）

Phase 1a 時点で `new` と `doctor` を実装。`dev` は Phase 1b で追加する（tier2 領域が存在するまで不要）。

**`k1s0 new` のサブコマンド体系と横断ファイル自動改訂**: 新 Pod / 新 crate / 新 API の追加は、単にソースファイルを生成するだけでは足りない。03 章 DS-IMPL-DIR-080（Go Pod 追加手続）と 04 章 DS-IMPL-DIR-120（Rust crate 追加手続）は、**1 回の追加あたり 7 箇所の改訂**（ソース生成 + `Cargo.toml` / `go.work` 更新 + `.github/workflows/pr.yml` の path filter 追加 + CODEOWNERS エントリ追加 + 対応 ADR スタブ + 雛形テストファイル + `README.md` 参照更新）を要求する。これを人手で行うと、10 Pod を超えた時点で path filter と CODEOWNERS が必ず drift する（過去事例: 類似規模の monorepo で 8 Pod 時点で workflows の paths-ignore が lint と乖離）。`k1s0 new` は**横断ファイル改訂までを atomic に実行する**責務を持ち、手作業を禁ずる。サブコマンドは以下 5 種。

- **`k1s0 new pod-go --name <pod>`**: `src/tier1/go/cmd/<pod>/` + `src/tier1/go/internal/pods/<pod>/{handler,service,domain,repository}/` の 4 層スケルトン + `integrationtest/<pod>/` + `wire.go` stub を生成。さらに以下 4 箇所を自動改訂する。(1) `/CODEOWNERS` に `/src/tier1/go/cmd/<pod>/ @k1s0/tier1-go-team` と `/src/tier1/go/internal/pods/<pod>/ @k1s0/tier1-go-team` を追加、(2) `/.github/workflows/pr.yml` の `paths:` filter に `src/tier1/go/cmd/<pod>/**` と `src/tier1/go/internal/pods/<pod>/**` を追加、(3) `/docs/02_構想設計/adr/` に `ADR-PODS-<NNN>-add-<pod>-pod.md` の ADR スタブを生成、(4) `src/tier1/go/README.md` の Pod 一覧に行を追加。
- **`k1s0 new pod-rust --name <pod>`**: `src/tier1/rust/crates/pods/<pod>/{src/{grpc,service,domain,adapter},tests}/` を生成。横断改訂は (1) `src/tier1/rust/Cargo.toml` の `[workspace]` `members` に `crates/pods/<pod>` を追加（alphabetical order を維持）、(2) CODEOWNERS に `/src/tier1/rust/crates/pods/<pod>/ @k1s0/tier1-rust-team`、(3) `pr.yml` の paths filter に `src/tier1/rust/crates/pods/<pod>/**` を追加、(4) ADR スタブ起票。Go 版と対称に扱う。
- **`k1s0 new shared-crate --name <crate>`**: `src/tier1/rust/crates/shared/<crate>/` を生成するが、本サブコマンドは **ADR pre-check を必須**とする（03 章 DS-IMPL-DIR-050 / 04 章 DS-IMPL-DIR-088 の数値判定基準に準拠）。`--adr <ADR-ID>` フラグで既存 ADR を指定しない限り exit code 1。ADR 指定時のみ Cargo.toml members 追加と CODEOWNERS（`@k1s0/tier1-architects` 必須承認）を自動改訂する。Go 側の `internal/shared/<topic>/` も対称のコマンド `k1s0 new shared-go --name <topic> --adr <ID>` で扱う。
- **`k1s0 new api --name <api>`**: `src/tier1/contracts/v1/<api>/v1.proto` + `contracts/buf.gen.yaml` の `<api>` 登録 + Go / Rust 生成先ディレクトリの README stub を生成。02 章 DS-IMPL-DIR-031 の `tools/check-proto-symmetry/` にも `<api>` を自動登録し、片側未生成を検出できる状態にする。ADR 起票を必須とする（新 API 追加は tier1 bounded context の境界変更）。
- **`k1s0 new tier2-service --name <service> --lang <go|csharp>`**: Backstage Software Template の Dry-Run 代替として、ローカルで `tools/backstage-templates/tier2/<lang>/skeleton/` を展開する（実運用では Backstage UI 経由、ローカル検証用）。GitHub repo 作成と catalog 登録は行わない（副作用を持たない）。

横断改訂の atomicity は、**生成 → 改訂 → git diff 表示 → ユーザ確認 → git add** の順で 1 PR 単位にまとめる。途中 fail 時は全改訂を rollback する（`tools/k1s0-cli/src/txn.rs` に transaction wrapper を実装）。これにより「ソースだけ生成されて CODEOWNERS と workflows が未改訂の半端状態」が commit される事態を防ぐ。

**CLI 自身のテスト**: `k1s0 new` の生成結果は 06 章 DS-IMPL-DIR-173 の `tests/golden/` 配下（`tests/golden/k1s0-cli/pod-go/` など）にゴールデンを配置し、CLI の PR で diff をレビューする。sub-command 追加時はゴールデンテストの追加を必須とする。

**確定フェーズ**: Phase 1a（new-pod-go / new-pod-rust / new-api / doctor + 横断改訂の atomic 実装）、Phase 1b（new-tier2-service、new-shared-\*、dev）。**対応要件**: DX-LD-\*、DX-DEVEX-\*、DX-GP-\*、NFR-C-NOP-001、NFR-C-NOP-002。**上流**: DS-DEVX-LOCAL-\*、DS-IMPL-DIR-007、DS-IMPL-DIR-018、DS-IMPL-DIR-080、DS-IMPL-DIR-120、DS-IMPL-DIR-173、DS-IMPL-DIR-199。

## DS-IMPL-DIR-184 tools/mock-server/ の配置

`tools/mock-server/` は tier1 API 11 個のモック実装を Go で提供する。`k1s0 dev up --tier1=mock` から呼び出され、tier2 開発者が tier1 を立てずにローカル開発できるようにする。

実装方針: Protobuf から生成された gRPC サーバインタフェースを実装し、`fixtures/` ディレクトリ（[DS-IMPL-DIR-169](06_テストとフィクスチャ配置.md)）から応答を返す。決定論的な応答を返す（入力 → 出力が 1:1）ことで、tier2 側のテストが安定する。

構造は `tools/mock-server/main.go` 1 ファイルで開始し、300 行超過時に `internal/` に分割する。

**確定フェーズ**: Phase 1b（tier2 登場時）。**対応要件**: DX-LD-\*、DX-DEVEX-\*。**上流**: DS-DEVX-LOCAL-\*、DS-IMPL-DIR-169。

## DS-IMPL-DIR-185 tools/backstage/ と tools/backstage-templates/ の配置

Backstage 系の配置は責務別に 2 ディレクトリに分ける。`tools/backstage/` は Backstage 本体（TechDocs と Catalog）の設定、`tools/backstage-templates/` は Backstage Software Template による雛形生成定義で、参照される tier / 変種が増えても独立に進化できるようにする。

### tools/backstage/（Phase 1c）

Backstage TechDocs と Catalog の設定を置く。

- `mkdocs.yml`: TechDocs の MkDocs 設定（`docs/` を source とする）
- `catalog-info.yaml`: Backstage Component / System / API の登録

Phase 1a 〜 1b 時点では Backstage が立ち上がっていないため、本ディレクトリは Phase 1c で追加する。Phase 1a 時点では空のディレクトリも作らず、08 章の命名規約で「必要時に追加するディレクトリ」として列挙する。

### tools/backstage-templates/（Phase 1b〜）

`tools/backstage-templates/tier2/` と `tools/backstage-templates/tier3/` の 2 階層で分類し、その下で変種別（Go/C#、Web/Mobile/BFF）に分ける。各テンプレートは Backstage Software Template の仕様に従い `template.yaml`（parameters / steps / publish）と `skeleton/`（生成先に展開されるファイル群）で構成する。

Phase 1b では `tier2/go/` を先行投入（10 章 DS-IMPL-DIR-241〜245 の tier2-go 構成を反映）、tier2/csharp と tier3 系は需要に応じて Phase 1b 後半から Phase 2 に掛けて順次追加する。テンプレート本体が参照する値（tier1 gRPC エンドポイント、Helm Chart バージョン等）は placeholder を使い、生成時に Backstage の Action（`fetch:template` / `publish:github`）で注入する。

Backstage Software Template の breaking change（parameters 削除・rename）は必ず ADR を起票し、利用済み tier2/tier3 リポジトリへのマイグレーション手順を同梱する。テンプレート生成結果のゴールデンは 06 章 DS-IMPL-DIR-173 の `tests/golden/` に格納し、テンプレート更新時の差分検知を自動化する。

**確定フェーズ**: `tools/backstage/` は Phase 1c、`tools/backstage-templates/` は Phase 1b（tier2/go 先行）〜 Phase 2（全変種）。**対応要件**: DX-DEVEX-\*、DX-CICD-\*、NFR-SUP-\*。**上流**: DS-DEVX-DOC-\*、DS-IMPL-DIR-173（golden）、DS-IMPL-DIR-241〜278（tier2/tier3 雛形）。

## scripts/ 配下の詳細配置

`scripts/` は bash で書かれた開発者向けヘルパスクリプトで、[DS-IMPL-DIR-009](01_リポジトリルート構成.md) で責務が固定済み。本章ではファイル別の詳細を定める。

## DS-IMPL-DIR-186 scripts/bootstrap.sh の責務

`scripts/bootstrap.sh` は新規開発者が `git clone` 直後に実行する唯一のスクリプト。以下を順次実行する。

1. `mise install`（`.mise.toml` から Go / Rust / Node / Python のバージョンをインストール）
2. `pre-commit install`（`.pre-commit-config.yaml` を有効化）
3. `docker` / `kind` / `kubectl` / `helm` / `buf` の存在確認（なければインストール手順を案内）
4. `tools/proto-gen/` をビルドし、`.proto` から Go / Rust 生成物を一度生成
5. `cargo fetch` / `go mod download` で依存解決

失敗時は原因を標準出力に出し、exit code 1 で終了する。冪等性を保ち、2 回目以降は差分だけ処理する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-LD-\*、DX-DEVEX-\*。**上流**: DS-DEVX-LOCAL-\*、DS-IMPL-DIR-009。

## DS-IMPL-DIR-187 scripts/ 各スクリプトの責務一覧

`scripts/` 配下には Phase 1a で以下 7 種を配置する。

- `bootstrap.sh`: 初回環境セットアップ（上記 DS-IMPL-DIR-186）
- `proto-gen.sh`: `tools/proto-gen/` の薄いラッパ。buf generate を実行して Go / Rust 生成物を更新
- `run-e2e.sh`: `tests/e2e/` の E2E シナリオを kind クラスタ上で実行
- `lint-all.sh`: 全言語（Go: golangci-lint / Rust: clippy / TypeScript: eslint / Helm: helm lint）の lint を一括実行
- `smoke-test.sh`: Phase 1a の smoke test（OTel ping / ファサード 3 Pod の Service Invoke 往復）
- `db-migrate.sh`: PostgreSQL 向けの sqlx migration 実行ヘルパ（AUDIT crate 用）
- `release.sh`: リリースタグ打ちとバージョン bump（Phase 1c のリリース時に使用）

1 スクリプト 200 行以内を目標とし、それを超える場合は Go または Rust ツール化して `tools/` に移す（300 行制限の精神を scripts にも適用）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、DX-LD-\*。**上流**: DS-IMPL-DIR-009、DS-DEVX-CICD-\*。

## DS-IMPL-DIR-188 scripts の shebang と実行権限

全 `.sh` ファイルは冒頭に以下を必ず含める。

```bash
#!/usr/bin/env bash
# <日本語のファイル説明コメント>
set -Eeuo pipefail
```

`set -Eeuo pipefail` を先頭に置くことで「エラー即停止 / 未定義変数 NG / pipe 失敗の伝搬」を強制する。実行権限（`chmod +x`）は git に記録し（`git update-index --chmod=+x`）、`ls -l scripts/` で全ファイルが `-rwxr-xr-x` であることを確認する。

Windows 開発者への配慮として、`.gitattributes` で `*.sh text eol=lf` を設定済み（[DS-IMPL-DIR-016](01_リポジトリルート構成.md)）。改行コード差異による実行失敗を防ぐ。

**確定フェーズ**: Phase 1a。**対応要件**: DX-LD-\*、NFR-F-ENV-\*。**上流**: DS-IMPL-DIR-009、DS-IMPL-DIR-016。

## .github/ 配下の詳細配置

`.github/` の完全な構造は以下のとおり。

```
.github/
├── CODEOWNERS                            # 所有権割当（root からの参照も可）
├── PULL_REQUEST_TEMPLATE.md              # PR テンプレート
├── ISSUE_TEMPLATE/
│   ├── bug_report.md
│   ├── feature_request.md
│   └── design_proposal.md                # ADR 起票の issue テンプレ
├── workflows/
│   ├── pr.yml                            # PR でトリガー（unit + lint + breaking）
│   ├── main.yml                          # main push でトリガー（integration + image build）
│   ├── release.yml                       # release タグでトリガー（e2e + contract + publish）
│   ├── nightly.yml                       # cron でトリガー（load + bench + security scan）
│   └── reusable/                         # 再利用ワークフロー
│       ├── go-test.yml
│       ├── rust-test.yml
│       ├── helm-lint.yml
│       └── container-scan.yml
├── actions/                              # カスタムアクション（composite）
│   ├── setup-toolchain/
│   │   └── action.yml
│   └── buf-generate/
│       └── action.yml
└── dependabot.yml                        # Dependabot 設定
```

`workflows/` の 4 系統分離は、CI コスト・実行時間・failure の切り分け容易性の 3 軸で最適化する。`reusable/` と `actions/` の分離は、「再利用ワークフロー（ジョブ単位）」と「カスタムアクション（ステップ単位）」の粒度差を反映する。

## DS-IMPL-DIR-189 workflows の 4 系統分割

CI ワークフローは以下 4 系統に分ける。

- `pr.yml`: PR の各 push でトリガー。目的は「即時 feedback」。実行時間 10 分以内を目標
- `main.yml`: main ブランチへの push でトリガー。目的は「integration 検証 + image 生成」。実行時間 30 分以内
- `release.yml`: `v*` タグ push でトリガー。目的は「E2E / 契約検証 + publish」。実行時間 60 分以内
- `nightly.yml`: `cron: '0 17 * * *'`（UTC 17:00 = JST 02:00）でトリガー。目的は「load + bench + security scan」。実行時間 120 分以内

4 系統の合計ジョブ数は Phase 1a 時点で 25 前後と見積もる。ジョブ間で共通するステップ（Go / Rust toolchain セットアップ、buf generate）は `reusable/` と `actions/` に切り出し、重複を排除する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-001。**上流**: DS-DEVX-CICD-\*、DS-IMPL-DIR-011、DS-IMPL-DIR-177（テスト配置対応）。

## DS-IMPL-DIR-190 pr.yml のジョブ構成

`pr.yml` は PR 1 回あたり以下ジョブを並列実行する。

1. `lint-go`: `golangci-lint run ./src/tier1/go/...` + `go-cleanarch -config src/tier1/go/.cleanarch.yaml`（03 章 DS-IMPL-DIR-074 の `cmd → pods → shared` 層分離と Pod 間 import 禁止を強制）
2. `lint-rust`: `cargo clippy --workspace -- -D warnings` + `cargo fmt --check`
3. `lint-proto`: `buf lint` + `buf breaking --against '.git#branch=main,subdir=src/tier1/contracts'`
4. `lint-helm`: `helm lint src/tier1/infra/delivery/helm/*`
5. `lint-shell`: `shellcheck scripts/*.sh`
6. `check-deps`: `tools/check-deps/` による cross-language 層間依存検証
7. `check-rust-deps`: `tools/check-rust-deps/` による Rust path 依存の検証（04 章 DS-IMPL-DIR-114 の `pods → shared/sdk` のみ許容・`crates/pods/` 間の直接依存禁止）
8. `test-go-unit`: `go test -short ./src/tier1/go/...`
9. `test-rust-unit`: `cargo test --workspace --lib`
10. `build-go`: `go build ./src/tier1/go/cmd/...`（artifacts は捨てる）
11. `build-rust`: `cargo build --workspace`（artifacts は捨てる）
12. `docs-lint`: markdown lint + textlint（日本語文体チェック）
13. `lint-k8s`: `conftest test src/tier1/infra/governance/policies/ --policy tests/policy/`（05 章 DS-IMPL-DIR-135〜137 の Kyverno/PSS 設定の形式検査）

全ジョブが並列実行され、最長ジョブで完了時間が決まる。10 分以内の目標は test-rust-unit（現状 2 〜 3 分）と build-rust（5 〜 7 分）がボトルネックになるため、`sccache` と GitHub Actions cache を活用する。ジョブ数が 11 → 13 に増えたことで runner の同時実行枠が逼迫した場合は、`lint-k8s` を `reusable/` に切り出して並列 matrix の外に出すことで総時間への影響を抑える。

**critical / optional レーン分類と四半期生存判定**: CI ジョブは単に数を増やすほど merge-block の発生率が線形に増え、2 名運用（NFR-C-NOP-001）下では「CI が通らないから別経路で merge する」副経路の生成源になる。これを防ぐため、13 ジョブを**必須（critical）レーン**と**警告（optional）レーン**の 2 レーンに明示分類し、optional 違反は merge-block せず weekly digest で可視化するだけにとどめる。分類と根拠は次のとおり。

- **critical レーン（8 ジョブ、merge-block）**: `lint-go` / `lint-rust` / `lint-proto` / `check-deps` / `check-rust-deps` / `build-go` / `build-rust` / `test-go-unit`。根拠は「これらが fail した状態の main ブランチは tier2 / tier3 へ伝搬すると即時に業務影響を起こす」点。特に `lint-proto`（buf breaking）・`check-deps`（layer 逸脱）・`check-rust-deps`（Pod 間横断）は tier1 契約境界を破壊する経路で、ratchet 運用を許容しない。
- **optional レーン（5 ジョブ、merge 許容 + 警告）**: `lint-helm` / `lint-shell` / `lint-k8s` / `docs-lint` / `test-rust-unit`。根拠は「これらの fail は Phase 1a 時点で業務影響が限定的で、後続 `main.yml` / `nightly.yml` で再検出可能」点。optional ジョブ fail 時は PR に `⚠️ optional-lint-fail: <job>` ラベルを自動付与し、週次で `@k1s0/devex-team` に digest 通知（後述の生存判定の一次入力）。

**Phase 1b 以降で追加される lint の初期扱い**（`check-proto-symmetry` / `check-dapr-components` / `check-values-sot` / `bff-boundary-lint` / `check-file-size` / `template-drift-detector` / `check-ds-mapping` の 7 種）は、**導入から 1 四半期は optional レーンに置く**。運用データ（週次 fail 率 / false positive 率 / 対処時間）を収集したうえで、四半期末に critical へ昇格するかを判断する。false positive 率 5% 超または平均対処時間 30 分超の lint は critical 昇格を見送る。

**四半期生存判定（quarterly lint review）**: 全 lint ジョブは四半期に 1 回、`@k1s0/devex-team` と `@k1s0/tier1-architects` が以下 4 指標で生存判定する。

1. **起動率**: 過去 90 日の PR のうち当該ジョブが実際に走った PR の比率（10% 未満 = 動いていない疑い）
2. **fail 率**: 過去 90 日の fail 数 / 実行数（60% 超 = signal として破綻、0% 近傍 = 意味のない lint の疑い）
3. **false positive 率**: fail 中「lint のバグ」として PR コメントで扱われた件数比率（5% 超 = 保守責任の再割当）
4. **平均対処時間**: fail 発生から該当 PR が再び pass するまでの中央値（30 分超 = 開発体験を壊す候補）

判定結果は `docs/05_実装/00_ディレクトリ設計/` 配下に追加しない（運用データは `docs/` に commit しない）。代わりに Backstage TechDocs の生成時パイプラインから四半期レポートを生成し、`@k1s0/tier1-architects` に回付する。判定で「廃止」となった lint は ADR を 1 本起票して `tools/<name>/` ディレクトリごと git から削除する。死骸化した lint を CI に残すことを禁止する（副経路生成源になる）。

**保守オーナーの明示**: 各 lint ツール（`tools/<name>/`）には `OWNERS` ファイルを 1 行で配置し、primary owner（GitHub team）と secondary owner を宣言する。オーナー未宣言の lint は CI に組み込めない（PR レビューで reject）。オーナーが異動・退職した時点で四半期判定を待たずに生存判定を繰り上げる。

**確定フェーズ**: Phase 1a（レーン分類 + 8 ジョブ critical 指定）、Phase 1b（追加 lint 群の optional 投入と初回四半期判定）、毎四半期（生存判定の継続実施）。**対応要件**: DX-CICD-\*、DX-TEST-001、NFR-C-NOP-001、NFR-C-NOP-002。**上流**: DS-DEVX-CICD-\*、DS-IMPL-DIR-177、DS-IMPL-DIR-200（変更手続）。

## DS-IMPL-DIR-191 main.yml のジョブ構成

`main.yml` は main ブランチへの push 時に以下を実行する。pr.yml の全ジョブに加えて、以下が追加される。

1. `test-go-integration`: `go test -tags=integration ./src/tier1/go/integrationtest/...`
2. `test-rust-integration`: `cargo test --workspace --tests`（crate tests/）
3. `build-image`: 6 Pod の container image をビルドしレジストリへプッシュ（tag: `main-<sha>`）。Phase 1a は `ghcr.io/k1s0/`、Phase 1b 以降は placeholder `harbor.k1s0.internal/tier1/`（実 FQDN は [08 章 DS-IMPL-DIR-213](08_命名規約と配置ルール.md) 参照）。
4. `sign-image`: cosign による署名（`COSIGN_EXPERIMENTAL=1`）
5. `sbom-generate`: syft による SBOM 生成、プッシュ
6. `scan-image`: trivy / grype による脆弱性スキャン
7. `update-gitops`: `k1s0-gitops` repo に image tag 更新の PR を自動作成（dev 環境向け）

build-image は 6 並列で実行（facade 3 Pod + rust 3 Pod）。レジストリへの push は Phase 1a では GitHub Actions の `GITHUB_TOKEN` によるトークン認証（ghcr.io）、Phase 1b 以降は robot account + mTLS（Harbor）で行う。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-E-NW-\*、NFR-H-INT-\*。**上流**: DS-DEVX-CICD-\*、DS-IMPL-DIR-060（Go image）、DS-IMPL-DIR-100（Rust image）、DS-IMPL-DIR-157（image tag placeholder）。

## DS-IMPL-DIR-192 release.yml のジョブ構成

`release.yml` は `v1.0.0-phase1a` のようなタグ push でトリガーされる。main.yml の全ジョブに加えて以下を実行する。

1. `test-e2e`: `tests/e2e/scenarios/` 全シナリオを kind クラスタで実行
2. `test-contract`: `tests/contract/providers/` の Pact 検証
3. `publish-image`: image を `v1.0.0-phase1a` タグでリリースレジストリに再 push（Phase 1a は ghcr.io、Phase 1b 以降は placeholder `harbor.k1s0.internal`）
4. `publish-chart`: Helm Chart を `tier1-umbrella-1.0.0-phase1a.tgz` として OCI registry（Phase 1a: ghcr.io、Phase 1b 以降: Harbor）に push
5. `publish-gitops-staging`: `k1s0-gitops` repo に staging 環境向け PR を作成
6. `create-github-release`: GitHub Release を作成（リリースノート自動生成）

E2E は 30 分以内の予算（[DS-IMPL-DIR-179](06_テストとフィクスチャ配置.md)）内に収める。タグの命名規則（`vX.Y.Z-phaseN`）は 08 章で詳述。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、DX-TEST-003、DX-TEST-004。**上流**: DS-DEVX-CICD-\*、DS-IMPL-DIR-177、DS-IMPL-DIR-179。

## DS-IMPL-DIR-193 nightly.yml のジョブ構成

`nightly.yml` は JST 02:00（UTC 17:00）に cron でトリガーされる。

1. `test-load-k6`: `tests/load/k6/` の全スクリプトを実行し、結果を Prometheus remote-write
2. `bench-rust`: `cargo bench --workspace` を実行し、criterion 結果を保存
3. `security-scan`: trivy による image 再スキャン（CVE データベース更新後）
4. `license-scan`: `cargo deny check licenses` + `go-licenses check` による OSS ライセンス逸脱検出
5. `soak-test`: `tests/load/locust/soak.py` を 60 分実行（Phase 1c 以降有効化）

失敗時は Slack `#k1s0-ci-alerts` に通知。benchmark の回帰検出閾値は前夜比 10% 以上の劣化とする（false positive を抑制）。

**確定フェーズ**: Phase 1a（k6 + security）、Phase 1b（bench）、Phase 1c（soak）。**対応要件**: NFR-B-PERF-\*、NFR-E-AC-\*、NFR-H-INT-\*、BC-LICENSE-\*。**上流**: DS-DEVX-CICD-\*、DS-IMPL-DIR-168（load results）。

## DS-IMPL-DIR-194 reusable workflows と custom actions の分離

`workflows/reusable/` と `actions/` の責務分離は以下。

- `workflows/reusable/*.yml`: **ジョブ単位**の再利用。複数 runner を使う可能性のある処理（例: `go-test.yml` は OS matrix を内部に持つ）
- `actions/*/action.yml`: **ステップ単位**の再利用。単一 runner 内の composite action（例: `setup-toolchain/action.yml` は mise install + Go/Rust setup を 1 ステップにまとめる）

4 系統ワークフローから呼ばれる共通処理は必ずどちらかに切り出し、直接コピペしない。呼び出し側の記述は 3 行以内に収まることを目標とする。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*。**上流**: DS-DEVX-CICD-\*。

## DS-IMPL-DIR-195 dependabot.yml の更新頻度

`.github/dependabot.yml` は依存自動更新の設定。対象は 4 種類。

- Go module（`src/tier1/go/go.mod`、`tools/check-deps/go.mod`、`tools/mock-server/go.mod`、`tests/e2e/go.mod`、`tests/contract/go.mod`）。Go は tier1 側が単一 `go.mod` 構成であり `internal/pods/` / `internal/shared/` の分割は Dependabot の対象単位に影響しない
- Cargo（`src/tier1/rust/Cargo.toml` の workspace 直下、`tools/proto-gen/Cargo.toml`、`tools/check-rust-deps/Cargo.toml`、`tools/k1s0-cli/Cargo.toml`）。Rust workspace は単一 `Cargo.toml` でメンバー `crates/pods/*` / `crates/shared/*` / `crates/sdk/*` を一括解決するため、個別 crate ごとに manifest 対象を追加する必要はない
- GitHub Actions（`.github/workflows/*.yml`、`.github/actions/*/action.yml`）
- Docker（`.devcontainer/Dockerfile`、`src/tier1/go/build/Dockerfile.*`、`src/tier1/rust/build/Dockerfile.*`）

更新頻度は全て `weekly`（月曜日 JST 09:00）とし、大量の PR が同時に届かないよう分散させる。security update は `weekly` を無視し即時通知。OSS Supply chain セキュリティ（BC-COMPLIANCE-\*）の主要な実装手段となる。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-H-INT-\*、BC-COMPLIANCE-\*、NFR-E-AC-\*。**上流**: DS-DEVX-CICD-\*。

## .devcontainer/ 配下の詳細配置

`.devcontainer/` の完全な構造は以下のとおり。

```
.devcontainer/
├── devcontainer.json                     # メイン設定
├── Dockerfile                            # ベースイメージ + toolchain
├── docker-compose.yml                    # Dev Container 起動時に併用するバックエンド
└── post-create.sh                        # Container 初回作成時のセットアップ
```

Dev Container は VS Code / IntelliJ の両方で動作し、OS 差異（macOS / Linux / Windows + WSL2）を吸収する。Phase 1a の全開発者は Dev Container 利用を推奨する。

## DS-IMPL-DIR-196 devcontainer.json の設定項目

`.devcontainer/devcontainer.json` は以下の項目を必ず含む。

- `name`: `k1s0-dev`
- `build.dockerfile`: `Dockerfile`
- `features`: `ghcr.io/devcontainers/features/docker-outside-of-docker` など
- `customizations.vscode.extensions`: Go / rust-analyzer / Protobuf / Helm / YAML / Markdown
- `customizations.vscode.settings`: Go / Rust / Markdown の format on save、golangci-lint 連携
- `postCreateCommand`: `./post-create.sh`
- `mounts`: ホスト側 `~/.ssh`、`~/.gitconfig` を read-only でマウント
- `forwardPorts`: 3000（Grafana）、8080（tier1 API）、50051（gRPC）、5432（Postgres）
- `remoteUser`: `vscode`（非 root）

設定は JSON Schema `https://raw.githubusercontent.com/devcontainers/spec/main/schemas/devContainer.schema.json` に準拠し、IDE 側の補完で誤記を検出できる。

**確定フェーズ**: Phase 1a。**対応要件**: DX-LD-\*、DX-DEVEX-\*、NFR-F-ENV-\*。**上流**: DS-DEVX-LOCAL-\*、DS-IMPL-DIR-011。

## DS-IMPL-DIR-197 .devcontainer/Dockerfile の構成

`.devcontainer/Dockerfile` のベースは `mcr.microsoft.com/devcontainers/base:ubuntu-22.04` とする。

インストールする toolchain と版は `.mise.toml`（[DS-IMPL-DIR-017](01_リポジトリルート構成.md)）と同期させる。

- mise（toolchain マネージャ）
- Go（`.mise.toml` の `go` に従う）
- Rust（`.mise.toml` の `rust` に従う、`rust-toolchain.toml` と整合）
- Node（Phase 1c 以降の Backstage 用）
- Python（Locust 用）
- buf CLI、kubectl、helm、kind、k3d
- cosign、syft、trivy
- shellcheck、textlint、markdownlint
- docker-ce-cli（Docker-outside-of-Docker 用、daemon は不要）

ベースイメージは Phase 1a 時点で 2GB 以内に抑える。超過時は後述の Dockerfile.slim を Phase 1b で追加する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-LD-\*、NFR-F-ENV-\*。**上流**: DS-DEVX-LOCAL-\*、DS-IMPL-DIR-017。

## DS-IMPL-DIR-198 .devcontainer/docker-compose.yml の位置づけ

`.devcontainer/docker-compose.yml` は Dev Container 起動時に同時起動する開発用バックエンドを定義する。Phase 1a では以下 3 つ。

- `postgres:16`: AUDIT / WORKFLOW のローカル DB（testcontainers-go が使うのとは別）
- `valkey:7`: STATE のキャッシュ
- `openbao:2.0`: SECRET のテスト用

本 compose は **ローカル開発専用**で、本番 / CI では使わない。Helm Chart（`src/tier1/infra/delivery/helm/`）および Operator / backend 定義（`src/tier1/infra/platform/operators/` / `src/tier1/infra/platform/backends/`）と設定値は別管理。Dev Container 終了時はコンテナも停止し、ホスト側のリソースを専有しない。

**確定フェーズ**: Phase 1a。**対応要件**: DX-LD-\*。**上流**: DS-DEVX-LOCAL-\*、DS-IMPL-DIR-143〜150（backend operators）。

## DS-IMPL-DIR-199 CODEOWNERS の詳細割当

`CODEOWNERS` は [DS-IMPL-DIR-018](01_リポジトリルート構成.md) で canonical な 8 チームを定めた（`@k1s0/tier1-architects` / `@k1s0/product-owners` / `@k1s0/api-leads` / `@k1s0/tier1-go-team` / `@k1s0/tier1-rust-team` / `@k1s0/infra-team` / `@k1s0/devex-team` / `@k1s0/security-team`）。本章はその 8 チームだけを使い、03 / 04 / 05 章で深化した `pods/ vs shared/`（Go・Rust）および `delivery / runtime / governance / platform`（infra）の 2 軸分離を CODEOWNERS の path filter に忠実に投影する。

CODEOWNERS の設計原則は次の 3 つである。第一に、Pod / crate ディレクトリ（高頻度変更・Pod 自走）は言語チームのみを所有者に置き、アーキテクトを強制的に巻き込まない。第二に、`shared/` ディレクトリ（低頻度変更・影響範囲広大）はアーキテクトを必須レビューに固定し、水平方向の不整合を防ぐ。第三に、セキュリティ・契約・法令に関わる領域（SECRET / AUDIT / PII / Kyverno policies / Protobuf contracts / SDK）は所管チームに加えて `@k1s0/security-team` または `@k1s0/api-leads` を AND 承認の対象に加える。AND 条件は CODEOWNERS の記法では表現できないため、GitHub branch protection の required reviewer ルールで追加指定する。

```
# root 全般（default owner）
*                                  @k1s0/tier1-architects

# ドキュメント群（フェーズで所有者が遷移する）
/docs/01_企画/                     @k1s0/product-owners
/docs/02_構想設計/                 @k1s0/tier1-architects
/docs/03_要件定義/                 @k1s0/tier1-architects @k1s0/product-owners
/docs/04_概要設計/                 @k1s0/tier1-architects
/docs/05_実装/                     @k1s0/tier1-architects @k1s0/devex-team

# Protobuf 契約（API リード + アーキテクト両承認。branch protection で AND 強制）
/src/tier1/contracts/              @k1s0/tier1-architects @k1s0/api-leads

# Go ファサード層：Pod 別（高頻度・自走）
/src/tier1/go/internal/pods/state/        @k1s0/tier1-go-team
/src/tier1/go/internal/pods/secret/       @k1s0/tier1-go-team @k1s0/security-team
/src/tier1/go/internal/pods/workflow/     @k1s0/tier1-go-team

# Go 共有層：shared/（低頻度・横断影響）。アーキテクト必須
/src/tier1/go/internal/shared/dapr/       @k1s0/tier1-go-team @k1s0/tier1-architects
/src/tier1/go/internal/shared/common/     @k1s0/tier1-go-team @k1s0/tier1-architects
/src/tier1/go/internal/shared/policy/     @k1s0/tier1-go-team @k1s0/security-team
/src/tier1/go/internal/shared/otel/       @k1s0/tier1-go-team @k1s0/infra-team
/src/tier1/go/internal/shared/log/        @k1s0/tier1-go-team @k1s0/infra-team
/src/tier1/go/internal/shared/metrics/    @k1s0/tier1-go-team @k1s0/infra-team
/src/tier1/go/internal/shared/flagd/      @k1s0/tier1-go-team @k1s0/devex-team
/src/tier1/go/internal/shared/proto/      @k1s0/tier1-go-team @k1s0/api-leads

# Rust pods 層：crate 別（高頻度・自走）
/src/tier1/rust/crates/pods/audit/        @k1s0/tier1-rust-team @k1s0/security-team
/src/tier1/rust/crates/pods/decision/     @k1s0/tier1-rust-team
/src/tier1/rust/crates/pods/pii/          @k1s0/tier1-rust-team @k1s0/security-team

# Rust shared 層：横断 crate。アーキテクト必須
/src/tier1/rust/crates/shared/common/     @k1s0/tier1-rust-team @k1s0/tier1-architects
/src/tier1/rust/crates/shared/proto-gen/  @k1s0/tier1-rust-team @k1s0/api-leads
/src/tier1/rust/crates/shared/otel-util/  @k1s0/tier1-rust-team @k1s0/infra-team
/src/tier1/rust/crates/shared/policy/     @k1s0/tier1-rust-team @k1s0/security-team

# Rust SDK 層：外部公開 API の互換性。API リード + アーキテクト AND
/src/tier1/rust/crates/sdk/               @k1s0/tier1-rust-team @k1s0/tier1-architects @k1s0/api-leads

# infra：delivery（Helm / Kustomize / Rollouts）は release と結合しやすいため go/rust 両チームも関与
/src/tier1/infra/delivery/helm/tier1-facade/    @k1s0/infra-team @k1s0/tier1-go-team
/src/tier1/infra/delivery/helm/tier1-rust/      @k1s0/infra-team @k1s0/tier1-rust-team
/src/tier1/infra/delivery/helm/tier1-umbrella/  @k1s0/infra-team @k1s0/tier1-architects
/src/tier1/infra/delivery/kustomize/            @k1s0/infra-team
/src/tier1/infra/delivery/rollouts/             @k1s0/infra-team @k1s0/tier1-architects

# infra：runtime（Dapr Component / Configuration / Subscription）は Dapr 契約に直結
/src/tier1/infra/runtime/components/            @k1s0/infra-team @k1s0/tier1-architects
/src/tier1/infra/runtime/configuration/         @k1s0/infra-team @k1s0/tier1-architects
/src/tier1/infra/runtime/subscriptions/         @k1s0/infra-team @k1s0/api-leads

# infra：governance（Kyverno / Namespace / PSS）はセキュリティ主管
/src/tier1/infra/governance/policies/           @k1s0/security-team @k1s0/infra-team
/src/tier1/infra/governance/namespaces/         @k1s0/infra-team @k1s0/security-team

# infra：platform（Operator / backends）は infra-team 主管
/src/tier1/infra/platform/operators/            @k1s0/infra-team
/src/tier1/infra/platform/backends/             @k1s0/infra-team

# tests（粒度別）
/tests/e2e/framework/              @k1s0/devex-team @k1s0/tier1-architects
/tests/e2e/scenarios/state/        @k1s0/tier1-go-team
/tests/e2e/scenarios/secret/       @k1s0/tier1-go-team @k1s0/security-team
/tests/e2e/scenarios/workflow/     @k1s0/tier1-go-team
/tests/e2e/scenarios/audit/        @k1s0/tier1-rust-team @k1s0/security-team
/tests/e2e/scenarios/decision/     @k1s0/tier1-rust-team
/tests/e2e/scenarios/pii/          @k1s0/tier1-rust-team @k1s0/security-team
/tests/e2e/scenarios/golden_path*  @k1s0/tier1-architects
/tests/contract/                   @k1s0/api-leads @k1s0/tier1-architects
/tests/load/                       @k1s0/infra-team
/tests/fixtures/                   @k1s0/devex-team @k1s0/security-team
/tests/golden/                     @k1s0/api-leads

# tools / scripts（Backstage Software Template は tier 粒度で所有者を分ける）
/tools/proto-gen/                              @k1s0/devex-team @k1s0/api-leads
/tools/check-deps/                             @k1s0/devex-team @k1s0/tier1-architects
/tools/check-rust-deps/                        @k1s0/devex-team @k1s0/tier1-rust-team
/tools/k1s0-cli/                               @k1s0/devex-team
/tools/mock-server/                            @k1s0/devex-team @k1s0/api-leads
/tools/backstage/                              @k1s0/devex-team
/tools/backstage-templates/tier2/go/           @k1s0/devex-team @k1s0/tier1-go-team
/tools/backstage-templates/tier2/csharp/       @k1s0/devex-team @k1s0/tier1-architects
/tools/backstage-templates/tier3/              @k1s0/devex-team @k1s0/tier1-architects
/tools/                                        @k1s0/devex-team
/scripts/                                      @k1s0/devex-team

# .github / .devcontainer
/.github/workflows/                @k1s0/devex-team @k1s0/tier1-architects
/.github/CODEOWNERS                @k1s0/tier1-architects
/.devcontainer/                    @k1s0/devex-team

# root 設定ファイル
/CLAUDE.md                         @k1s0/tier1-architects
/LICENSE                           @k1s0/tier1-architects
/.mise.toml                        @k1s0/devex-team
/.gitattributes                    @k1s0/devex-team
/.editorconfig                     @k1s0/devex-team
```

CODEOWNERS は GitHub 側の branch protection で「少なくとも 1 人の owner レビュー必須」と組み合わせる。複数チームが列挙されている場合の意味は OR 条件（どちらか 1 チームの承認で通る）。AND 条件が必要な箇所（`contracts/` / `crates/sdk/` / `pods/secret/` / `pods/audit/` / `pods/pii/` / `governance/policies/` 等）は、GitHub ブランチ保護設定で個別の required reviewer ルールを追加する（CODEOWNERS 単体では AND を表現できないため）。`LICENSE` は 01 章で法務レビューを外部プロセス扱いとしたので、CODEOWNERS は `@k1s0/tier1-architects` のみを置き、法務合意は PR 本文のチェックリストで担保する。

path 順序に依存する注意点として、GitHub の CODEOWNERS は「最後にマッチした行が有効」という評価規則を持つ。したがって `*` を先頭の default に置き、深いパスを後ろに並べる現在の並び順を維持すること。`tools/` 配下で個別のツール行（例: `/tools/backstage-templates/tier2/go/`）を先に置き、最後に `/tools/` 全体の fallback を置く順序にすると、個別指定が全体指定で上書きされ devex-team 単独になってしまう。そのため上の並びでは個別指定を最後に置くか、または `/tools/` 全体の fallback を個別指定の前に配置する必要がある。本書は後者（fallback 先置き）を採用したので、CODEOWNERS 編集時は個別 tools 行を `/tools/` 全体行よりも後に保つこと。評価順の検証は Phase 1b 追加予定の `tools/check-deps/internal/rules/codeowners.go` で自動化する。

**確定フェーズ**: Phase 1a（実チーム名は 01 章 DS-IMPL-DIR-018 と同じく Phase 0 仮置き、Phase 1a で GitHub Team にバインド）。**対応要件**: NFR-C-NOP-001、DX-CICD-\*、NFR-H-INT-\*。**上流**: DS-IMPL-DIR-018。

## DS-IMPL-DIR-200 CI/CD 配置変更の ADR 起票条件

以下の変更は ADR 起票を要する。

1. ワークフロー 4 系統分割（pr / main / release / nightly）の変更
2. 新規ツールの `tools/<name>/` 追加（Phase 1a 時点の 6 種＝`proto-gen` / `check-deps` / `check-rust-deps` / `k1s0-cli` / `mock-server` / `backstage-templates` 以外、および Phase 1c で追加する `backstage`）
3. `tools/backstage-templates/` 配下の tier / 変種の新設（例: `tier2/python/` 追加、`tier3/desktop-tauri/` 追加）および template.yaml の breaking change（parameters 削除・rename）
4. Go `internal/pods/` / `internal/shared/` の第 1 階層カテゴリ変更、Rust `crates/pods/` / `crates/shared/` / `crates/sdk/` の 3 分類変更、または infra `delivery / runtime / governance / platform` の 4 カテゴリ変更に伴う CI path filter の書き換え
5. Dev Container のベースイメージ変更（ubuntu → alpine 等）
6. Dependabot の対象言語追加
7. CODEOWNERS のチーム構成変更（新規チーム追加・既存チーム統廃合）および `shared/` / `governance/` / `sdk/` の AND 必須化の解除

軽微な変更（workflow 内のステップ追加、actions の version bump、CODEOWNERS の 1 行追加で Pod / crate 単独の所有者を調整するもの）は ADR 不要だが、`devex-team` と `tier1-architects` の両チームのレビューを要する。

**確定フェーズ**: Phase 0（ルール）、各 Phase（適用）。**対応要件**: NFR-C-NOP-001、DX-CICD-\*。**上流**: DS-SW-COMP-138、DS-IMPL-DIR-018。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定フェーズ |
|---|---|---|
| DS-IMPL-DIR-181 | tools/proto-gen/ の配置 | Phase 1a |
| DS-IMPL-DIR-182 | tools/check-deps/ の配置 | Phase 1a |
| DS-IMPL-DIR-183 | tools/k1s0-cli/ のコマンド配置 | Phase 1a/1b |
| DS-IMPL-DIR-184 | tools/mock-server/ の配置 | Phase 1b |
| DS-IMPL-DIR-185 | tools/backstage/ と tools/backstage-templates/ の配置 | Phase 1b/1c |
| DS-IMPL-DIR-186 | scripts/bootstrap.sh の責務 | Phase 1a |
| DS-IMPL-DIR-187 | scripts/ 各スクリプトの責務一覧 | Phase 1a |
| DS-IMPL-DIR-188 | scripts の shebang と実行権限 | Phase 1a |
| DS-IMPL-DIR-189 | workflows の 4 系統分割 | Phase 1a |
| DS-IMPL-DIR-190 | pr.yml のジョブ構成 | Phase 1a |
| DS-IMPL-DIR-191 | main.yml のジョブ構成 | Phase 1a |
| DS-IMPL-DIR-192 | release.yml のジョブ構成 | Phase 1a |
| DS-IMPL-DIR-193 | nightly.yml のジョブ構成 | Phase 1a〜1c |
| DS-IMPL-DIR-194 | reusable workflows と custom actions の分離 | Phase 1a |
| DS-IMPL-DIR-195 | dependabot.yml の更新頻度 | Phase 1a |
| DS-IMPL-DIR-196 | devcontainer.json の設定項目 | Phase 1a |
| DS-IMPL-DIR-197 | .devcontainer/Dockerfile の構成 | Phase 1a |
| DS-IMPL-DIR-198 | .devcontainer/docker-compose.yml の位置づけ | Phase 1a |
| DS-IMPL-DIR-199 | CODEOWNERS の詳細割当 | Phase 1a |
| DS-IMPL-DIR-200 | CI/CD 配置変更の ADR 起票条件 | Phase 0 |

### 対応要件一覧

- DX-CICD-\*（CI/CD 全般）、DX-LD-\*（ローカル開発）、DX-DEVEX-\*、DX-TEST-\*
- NFR-C-NOP-001（2 名運用）、NFR-E-AC-\*、NFR-E-NW-\*、NFR-F-ENV-\*、NFR-H-INT-\*（完整性）、NFR-SUP-\*
- BC-COMPLIANCE-\*（OSS Supply chain）、BC-LICENSE-\*

### 上流設計 ID

DS-SW-COMP-113（依存検査）、DS-SW-COMP-138（変更手続）、DS-DEVX-CICD-\*（CI/CD 方式）、DS-DEVX-LOCAL-\*（ローカル開発方式）、DS-DEVX-DOC-\*、DS-IMPL-DIR-007〜013（01 章ルート構成）、DS-IMPL-DIR-016〜019（root 設定）、DS-IMPL-DIR-021（contracts）、DS-IMPL-DIR-060（Go image）、DS-IMPL-DIR-100（Rust image）、DS-IMPL-DIR-143〜150（backend operators）、DS-IMPL-DIR-157（image tag）、DS-IMPL-DIR-168（load results）、DS-IMPL-DIR-169（fixtures）、DS-IMPL-DIR-177（テスト配置対応）、DS-IMPL-DIR-179（テスト時間予算）。
