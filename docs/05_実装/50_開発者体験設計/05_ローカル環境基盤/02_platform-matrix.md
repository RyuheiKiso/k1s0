# 02. 採用側プラットフォーム matrix（Windows/WSL2/macOS/Linux × dev/CI）

本ファイルは採用側組織の開発者が k1s0 を動かす **OS × 役割 × runner** の組合せを matrix で固定し、初日に「動かない / 遅い」が起きない構成を明示する。Windows 特化の詳細は [`01_WindowsWSL2環境構成.md`](01_WindowsWSL2環境構成.md) を参照、本ファイルは横断的視点で全 OS の比較・サポート方針・CI runner ポリシーを担当する。

## 関連設計

- [ADR-DEV-002（Windows WSL2 Docker runtime）](../../../02_構想設計/adr/ADR-DEV-002-windows-wsl2-docker-runtime.md)
- [ADR-DEV-001（Paved Road）](../../../02_構想設計/adr/ADR-DEV-001-paved-road.md)
- [01_WindowsWSL2環境構成.md](01_WindowsWSL2環境構成.md)
- [10_DevContainer_10役/01_DevContainer_10役設計.md](../10_DevContainer_10役/01_DevContainer_10役設計.md)
- [plan/02_開発環境整備/10_採用側プラットフォーム_matrix.md](../../../../plan/02_開発環境整備/10_採用側プラットフォーム_matrix.md)
- [tools/devcontainer/README.md](../../../../tools/devcontainer/README.md)

## サポートマトリクス（OS × 役割）

各 OS で **dev（開発者ローカル開発）** と **CI runner** の動作可否を区分する。`○` 完全サポート、`△` 制約付き、`×` 非対応。

| OS / Arch | dev（個人 / 起案者） | dev（採用側） | CI runner | 主な制約 / 備考 |
|---|---|---|---|---|
| **Linux x86_64** | ○ | ○ | ○（`ubuntu-22.04`） | リリース時点の主要構成。GitHub-hosted runner と同一環境で再現性最大 |
| **Linux arm64** | △ | △ | × （リリース時点 SHOULD） | tier1 / SDK のビルドは可能だが、Dapr / Istio Ambient の arm64 image が一部制限あり。CI matrix への追加はリリース時点+ |
| **Windows + WSL2** | ○ | ○（最多想定） | × （CI は Linux container only） | ADR-DEV-002 の正規構成。Docker Desktop または Rancher Desktop 必須。詳細は [`01_WindowsWSL2環境構成.md`](01_WindowsWSL2環境構成.md) |
| **Windows native（WSL2 なし）** | × | × | × | **非サポート**。WSL2 を導入することで動作 |
| **macOS arm64（M1/M2/M3）** | △（Codespaces 推奨） | ○ | × | Docker Desktop / Colima / Rancher Desktop / OrbStack のいずれか必須。.NET MAUI iOS ビルドは local（CI 非対応） |
| **macOS x86_64（Intel）** | ○ | ○ | × | 同上、ARM64 EOL に伴い段階非推奨化 |

### 「サポートしない」構成の明示

採用検討段階で誤解を防ぐため、以下を明示的に **非サポート** と宣言する:

- **Windows native（WSL2 なし）**: PowerShell のみで動かす構成は対象外。`up.sh` / `checkout-role.sh` は bash 前提
- **古い WSL1**: WSL2 に upgrade 必須（kernel ≥ 5.15、cgroup v2 対応）
- **Docker Desktop の商用 license なし採用側組織**: 商用利用は別ライセンスが必要。代替（Rancher Desktop / Colima / OrbStack）を [`01_WindowsWSL2環境構成.md`](01_WindowsWSL2環境構成.md) で誘導
- **macOS Apple Silicon の native iOS ビルドなし**: CI でも .NET MAUI iOS ビルドを matrix に含めない（GitHub-hosted の macOS runner はコスト高）

## OS 別の prerequisites と既知の制約

### Linux x86_64

採用側で最も推奨される構成。

- kernel ≥ 5.15 / cgroup v2（`tools/sparse/verify.sh` で前提検証可能）
- containerd or Docker CE
- iSCSI initiator（Longhorn 用）
- chrony / systemd-timesyncd（時刻同期）

### Linux arm64

- 上記に加え、Dapr / Istio Ambient の arm64 公式 image を参照
- Dapr Sidecar の arm64 サポートは v1.13+ で安定化、リリース時点では x86_64 と同等
- Strimzi Kafka の arm64 image は `quay.io/strimzi/kafka:0.47.0-kafka-3.7.1-arm64` 等を選択

### Windows + WSL2

- Windows 10 22H2 または Windows 11 21H2+
- WSL2 + Ubuntu 22.04 LTS
- Docker Desktop（個人利用）または Rancher Desktop / Podman Desktop / OrbStack（商用代替）
- VS Code + Dev Containers extension
- Git for Windows の line ending を LF（`core.autocrlf = input`）

### macOS arm64 / x86_64

- macOS 13 Ventura 以降
- Docker Desktop（個人利用）または Colima / Rancher Desktop / OrbStack
- Xcode（iOS / .NET MAUI のみ、リリース時点 では tier3-native-dev role 限定）

### Codespaces（推奨経路）

- リポジトリの **Code → Codespaces** で起動可能
- 4-core / 8GB RAM / 32GB disk が最小、推奨は 8-core / 16GB RAM
- `tools/devcontainer/profiles/<role>/devcontainer.json` を `Reopen in Container` で選択

## CI runner ポリシー

### リリース時点 採用方針

- **GitHub-hosted runner のみ採用**。`ubuntu-22.04` 系 / `ubuntu-latest` を default
- self-hosted runner（actions-runner-controller / ARC）は採用しない
- larger runner（4-core / 8-core / 16-core）は integration / e2e ジョブで利用

### self-hosted runner を採用しない理由

- 個人 OSS で起案者が runner インフラを保守する負担を持てない
- self-hosted は CI 時間短縮には有効だが、採用初期の小規模運用では費用対効果が低い
- 採用側組織が self-hosted を導入する場合は **fork 後の選択** とし、k1s0 本体には組み込まない（plan 14-09 採用側 fork 運用と整合）

### 採用側 self-hosted への移行ガイダンス

採用側で利用量が増え GitHub-hosted の有料枠が逼迫する場合、以下の順で検討する:

1. **larger runner**（GitHub-hosted）への upgrade（最小コスト）
2. **GitHub Codespaces** で開発者ローカル CI を代替
3. **self-hosted ARC**（k8s クラスタ内）を採用側 fork 配下に追加

ARC の参照実装は [`docs/05_実装/30_CI_CD設計/10_reusable_workflow/01_reusable_workflow設計.md`](../../30_CI_CD設計/10_reusable_workflow/01_reusable_workflow設計.md) の self-hosted 節に記載。本体 main では発動しない。

### CI matrix 拡張ロードマップ

| 段階 | matrix 構成 |
|---|---|
| リリース時点 | `ubuntu-22.04` × `amd64` のみ |
| リリース時点+ 3ヶ月 | `ubuntu-22.04` × {`amd64`, `arm64`} の 2 軸 |
| リリース時点+ 6ヶ月 | `macos-latest` を SDK 言語別 hello で追加（4 言語の native ビルド検証） |
| 全社展開期 | `windows-latest` を tier3-native MAUI 用に追加（コスト高、選択的） |

## Codespaces / GitPod / DevPod の比較

採用側が cloud dev environment を選ぶ場合の選択肢を中立的に提示する。

| サービス | コスト感 | 起案者推奨度 | 備考 |
|---|---|---|---|
| **GitHub Codespaces** | $0.18/hr × core（無料枠 60h/月） | ◎（リリース時点 default） | 本リポジトリの `.devcontainer/` 設定が直接動く |
| **GitPod** | $0.04/credit、$9/月で 50h | ○ | 個人で安価。Dev Container 直接対応 |
| **DevPod** | self-hosted（無料） | △ | 採用側 cluster で立てる場合の選択肢 |

## 関連

- [01_WindowsWSL2環境構成.md](01_WindowsWSL2環境構成.md) — Windows/WSL2 の詳細手順
- [tools/devcontainer/README.md](../../../../tools/devcontainer/README.md) — Dev Container 配置
- [tools/sparse/README.md](../../../../tools/sparse/README.md) — sparse-checkout 役割切替
- [ADR-DEV-002](../../../02_構想設計/adr/ADR-DEV-002-windows-wsl2-docker-runtime.md)
