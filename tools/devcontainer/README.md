# Dev Container 実装資産

本ディレクトリは k1s0 開発で利用する Dev Container の実装資産（Dockerfile / postCreate / 役割別プロファイル）を保持する。設計書は [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md) に固定されており、本 README はその実装段階の進捗とフェーズ間の差分を宣言する。

## 配置

```text
tools/devcontainer/
├── README.md                    # 本ファイル: 実装ステータスと差分宣言
├── postCreate.sh                # 共通 postCreate (toolchain 検証 + cone verify + local-stack 起動)
├── doctor.sh                    # 役別 toolchain 診断（CI / 開発者ローカル両対応）
├── base/                        # 未配置: 共通最小ベース image (debian slim 由来) は次フェーズ
└── profiles/                    # 10 役プロファイル全実体化済み (Phase 2)
    ├── tier1-rust-dev/          # Rust Edition 2024 / mold / cargo-nextest
    ├── tier1-go-dev/            # Go 1.24 + Dapr Go SDK + delve
    ├── tier2-dev/               # .NET 8 + Go + Temporal CLI
    ├── tier3-web-dev/           # Node 22 + pnpm + Playwright
    ├── tier3-native-dev/        # .NET 8 + MAUI + Android SDK
    ├── platform-cli-dev/        # Rust + Node 22 (Backstage プラグイン用) + cosign + syft
    ├── sdk-dev/                 # Rust + Go + .NET + Node を最小束ね
    ├── infra-ops/               # kubectl / helm / argocd / istioctl / kyverno / opentofu / k6
    ├── docs-writer/             # drawio + markdownlint + textlint + Pandoc + Mermaid CLI
    └── full/                    # 全機能 (CI 専用、日常開発非推奨)
```

リポジトリルートの [`.devcontainer/devcontainer.json`](../../.devcontainer/devcontainer.json) は最軽量の `docs-writer` プロファイルへの相対 symlink になっており、`tools/sparse/checkout-role.sh <role>` でリンク先を切替える。

## Phase 2 の到達状態

設計書（10_DevContainer_10役）の正典が定義する 10 役は全実体化されており、Paved Road（ADR-DEV-001）の「正しい道一本化」が各役割で並走可能になった。Phase 1 bootstrap で「tier1 中心の唯一の道」に縮約していた状態からの移行は完了し、`.devcontainer/settings/`・`.sparse-checkout/roles/`・`tools/sparse/checkout-role.sh`・`tools/local-stack/` の 4 系統が同時に揃った時点を Phase 2 到達と定義する。

各プロファイルは Microsoft 公式 base image（`mcr.microsoft.com/devcontainers/rust:1-bookworm` 等）を直接 `FROM` する。設計の正典は `tools/devcontainer/base/Dockerfile`（debian slim 由来）からの派生を要求するが、Platform/Build 体制が確立した時点で着手する次フェーズの作業として残している。

## ホスト docker.sock 経由の構成

[ADR-DEV-002](../../docs/02_構想設計/adr/ADR-DEV-002-windows-wsl2-docker-runtime.md)（WSL2 ネイティブ docker-ce）に従い、Dev Container 内から host docker.sock 経由で kind クラスタ・ローカルコンテナを操作する構成を採る。`devcontainer.json` の `docker-outside-of-docker` Feature が host の `/var/run/docker.sock` を bind mount するため、`docker` および `kind` コマンドは container から host docker daemon に対して直接動作する。Docker-in-Docker は採用しない（CI 用 `full` プロファイルでも将来は同方針）。

## 役割切替とローカル本番再現スタック

役割切替は [`tools/sparse/checkout-role.sh <role>`](../../tools/sparse/checkout-role.sh) を実行する。これは `.sparse-checkout/roles/<role>.txt` の cone を `git sparse-checkout set` で適用し、同時に `.devcontainer/devcontainer.json` を `tools/devcontainer/profiles/<role>/devcontainer.json` への相対 symlink に張り替える。複数役を兼任する場合は `-m` オプションでカンマ区切り指定する。`--verify` で現状の整合のみをチェックでき、`postCreate.sh` から自動的に呼ばれる。

ローカル本番再現スタックは [`tools/local-stack/`](../../tools/local-stack/) に配置されている。`tools/local-stack/up.sh --role <role>` で kind クラスタ（control-plane 1 + worker 3、Calico CNI）と、その上に Argo CD・cert-manager・MetalLB・Istio Ambient・Kyverno・SPIRE・Dapr operator・flagd・CNPG・Strimzi Kafka・MinIO・Valkey・OpenBao・Backstage・観測性スタック（Grafana/Loki/Tempo）・Keycloak が役割別に段階配備される。kind の `extraPortMappings` 経由で host の 30080（Argo CD）・30700（Backstage）・30300（Grafana）に直接アクセスできる。

`postCreate.sh` の末尾は `up.sh` を foreground で実行する。Dev Container 起動 〜 kind スタック完備までを 1 本のログで追跡できる代わりに、初回の rebuild は分単位で時間がかかる。背景化したい場合は `K1S0_LOCAL_STACK_BG=1` を `containerEnv` に追加すれば `.devcontainer/.log/local-stack-up.log` に流す経路に切替わる。完全スキップは `K1S0_SKIP_LOCAL_STACK=1`。

## 設計書と現状の差分

| 設計書の規定 | Phase 2 の現実装 | 解消予定フェーズ |
|---|---|---|
| 10 役プロファイルの並走 | 全 10 役配置済み（Dockerfile / devcontainer.json / extensions.json 揃い） | （達成済み） |
| 各プロファイルに `extensions.json` 個別配置 | 全プロファイルで分離配置済み | （達成済み） |
| `tools/sparse/checkout-role.sh` での役割切替 | 実装済み（symlink 張替え + cone set + 兼任マージ + --verify） | （達成済み） |
| postCreate での `tools/local-stack/up.sh` 呼び出し | foreground 既定で結線済み（`K1S0_LOCAL_STACK_BG=1` で背景化、`K1S0_SKIP_LOCAL_STACK=1` で完全スキップ） | （達成済み） |
| `base/Dockerfile` から派生 | Microsoft 公式 image から直接派生（base/ は未配置） | Phase 3（Platform/Build 体制確立後） |
| Dev Container image の GHCR digest 固定（IMP-DEV-DC-016） | ローカルビルドのみ（Renovate 連動なし） | Phase 3（Platform/Build 体制確立後） |
| `kind`/`k3d` 同時利用 | 本番再現スタックは kind 単独。`tools/local-stack/k3d-config.yaml` 未配置 | Phase 3（k3d 軽量経路の必要性が顕在化した時点で追加） |
| `examples/<role>-service/` の中身 | ディレクトリ自体未配置。10 役 cone は examples/ を参照しているため、役割切替後に当該ディレクトリは空のまま | Phase 3（tier1/tier2/tier3 の最初の実稼働サービスを起こす時点で配置） |
| `tools/codegen/buf/` `tools/codegen/openapi/` | ディレクトリ未配置（cone 参照対象として空） | Phase 3（最初の `.proto` / OpenAPI 仕様が確定した時点） |
| `tests/contract/` `tests/integration/` `tests/fuzz/rust/` `tests/golden/` | ディレクトリ未配置 | Phase 3（テスト基盤の最初のジョブを CI に乗せる時点） |

## toolchain 診断（doctor.sh）

`postCreate.sh` は環境構築の通過点として fail-soft でツール一覧を出すのみだが、`doctor.sh` は現状診断（fail-hard）として CI とローカル両方で利用する。

```bash
# 自動検出（symlink から role 判定）
./tools/devcontainer/doctor.sh

# 役を明示
./tools/devcontainer/doctor.sh tier1-rust-dev

# 全 10 役の必須／推奨ツール状態を一括確認（CI matrix で代替可能）
./tools/devcontainer/doctor.sh --all

# 機械可読出力（JSON）
./tools/devcontainer/doctor.sh --json --all
```

役ごとの必須／推奨ツールは `doctor.sh` 内の `REQUIRED` / `RECOMMENDED` / `MIN_VERSION` 配列で集中管理されており、設計書（[`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md)）の各役記述と整合する。

| 役 | 必須ツール | 主な推奨ツール | 最低バージョン |
|---|---|---|---|
| `docs-writer` | git, markdownlint-cli2 | textlint, mmdc, pandoc, drawio-export | — |
| `tier1-rust-dev` | git, rustc, cargo, buf, protoc | cargo-nextest, cargo-audit, cargo-deny, cargo-fuzz, mold | rustc ≥ 1.83 |
| `tier1-go-dev` | git, go, buf, protoc | dapr, golangci-lint, dlv | go ≥ 1.22 |
| `tier2-dev` | git, dotnet, go | temporal, buf | dotnet ≥ 8, go ≥ 1.22 |
| `tier3-web-dev` | git, node, pnpm | playwright | node ≥ 20, pnpm ≥ 9 |
| `tier3-native-dev` | git, dotnet | adb | dotnet ≥ 8 |
| `platform-cli-dev` | git, rustc, cargo, node | cosign, syft | rustc ≥ 1.83, node ≥ 20 |
| `sdk-dev` | git, rustc, cargo, go, dotnet, node, pnpm, buf | — | 各 LTS |
| `infra-ops` | git, kubectl, helm, kustomize | kind, argocd, istioctl, flagd, tofu, k6, dapr, cosign, syft | — |
| `full` | 上記の union | markdownlint-cli2 など | 各 LTS |

CI では各 role の Dev Container を立て、`doctor.sh <role>` を必須 status check として実行する。役固有の必須ツールが欠けた PR は CI で reject される（02-07 reusable workflow と統合予定）。

`postCreate.sh` との責務分離:

| スクリプト | 実行タイミング | 失敗時 | 役割 |
|---|---|---|---|
| `postCreate.sh` | Dev Container 起動直後（自動） | 警告のみ（fail-soft） | 環境構築の通過点。toolchain version 列挙、sparse 整合チェック、local-stack 起動 |
| `doctor.sh` | CI / 任意（手動） | exit 1（fail-hard） | 現状診断。役別必須ツール集合との突合、CI ゲート用 |

## ビルド / 利用

```bash
# WSL2 から VS Code を起動（既定 .devcontainer/devcontainer.json は docs-writer を指す）
cd ~/github/k1s0
code .
# VS Code の "Reopen in Container" を選択

# 役割切替（再起動前に実行）
./tools/sparse/checkout-role.sh tier1-rust-dev

# 本番再現スタックを別途起動 (postCreate でバックグラウンド起動済みの場合は不要)
./tools/local-stack/up.sh --role tier1-rust-dev
./tools/local-stack/status.sh
```

CLI から直接 image をビルドする場合:

```bash
docker build -f tools/devcontainer/profiles/<role>/Dockerfile \
    -t k1s0/devcontainer-<role>:dev tools/devcontainer/profiles/<role>/
```

ボリューム化された cargo registry / cargo git / target / nuget / pnpm-store 等は `docker volume ls | grep k1s0-` で確認できる。

## 関連

- 設計書: [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md)
- 設計書（ホスト側）: [`docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md`](../../docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md)
- 役割切替: [`tools/sparse/README.md`](../sparse/README.md)
- ローカル本番再現: [`tools/local-stack/README.md`](../local-stack/README.md)
- ADR: [ADR-DEV-001](../../docs/02_構想設計/adr/ADR-DEV-001-paved-road.md) / [ADR-DEV-002](../../docs/02_構想設計/adr/ADR-DEV-002-windows-wsl2-docker-runtime.md) / [ADR-TIER1-001](../../docs/02_構想設計/adr/ADR-TIER1-001-go-rust-hybrid.md) / [ADR-TIER1-002](../../docs/02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md) / [ADR-DIR-003](../../docs/02_構想設計/adr/ADR-DIR-003-sparse-checkout-cone-mode.md)
- IMP-DEV: `IMP-DEV-DC-010〜017` ([対応索引](../../docs/05_実装/50_開発者体験設計/90_対応IMP-DEV索引/01_対応IMP-DEV索引.md))
