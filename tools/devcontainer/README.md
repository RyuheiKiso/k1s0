# Dev Container 実装資産

本ディレクトリは k1s0 開発で利用する Dev Container の実装資産（Dockerfile / postCreate / 役割別プロファイル）を保持する。設計書は [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md) に固定されており、本 README はその実装段階の進捗とフェーズ間の差分を宣言する。

## 配置

```
tools/devcontainer/
├── README.md                    # 本ファイル: 実装ステータスと bootstrap 説明
├── postCreate.sh                # 共通 postCreate (Dapr CLI 導入 + toolchain 検証)
├── base/                        # Phase 2 で配置予定: 共通最小ベース image (debian slim)
└── profiles/
    └── tier1-rust-dev/          # Phase 1 bootstrap で稼働中の唯一プロファイル
        └── Dockerfile
```

リポジトリルートの [`.devcontainer/devcontainer.json`](../../.devcontainer/devcontainer.json) が本ディレクトリ配下の Dockerfile / postCreate を参照する。

## Phase 1 bootstrap の状態（リリース時点 単一開発者フェーズ）

設計書（10_DevContainer_10役）が定義する 10 役のうち、**`tier1-rust-dev` の 1 プロファイルのみ**を稼働化する。残り 9 役（`tier1-go-dev` / `tier2-dev` / `tier3-web-dev` / `tier3-native-dev` / `platform-cli-dev` / `sdk-dev` / `infra-ops` / `docs-writer` / `full`）は team 拡大段階で順次配置する。Paved Road（ADR-DEV-001）の「正しい道一本化」原則は、**現フェーズでは「tier1 中心の唯一の道」として一時的に縮約**しており、team 拡大段階で各役割が独立した正しい道に分かれる。

bootstrap 段階では、`tier1-rust-dev/Dockerfile` は `mcr.microsoft.com/devcontainers/rust:1-bookworm` を起点としている。設計の正典では `tools/devcontainer/base/Dockerfile`（debian slim 由来）から派生させる方針だが、Platform/Build チームによる base image の社内 GHCR 配布インフラが未整備のため、Phase 1 では Microsoft 公式 image を起点とする簡素化を採用する。Phase 2 で `base/` を整備した時点で `FROM` 先を切替える。

## ホスト docker.sock 経由の構成

[ADR-DEV-002](../../docs/02_構想設計/adr/ADR-DEV-002-windows-wsl2-docker-runtime.md)（WSL2 ネイティブ docker-ce）に従い、Dev Container 内から host docker.sock 経由で kind クラスタ・ローカルコンテナを操作する構成を採る。`devcontainer.json` の `docker-outside-of-docker` Feature が host の `/var/run/docker.sock` を bind mount するため、`docker` および `kind` コマンドは container から host docker daemon に対して直接動作する。Docker-in-Docker は採用しない（CI 用 `full` プロファイルでも将来は同方針）。

## 既知の差分（設計書と現状の乖離）

| 設計書の規定 | Phase 1 の現実装 | 解消予定フェーズ |
|---|---|---|
| 10 役プロファイルの並走 | tier1-rust-dev の 1 役のみ | team 拡大段階（5 名規模到達時） |
| `base/Dockerfile` から派生 | mcr.microsoft.com/devcontainers/rust:1-bookworm から派生 | Phase 2（Platform/Build 体制確立後） |
| `tools/sparse/checkout-role.sh` での役割切替 | 切替対象なし（1 役のみ） | 2 役目以降の追加と同時 |
| postCreate での `tools/local-stack/up.sh` 呼び出し | local-stack 未整備のため未呼び出し | local-stack 整備時（kind / Dapr Local 統合） |
| Dev Container image の GHCR digest 固定（IMP-DEV-DC-016） | ローカルビルド + Renovate 連動なし | Platform/Build 体制確立後 |
| 各プロファイルに `extensions.json` 個別配置 | `.devcontainer/devcontainer.json` 内インライン | プロファイル増加時に分離 |

## ビルド / 利用

```bash
# WSL2 から VS Code を起動
cd ~/github/k1s0
code .
# VS Code の "Reopen in Container" を選択 (初回ビルドは 5〜10 分)
```

CLI から直接ビルドを試したい場合:

```bash
docker build -f tools/devcontainer/profiles/tier1-rust-dev/Dockerfile -t k1s0/devcontainer-tier1-rust-dev:dev .
```

ボリューム化された cargo registry / cargo git / target は `docker volume ls | grep k1s0-` で確認できる。Volume を初期化したい場合は `docker volume rm k1s0-cargo-registry k1s0-cargo-git k1s0-target` で削除する。

## 関連

- 設計書: [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md)
- 設計書（ホスト側）: [`docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md`](../../docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md)
- ADR: [ADR-DEV-001](../../docs/02_構想設計/adr/ADR-DEV-001-paved-road.md) / [ADR-DEV-002](../../docs/02_構想設計/adr/ADR-DEV-002-windows-wsl2-docker-runtime.md) / [ADR-TIER1-001](../../docs/02_構想設計/adr/ADR-TIER1-001-go-rust-hybrid.md) / [ADR-TIER1-002](../../docs/02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md)
- IMP-DEV: `IMP-DEV-DC-010〜017` ([対応索引](../../docs/05_実装/50_開発者体験設計/90_対応IMP-DEV索引/01_対応IMP-DEV索引.md))
