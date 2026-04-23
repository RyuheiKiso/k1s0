# 01. DevContainer 10 役設計

本ファイルは k1s0 の開発者体験を支える Dev Container を、ADR-DIR-003 で確定した 10 役と 1:1 対応で設計する。`.devcontainer/` と `tools/devcontainer/profiles/` の二層配置、各役割のベースイメージと feature セット、VS Code 設定共有、ローカル Kubernetes（kind / k3d）と Dapr Local の統合までを物理配置レベルに落とし込む。

![10 役別 Dev Container 全体像](img/DevContainer_10役全体像.svg)

## なぜ 10 役別プロファイルが必要なのか

tier1 Rust 開発者に .NET SDK、pnpm、MAUI ワークロード、Longhorn CLI、OpenBao コマンドを全部入れた 12GB image を配ると、ビルドも rust-analyzer も遅くなり、VS Code の IntelliSense 応答が 10 秒を超える。逆に共通最小 image に寄せると、今度は「この tier の設定が入っていない」という Platform への問い合わせが毎回発生する。どちらも 2 名フェーズでは耐えられるが、10 名フェーズに入った瞬間に生産性が崩壊する。

ADR-DIR-003 の 10 役 cone 定義は、スパースチェックアウトだけでなく Dev Container もこの分割で提供することを前提に設計されている。本ファイルはその Dev Container 側の実装配置を固定し、「cone を切替えると同時に Dev Container も切替わる」という一貫した役割切替体験を Phase 0 から確定させる。

## `.devcontainer/` と `tools/devcontainer/profiles/` の二層構造

Dev Container の実体は 2 ディレクトリに分散配置する。ルート `.devcontainer/` は GitHub Codespaces / VS Code Remote Containers が既定で参照する場所であり、`tools/devcontainer/profiles/` は 10 役別の実体を保持する場所である。二層に分けた理由は、VS Code の `.devcontainer/devcontainer.json` が単一ファイルしか既定参照しないため、役割選択を別レイヤで行う必要があるからである。

```
.devcontainer/
├── devcontainer.json          # docs-writer（既定、最軽量）へのシンボリック参照
├── docker-compose.yml          # OpenBao dev / Postgres / Kafka / Valkey のローカル起動
└── settings/
    ├── common.settings.json     # 全役割共通（editor.formatOnSave 等）
    ├── extensions.common.json   # 全役割共通の VS Code 拡張
    └── <role>.settings.json     # 役割別上書き

tools/devcontainer/profiles/
├── tier1-rust-dev/
│   ├── devcontainer.json
│   ├── Dockerfile
│   └── extensions.json
├── tier1-go-dev/
├── tier2-dev/
├── tier3-web-dev/
├── tier3-native-dev/
├── platform-cli-dev/
├── sdk-dev/
├── infra-ops/
├── docs-writer/
└── full/
```

既定のルート `.devcontainer/devcontainer.json` は `docs-writer` プロファイルへの参照とする。理由は `docs-writer` が最軽量（約 1GB）で、リポジトリを clone して VS Code で開いた直後の「とりあえず動く」体験を最速化できるためである。開発者は `tools/sparse/checkout-role.sh <role>` を実行した際、`.devcontainer/devcontainer.json` が `tools/devcontainer/profiles/<role>/devcontainer.json` を指すよう自動書換される（スクリプトが `ln -sf` 相当の処理を実施、Windows は `mklink` 対応）。

## 10 役のベースイメージとサイズ目標

全プロファイルは `mcr.microsoft.com/devcontainers/universal:2-linux` を起点とせず、役割別に最小構成を組む方針とする。`universal:2` は 10GB 近い巨艦であり、Paved Road の「応答性重視」と噛み合わない。代わりに Debian slim ベースの自作 image を `tools/devcontainer/base/` でビルドし、feature を役割別に積む。

- `docs-writer` : 約 1GB。drawio / markdownlint / textlint / Pandoc / Mermaid CLI のみ。Rust / Go / Node / .NET は含まない
- `tier1-rust-dev` : 約 4GB。Rust Edition 2024 toolchain / cargo-fuzz / cargo-nextest / buf / protoc / ZEN Engine テスト資材
- `tier1-go-dev` : 約 3.5GB。Go 1.24 / Dapr Go SDK 開発依存 / buf / protoc / delve
- `tier2-dev` : 約 5GB。.NET 8 SDK / Go 1.24 / Temporal CLI / Dapr Components テスト資材
- `tier3-web-dev` : 約 4.5GB。Node 22 / pnpm 9 / Playwright / TypeScript / React DevTools
- `tier3-native-dev` : 約 5.5GB。.NET 8 SDK + MAUI workload / Android SDK 最小セット / WinUI 開発資材
- `platform-cli-dev` : 約 3GB。Rust Edition 2024 / Node 22（Backstage プラグイン用） / cosign / syft
- `sdk-dev` : 約 5GB。4 言語 SDK ビルド用（Rust / Go / .NET / Node）。ただし各言語は最小プロファイル
- `infra-ops` : 約 4.5GB。kubectl / helm / kustomize / argocd CLI / istioctl / flagd CLI / openTofu / k6
- `full` : 約 8GB。全機能。CI 検証用に限定し、日常開発での使用は非推奨とする

サイズ目標は Phase 0 時点での目安であり、Phase 1a 以降は Dev Container Features の bakery 化（Platform/Build チーム管理の社内レジストリ）で digest を固定しつつ、軽量化継続。`full` のみ 8GB を許容する理由は、CI での「全役割 matrix テスト」に使うためで、個人の日常開発では `full` を選ばせない運用とする。

## devcontainer.json のパターン

各プロファイルの `devcontainer.json` は次の共通構造を守る。`tier1-rust-dev` を例に記載する（他 9 役も同構造）。

- `name` : 役割名をそのまま記載。Backstage カタログから参照される
- `build.dockerfile` : 同一ディレクトリの `Dockerfile` を参照
- `features` : Dev Container Features を役割別に組合せ（kubectl / helm / dapr-cli 等は infra-ops と full のみ）
- `customizations.vscode.extensions` : `extensions.json` を include
- `customizations.vscode.settings` : `../../settings/common.settings.json` と `../../settings/<role>.settings.json` を merge
- `postCreateCommand` : `tools/sparse/checkout-role.sh <role> --verify` と `tools/local-stack/up.sh --role <role>` を順次実行
- `remoteUser` : `vscode`（uid 1000）
- `mounts` : `/var/run/docker.sock` は CI 用 `full` のみ bind mount、他役割は docker-in-docker

## VS Code 設定共有（settings / extensions）

IDE 設定の共有は `01_開発者体験原則.md` 原則 5（IMP-DEV-POL-005）の実装面である。`.devcontainer/settings/` に以下を配置する。

- `common.settings.json` : 全役割共通。`editor.formatOnSave=true` / `files.insertFinalNewline=true` / `files.trimTrailingWhitespace=true` / `editor.rulers=[100]`
- `extensions.common.json` : 全役割共通。GitLens / EditorConfig / Markdown All in One / Even Better TOML
- `<role>.settings.json` : 役割別上書き。例えば `tier1-rust-dev.settings.json` は `rust-analyzer.cargo.features=["proto-gen"]` / `rust-analyzer.check.command=clippy`
- `<role>.extensions.json` : 役割別拡張。`tier1-rust-dev` は rust-analyzer / Even Better TOML / CodeLLDB、`tier3-web-dev` は ESLint / Prettier / Tailwind CSS / Playwright

個人上書きは `.vscode/settings.local.json`（gitignore 対象）に分離する。これにより共有設定が個人都合で汚染されない。JetBrains IDE（Rider / IntelliJ）利用者向けには `.idea/` の inspection profile / code style のみ共有し、workspace.xml は除外する方針を IMP-DEV-POL-005 と同期する。

## ローカル Kubernetes と Dapr Local の統合

Dev Container 起動と同時にローカル Kubernetes クラスタ（kind / k3d）を立ち上げ、Dapr Local stack を注入する。これは IMP-DEV-POL-006（kind/k3d + Dapr Local で本番再現）の物理配置である。

- `tools/local-stack/kind-cluster.yaml` : kind 用クラスタ定義（CI 差分検証用）。control-plane 1 台 + worker 3 台、CNI は Calico、ingress は MetalLB 互換の `cloud-provider-kind`
- `tools/local-stack/k3d-config.yaml` : k3d 用（個人開発日常用）。起動 15 秒以内、リソース消費が軽い
- `tools/local-stack/up.sh` : `--role <role>` を受け取り、役割に応じて必要な namespace / Dapr component / flagd / Istio Ambient minimal を配備する
- `tools/local-stack/dapr/` : Dapr Local の components.yaml（state-store / pub-sub / bindings / secret-store）を役割別に配置。tier1 Rust / Go から `localhost:3500`（Dapr sidecar）または `dapr.io/app-id` 経由で参照
- `tools/local-stack/openbao-dev/` : OpenBao dev server の docker-compose 定義。ADR-SEC-002 に従い dev モードで起動し、ローカル専用のルートトークンを `.devcontainer/.env.local`（gitignore）に記録。本番 OpenBao は別インスタンス

minikube は採用しない。理由は kind / k3d に比べて起動が遅く、Docker Desktop の VM 層で 1 層深くなるためローカル体験が悪化するため。Rancher Desktop もローカル開発日常用には非推奨とし、k3d 直接利用を推奨する。

## time-to-first-commit への寄与と計測点

本ファイルで設計する Dev Container は、time-to-first-commit SLI（IMP-DEV-POL-004、2 名フェーズ 4h / 10 名フェーズ 2h）の主要短縮要素である。`postCreateCommand` の実行時間が SLI に直接効くため、次の計測点を GitHub Actions と Backstage Scorecards に露出する。

- `devcontainer-build-duration` : image ビルドの所要時間（既に pre-built image を社内レジストリに置く想定で、pull 時間のみが SLI に載る）
- `postcreate-duration` : sparse checkout + local-stack 起動の所要時間（目標 3 分以内）
- `first-hot-reload-duration` : VS Code 起動から最初の `cargo check` / `go run` / `pnpm dev` 完了までの時間（役割別目標: Rust 120 秒 / Go 60 秒 / Web 45 秒）

計測の詳細実装は `95_DXメトリクス/` に委譲し、本章は計測点の配置のみを固定する。

## Dev Container image の配布と digest 固定

Dev Container image は Platform/Build チームが GitHub Container Registry（ghcr.io/k1s0/devcontainer-<role>:<digest>）で配布する。digest は `tools/devcontainer/profiles/<role>/devcontainer.json` の `image` フィールドで sha256 固定し、Renovate（ADR-DEP-001）が週次で digest 更新 PR を出す。Renovate の PR は SRE + Platform の二者承認で merge される。

個人が `Dockerfile` をローカル編集して試す場合は、`devcontainer.json` に `build.dockerfile` を指定した「developer mode」へ切替える手順を README に明記する。developer mode のまま PR を出すことは CI で検出して拒否する（`image:` と `build:` の両方が指定されている状態を lint で検知）。

## Windows / macOS / Linux 間の差異吸収

k1s0 開発者の端末は Windows 11 / macOS 14+ / Ubuntu 22.04+ の 3 系統を想定する。Dev Container はこれらの差異を吸収する中核だが、完全に隠蔽できない部分を明文化しておく必要がある。

- **Docker runtime** : Windows / macOS は Docker Desktop または Rancher Desktop、Linux はネイティブ Docker Engine。Dev Container 起動スクリプトは `DOCKER_HOST` を自動検出し、Podman 利用者は `CONTAINER_ENGINE=podman` を `.env.local` に設定する
- **ファイルシステム性能** : Windows の WSL2 経由 bind mount は読み込みが遅いため、ソースツリーは WSL2 ファイルシステム内（`\\wsl$\Ubuntu\home\<user>\src`）に clone することを README で強く推奨
- **GPU / Apple Silicon** : MAUI の Android emulator、Playwright のヘッドレス Chromium は amd64 前提。Apple Silicon 利用者は Rosetta 2 経由または実機デバッグを推奨
- **改行コード** : `.gitattributes` で全テキストファイルを `text=auto eol=lf` に固定し、Windows ネイティブの CRLF が commit に混入しないよう pre-commit hook で検証

## 対応 IMP-DEV ID

本ファイルで採番する実装 ID は以下とする。

- `IMP-DEV-DC-010` : `.devcontainer/` と `tools/devcontainer/profiles/` の二層構造確定
- `IMP-DEV-DC-011` : 10 役別ベースイメージとサイズ目標（docs-writer 1GB / full 8GB / 他 3-5GB）
- `IMP-DEV-DC-012` : `devcontainer.json` の共通パターン（features / extensions / settings / postCreateCommand）
- `IMP-DEV-DC-013` : VS Code 設定共有（`.devcontainer/settings/` の common + role 分離）
- `IMP-DEV-DC-014` : ローカル Kubernetes（kind / k3d）と Dapr Local の統合（`tools/local-stack/`）
- `IMP-DEV-DC-015` : OpenBao dev server のローカル展開（本番 OpenBao と分離）
- `IMP-DEV-DC-016` : Dev Container image の digest 固定と Renovate 連動
- `IMP-DEV-DC-017` : time-to-first-commit 計測点の露出（postcreate-duration / first-hot-reload-duration）

## 対応 ADR / DS-SW-COMP / NFR

- ADR: [ADR-DIR-003](../../../02_構想設計/adr/ADR-DIR-003-sparse-checkout-cone-mode.md)（10 役 cone 定義）/ [ADR-DEV-001](../../../02_構想設計/adr/ADR-DEV-001-paved-road.md)（Paved Road 思想）/ [ADR-BS-001](../../../02_構想設計/adr/ADR-BS-001-backstage.md)（Backstage）/ [ADR-SEC-002](../../../02_構想設計/adr/ADR-SEC-002-openbao.md)（OpenBao）
- DS-SW-COMP: DS-SW-COMP-132（platform / scaffold）
- NFR: NFR-C-SUP-001（SRE 体制 2 名 → 10 名）/ NFR-C-NOP-004（運用監視）

## 関連章との境界

- [`00_方針/01_開発者体験原則.md`](../00_方針/01_開発者体験原則.md) の IMP-DEV-POL-003 / 005 / 006 の物理配置を本ファイルで固定する
- [`../../00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md`](../../00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md) と役割定義が 1:1 対応する
- [`../20_Golden_Path_examples/`](../20_Golden_Path_examples/) の `examples/` 起動は本ファイルの `tools/local-stack/up.sh` を前提とする
