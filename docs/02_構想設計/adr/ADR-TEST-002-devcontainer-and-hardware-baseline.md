# ADR-TEST-002: 開発環境を devcontainer で固定し、ハードウェア最低要件を ADR で正典化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / 開発者体験チーム / 採用初期の協力者

## コンテキスト

ADR-TEST-001 で「リリース時点では CI を導入せず、qualify 基盤をローカル `make qualify-release` に集約する」と決定したが、ローカル qualify が品質経路として成立する前提条件は **「開発者の手元環境がビット単位で固定されていること」** である。toolchain（Rust / Go / Node / Python の各バージョン）、依存バイナリ（kind / kubectl / helm / multipass / KWOK / chaos mesh / Sonobuoy / Velero / minio CLI 等）、OS distribution、glibc バージョン、locale、timezone のどれか一つでも開発者ごとにズレると、qualify report の再現性が原理的に崩れる。

採用検討者が release artifact に同梱された qualify report を信頼する根拠は「自分が同じ環境を立ち上げて `make qualify` を再走すれば同じ結果になる」という再現性である。CI badge の代替として release artifact 中心の品質公開を採用した以上、「環境を固定する SoT は何か」を ADR で正典化しないと、ADR-TEST-001 の決定が砂上の楼閣になる。

加えて、qualify 全層（L0–L10、ADR-TEST-003 で詳細）をローカルで回すには、kubeadm 3 control-plane HA + Chaos Mesh + KWOK 1000 node + 24h soak を同時 / 連続実行できる物理リソースが要る。低スペックマシン（8GB RAM / HDD 等）では L7/L8 がそもそも回らず、qualify が「動かないまま緑」になる。これは品質経路の偽装に等しいため、**ハードウェア最低要件も ADR で明文化し、低スペックでの参加を構造的に拒否**する必要がある。これは個人 OSS としては硬派な選択だが、qualify 中心モデルと整合する唯一の道である。

ADR-DEV-002 で「Windows 上の WSL2 + Docker」をリファレンス OS として採用済だが、その上で動く toolchain layer の固定方法は未定だった。toolchain 固定の選択肢として実用上は ① Nix flake、② devcontainer、③ asdf / mise + docker compose dev shell、④ 各自任せ（環境固定なし）の 4 パターンがあり、それぞれが学習コスト・採用スキル流用性・CI 移植性・採用検討者の即時試走容易性で大きく異なる。

選定では以下を満たす必要がある:

- **ビット単位の固定性**（toolchain version / 依存バイナリ / OS layer まで一意決定）
- **採用検討者の即時試走容易性**（VSCode + Docker のような世間で標準的な道具で 5 分以内に環境到達できる）
- **CI 移植性**（ADR-TEST-001 portable 制約 5 と整合、Phase 1 の CI runner image が再利用できる）
- **採用組織のスキル流用性**（10 年保守期間で世代交代しても保守できる、独自 DSL に閉じない）
- **arm64 / amd64 デュアル対応**（リファレンスは amd64 だが、arm64 検証用の補助機を排除しない）

## 決定

**開発環境の Single Source of Truth は `.devcontainer/devcontainer.json` + Dockerfile に固定する。** リファレンス IDE は VSCode（Dev Containers 拡張）だが、devcontainer 仕様（[Development Containers Specification](https://containers.dev/)）は OSS で標準化されており、JetBrains Gateway / GitHub Codespaces / `devcontainer` CLI（`@devcontainers/cli`）のいずれでも動かせる。**特定 IDE への完全ロックインではなく、devcontainer 仕様への依存に留める**。

devcontainer image は以下を満たす:

1. **ベース OS**: Ubuntu 24.04 LTS（ADR-DEV-002 のリファレンス OS と整合、glibc 2.39 を固定）
2. **toolchain 固定**: Rust（stable + nightly pinned date）、Go 1.24+、Node 20 LTS、Python 3.12 を `mise` 経由で `.tool-versions` ファイルから一意決定
3. **依存バイナリ pinned**: kind / kubectl / helm / multipass / KWOK / chaos-mesh CLI / sonobuoy / velero / mc（minio CLI）/ buf / cargo-nextest / golangci-lint 等を Dockerfile 内で SHA256 検証付きインストール
4. **archive 適用方針**: image tag は `ghcr.io/k1s0/devcontainer:<git-sha>` で起案者が build & push、commit と一意対応。`latest` tag は使わない
5. **arm64 / amd64 マルチアーキ**: `docker buildx` で両アーキ image を生成、開発者マシンの arch に応じて自動 pull
6. **mount 設計**: `/workspace` にリポジトリを bind mount、`/home/vscode/.cache/<tool>` を named volume で永続化（cargo / go mod / npm cache の再 download コスト削減）
7. **post-create script**: `bash .devcontainer/post-create.sh` で `mise install` / `cargo fetch` / `go mod download` / `pnpm install` / `kind` 用 docker network 作成までを自動化、開発者の初期化操作を「Reopen in Container」だけに圧縮する

ハードウェア最低要件は以下を ADR で正典化し、`docs/governance/HW-REQUIREMENTS.md` に詳細手順とともに公開する:

| 区分 | リファレンス機 | 補助機（任意） |
|------|---------------|---------------|
| CPU | x86_64 / 8 物理コア以上（推奨 16 コア） | Apple Silicon M2/M3/M4 系 / 8 コア以上 |
| RAM | 32GB 以上（推奨 64GB） | 16GB 以上 |
| Storage | NVMe 1TB 以上 / qualify 用空き 200GB | NVMe 256GB 以上 |
| OS | Windows 11 + WSL2 (Ubuntu 24.04) または Linux 直 (Ubuntu 24.04) | macOS 14+ |
| 用途 | qualify 全層実行 / nightly soak / release qualify | arm64 cross-build 検証、L9 upgrade での旧 K8s 動作確認 |

採用検討者の試走（`make qualify` の smoke / standard 層のみ）は **16GB RAM / 4 コア / SSD 100GB** の最低要件で成立するよう、qualify は層別に必要リソースを宣言する（`tools/qualify/<layer>/RESOURCES.md`）。本 ADR で固定するのは「起案者・協力者がフル qualify を回す側」の要件であり、採用検討者の試走要件は別表で分離する。

## 検討した選択肢

### 選択肢 A: Nix flake で開発環境を完全固定

- 概要: `flake.nix` を SoT とし、`nix develop` で toolchain・依存バイナリを宣言的に固定。直近の OSS（NixOS / Determinate Systems）が公式に Dev Container Spec も統合可能
- メリット:
  - **ビット単位の再現性**が言語仕様レベルで保証される（hash-based content addressable store）
  - 純粋関数的な依存管理で side-effect が原理的に起きない
  - flake が CI runner image にもそのまま使える（Nix 公式の GitHub Action 多数）
  - cross-arch（arm64 / amd64）が flake で自然に表現できる
- デメリット:
  - **学習コストが極端に高い**。Nix expression 言語と nixpkgs の慣習を採用組織が世代交代後も保守できる保証が薄い
  - 採用検討者が「VSCode + Docker」より高い前提知識を要求される（Nix を初見で触るハードル）
  - WSL2 + Docker Desktop 上での Nix 統合に追加設定が要る（`/nix` store の永続化、daemon mode 等）
  - Nix 自体の dev experience が Rust analyzer / VSCode extension との統合で不安定な場面がある
  - 採用組織のエンタープライズ部門が Nix を社内標準として認める可能性が低い

### 選択肢 B: devcontainer（採用）

- 概要: `.devcontainer/devcontainer.json` + Dockerfile で開発環境を Docker image に固定。Dev Containers Spec（containers.dev）に準拠し、VSCode / JetBrains Gateway / Codespaces / devcontainer CLI で動かす
- メリット:
  - **採用検討者の即時試走容易性が極めて高い**（VSCode + Docker → "Reopen in Container" の 1 操作で 5 分以内に環境到達）
  - Dev Containers Spec は OSS の標準仕様で、特定ベンダーロックインではない（CNCF / Microsoft 主導の OSS プロジェクト）
  - Dockerfile + `devcontainer.json` という業界標準スキルで保守できる、世代交代後も保守容易
  - CI 移植性が高い（GitHub Actions の `devcontainers/ci` action / Codespaces / 自前 docker build がすべて同じ image を使える、ADR-TEST-001 portable 制約 5 と整合）
  - arm64 / amd64 マルチアーキ build が `docker buildx` 標準機能で対応
  - Nix より学習コストが低く、採用組織の標準スキルから外れない
- デメリット:
  - **VSCode 偏依存に見えやすい**。仕様としては中立だが、世間の認識は「VSCode の機能」止まりが多い
  - Docker daemon への依存があり、Podman / containerd 専用環境では追加設定が要る
  - image size が肥大化しがち（toolchain 7 種 + 依存バイナリ 10+ 種で 5–10GB 規模）、初回 pull 時間が長い
  - WSL2 上の Docker Desktop ではファイル I/O 性能が劣化することがある（bind mount 性能）

### 選択肢 C: asdf / mise + docker compose dev shell の組み合わせ

- 概要: ホスト OS に直接 mise（or asdf）で toolchain を入れ、補助的に docker compose dev shell で重い依存（kind / multipass 等）をコンテナ化する分割アプローチ
- メリット:
  - ホスト OS で直接 IDE を動かすので I/O 性能 / debugger 接続が高速
  - 学習コストが低い（mise / asdf は CLI ツールとして単純）
  - 部分的に shell を docker 化することで「重い依存だけ隔離」が可能
- デメリット:
  - **ビット単位の固定性が崩れる**。ホスト OS の glibc / OpenSSL バージョン / locale 設定が開発者ごとに微妙に異なり、qualify report の再現性が保証できない
  - 「mise でホスト toolchain」と「docker compose で k8s 系」の二重管理になり、SoT が分散する
  - 採用検討者が即時試走するには「mise install + docker compose up + ...」の手順を踏ませる必要があり、devcontainer の "Reopen in Container" 一発に劣る
  - CI runner image との対応関係が曖昧になり、ADR-TEST-001 portable 制約 5 を満たしづらい

### 選択肢 D: 各自任せ（環境固定なし）

- 概要: README に「Rust 1.x / Go 1.x / kind / kubectl が要る」と書くだけで、具体的なバージョン・OS・固定方法は開発者裁量
- メリット:
  - 規約整備工数ゼロ、初期立ち上げが最速
  - 開発者の好みの IDE / OS / package manager を尊重できる
- デメリット:
  - **qualify 再現性が原理的に崩れる**。release artifact の品質公開モデル（ADR-TEST-001）と全面衝突
  - 「動かない」「flaky だ」の triage が個人 OSS の運用工数を破壊する
  - 採用検討者が何の環境を立ち上げれば良いか不明、5 分試走が成立しない
  - 採用組織が「k1s0 を保守する標準環境とは何か」を ADR から答えられず、10 年保守の前提を欠く

## 決定理由

選択肢 B（devcontainer）を採用する根拠は以下。

- **採用検討者の即時試走容易性が他選択肢と桁違い**: 採用検討者が VSCode + Docker をインストール済（業界標準）であれば、`git clone` → "Reopen in Container" の 1 操作で 5 分以内に同じ環境に到達できる。Nix（A）は学習段差が大きく、asdf+compose（C）は手順が長く、各自任せ（D）は不可能。release artifact 中心の品質公開モデル（ADR-TEST-001）が、採用検討者の手元再走で初めて完結するため、この経路を最短化することが ADR-TEST-001 の決定を生かす条件
- **CI 移植性で portable 制約 5 と完全整合**: devcontainer image は GitHub Actions の `devcontainers/ci` で同じ image をそのまま runner として使え、Phase 1 移行時に「devcontainer をビルドして job 内で動かす」だけで CI 化できる。Nix（A）も移植性は高いが、CI 環境での `nix develop` 起動コストが大きく、devcontainer の方が一般的な GitHub Actions 経路に乗りやすい
- **学習コストと保守スキル流用性**: devcontainer は「Dockerfile を読み書きできる人」なら保守できる。Nix は採用組織の世代交代後に Nix expression を読める人材を確保し続ける前提が要り、10 年保守で重荷。asdf+compose（C）は二重管理で SoT が割れ、各自任せ（D）は SoT が無い
- **ビット単位の固定性**: devcontainer は image tag を `git-sha` で固定し、image 自体を ghcr.io にプッシュすれば「同じ tag = 同じ環境」が保証される。Nix（A）はさらに強い保証を提供するが、devcontainer で十分に必要な再現性は得られる。差分は採用組織の保守可能性とトレードオフし、後者を優先
- **arm64 / amd64 マルチアーキ対応**: `docker buildx` で両アーキ image を 1 つの Dockerfile から生成でき、開発者マシンの arch に応じて自動 pull される。Apple Silicon 補助機での arm64 検証が devcontainer 仕様で自然に表現できる
- **ハードウェア最低要件を ADR で固定する論理的整合**: devcontainer 採用と「低スペック排除」は同じ硬派な姿勢で、両方をリリース時点に確定することで、qualify 全層が動く前提を構造的に保証する。最低要件を ADR で正典化することは、採用検討者に「k1s0 を本格的に保守するなら 30〜50 万円規模のハードウェア投資が要る」を最初から開示する誠実な態度でもある

## 影響

### ポジティブな影響

- 開発者間の環境差異起因 flaky が原理的に発生しない（image tag が同じなら環境がビット単位で一致）
- 採用検討者が VSCode + Docker で 5 分以内に同じ環境へ到達でき、release artifact の qualify report を手元で再走できる
- Phase 1 で CI を導入する際、devcontainer image をそのまま runner として使えるため移行コストが最小（ADR-TEST-001 Phase 1 の +1 人日の根拠）
- ハードウェア最低要件が `docs/governance/HW-REQUIREMENTS.md` で公開されることで、採用検討者の意思決定が早期化（投資判断を先延ばしせず、最初から提示）
- toolchain version の更新が `.devcontainer/Dockerfile` 1 ファイルへの PR で完結し、リポジトリ全体の version drift が防止される
- arm64 検証用の補助機を排除しない設計で、cross-arch portability の検証経路が確立する

### ネガティブな影響 / リスク

- 開発参加のハードウェアコスト（リファレンス機 30〜50 万円規模）が個人 OSS としては高めで、低スペックマシン保有者の参加機会を狭める。Phase 2 で contributor を募集する際、HW 投資ができる層に限定されるリスク
- VSCode への偏依存と認識されるリスクがある。`docs/governance/IDE-COMPATIBILITY.md` を別途整備し、JetBrains Gateway / `devcontainer` CLI / Codespaces 経由の動作確認を採用初期で実施する必要
- devcontainer image size が 5–10GB と大きく、初回 pull が低帯域環境で長時間化する。`docs/governance/QUICKSTART.md` で初回 pull 時間の見積りを開示する必要
- WSL2 上の Docker Desktop ではファイル I/O 性能が劣化することがあり、qualify の smoke 層であってもビルド時間が増える可能性。実測値を `docs/governance/HW-REQUIREMENTS.md` に併記し、開発者が事前に把握できるようにする
- ghcr.io 上の image を起案者が build & push する運用で、起案者不在時に image 更新が止まる単一障害点リスク。Phase 2 で contributor 2 名以上が確保された段階で push 権限を分散する移行計画を `docs/governance/CI-ROADMAP.md` に明記する必要
- mise / Dockerfile / `.tool-versions` の三重定義で version 情報が分散しないよう、`.devcontainer/Dockerfile` から `.tool-versions` を読む構造に統一する規律が要る

### 移行・対応事項

- `.devcontainer/devcontainer.json` を新設し、Dev Containers Spec 準拠の構成を記述（VSCode 拡張 / mount / postCreateCommand / forwardPorts / features 等）
- `.devcontainer/Dockerfile` を新設し、Ubuntu 24.04 LTS ベースで toolchain・依存バイナリを SHA256 検証付きインストール
- `.tool-versions` を新設し、mise が読み取る形式で Rust / Go / Node / Python のバージョンを宣言
- `.devcontainer/post-create.sh` を新設し、初回起動時の依存 fetch / docker network 作成 / kind cluster の hint メッセージを統合
- `.github/dependabot.yml`（Phase 0 では CI なしのため CI 連動はしない、Renovate もしくは手動更新）にて devcontainer 依存の更新追跡経路を整備（ADR-DEP-001 と整合）
- `docs/governance/HW-REQUIREMENTS.md` を新設し、リファレンス機 / 補助機 / 採用検討者試走機の 3 区分でハードウェア要件を散文 + 表で公開
- `docs/governance/IDE-COMPATIBILITY.md` を新設し、VSCode 以外（JetBrains Gateway / Codespaces / `devcontainer` CLI）での動作確認結果を採用初期に充足する
- `docs/governance/QUICKSTART.md` を新設し、`git clone` → "Reopen in Container" → `make qualify` の 3 ステップを採用検討者向けに公開
- `tools/devcontainer/build.sh` を新設し、`docker buildx` で arm64 / amd64 マルチアーキ image を ghcr.io へ push する手順を Runbook 化
- README.md に「k1s0 開発環境は devcontainer 固定。詳細は `docs/governance/HW-REQUIREMENTS.md` / `QUICKSTART.md`」の動線を追加
- ADR-DEV-002 に「WSL2 上で Docker Desktop を起動 → VSCode から Dev Containers 拡張で Reopen in Container」のフローを追記し、本 ADR と整合

## 参考資料

- ADR-TEST-001（CI 留保 + qualify portable 設計）— 本 ADR の前提となる portable 制約 5「devcontainer SoT」の根拠
- ADR-DEV-002（Windows + WSL2 + Docker runtime）— リファレンス OS 層の決定、本 ADR の toolchain 層と組み合わせて開発環境全体の SoT を成立させる
- ADR-DEP-001（Renovate）— devcontainer 依存の更新追跡経路
- ADR-POL-002（local-stack を構成 SoT に統一）— SoT 思想の先例、本 ADR は同思想を開発環境にも適用
- ADR-OPS-001（Runbook 8 セクション + バス係数 2）— ハードウェア要件の `docs/governance/HW-REQUIREMENTS.md` を Runbook 風に整備する根拠
- NFR-F-SYS-001（オンプレ完結）— devcontainer image を ghcr.io でなく自前 Harbor へ push する選択肢の余地
- NFR-C-NOP-001（小規模運用）— ハードウェア最低要件と個人 OSS の整合
- Development Containers Specification: containers.dev
- Microsoft Dev Containers extension（VSCode）: code.visualstudio.com/docs/devcontainers/containers
- mise（runtime version manager）: mise.jdx.dev
- 関連 ADR（採用検討中）: ADR-TEST-003（テストピラミッド）/ ADR-TEST-004（二層 E2E）/ ADR-TEST-005（環境マトリクス）
