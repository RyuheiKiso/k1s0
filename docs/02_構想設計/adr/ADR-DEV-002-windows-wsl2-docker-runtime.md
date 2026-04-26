# ADR-DEV-002: Windows 11 + WSL2 環境の Docker ランタイムに WSL ネイティブ docker-ce を採用

- ステータス: Accepted
- 起票日: 2026-04-26
- 決定日: 2026-04-26
- 起票者: kiso ryuhei
- 関係者: DX チーム / Platform/Build / Security / SRE / 全 Windows 利用開発者

## コンテキスト

k1s0 の開発者体験は ADR-DEV-001（Paved Road）で「正しい道が最短経路」となる構造を採用し、`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md` で Dev Container の 10 役別プロファイルと、kind / k3d + Dapr Local による本番再現（IMP-DEV-POL-006）を確定している。一方で**ホスト OS 側の Docker ランタイム**は、当該設計書 119 行目で「Windows / macOS は Docker Desktop または Rancher Desktop、Linux はネイティブ Docker Engine」と一行で概括されたまま、選定根拠と Windows + WSL2 ケースの具体方針が確定していない。

Windows + WSL2 ホストは k1s0 開発者の主流環境である。日本企業で支給される業務端末は Windows 11 が支配的であり、Linux ベースの開発は WSL2 経由が現実解となる。この環境で Docker ランタイムを選ぶ場合、選択肢は最低 4 系統存在する。

- **Docker Desktop for Windows**: Docker Inc. の商用製品。WSL2 backend モードで内部に専用 distribution (`docker-desktop`) を立て、その上で dockerd を動かす。
- **WSL2 distribution 内に docker-ce をネイティブ導入**: Ubuntu 等の distribution に Docker 公式 apt リポジトリから直接 `docker-ce` を入れ、systemd で起動する。
- **Rancher Desktop**: SUSE 提供の OSS。内部に k3s を抱え、moby / containerd を選択可能。
- **Podman / Podman Desktop**: Red Hat 系 OSS。daemonless / rootless が訴求点。

ホスト Docker ランタイムは個人の好みに任せると、ライセンス遵守・I/O 性能・kind 連携・リソース管理の 4 軸で組織横断の摩擦を生む。具体的には次の事象が頻発する。

- **Docker Desktop の商用ライセンス**: 2021 年以降、Docker Desktop は従業員数 250 名超または年間売上 1,000 万米ドル超の組織で有償サブスクリプションが必要となった。採用側組織が小規模運用から拡大段階に入った時、ランタイム選定をやり直す移行コストが発生する。最初から OSS 経路で揃えればこの境界を回避できる。
- **多重 VM スタックによる起動コスト**: Docker Desktop は WSL2 上に独立した `docker-desktop` distribution を立て、その上で dockerd を動かす。kind を使うと「Windows host VM → docker-desktop distro → kind コンテナ」と 3 層になり、`kubectl port-forward` のネットワーク経路が複雑化する。WSL ネイティブ docker-ce では「Ubuntu distro → kind コンテナ」の 2 層に抑えられる。
- **bind mount のクロス distro ペナルティ**: Docker Desktop の WSL2 backend は dockerd と利用者の distro が別 distro のため、利用者 distro の `/home/user/repo` を bind mount するとクロス distro 9p hop が発生する。WSL ネイティブ docker-ce では同一 distro 内 ext4 直アクセスのため、tier1 Rust の `cargo build` / rust-analyzer 応答が体感で改善する。
- **リソース割当の二重管理**: Docker Desktop は独自 UI で memory / CPU を設定し、`.wslconfig` と Docker Desktop Settings の整合を別々に取る必要がある。WSL ネイティブでは `.wslconfig` 一元で完結する。
- **debug 時の不可視性**: Docker Desktop の Linux VM 内ファイルは Windows / 利用者 WSL distro の両側から見えず、`docker exec` 以外の経路が存在しない。WSL ネイティブでは `/var/lib/docker` が利用者 distro の rootfs に配置され、storage driver の状態を直接観察できる。

加えて 2022 年 9 月の Microsoft 公式アナウンス以降、WSL2 は systemd を Stable サポートしており、`/etc/wsl.conf` の `[boot] systemd=true` 一行で `systemctl enable --now docker` が動作する。docker-ce を Docker Desktop と同等の起動信頼性で運用するための前提技術はリリース時点で確立済みである。Ubuntu 24.04 LTS が WSL2 の既定選択肢となっている現状（Microsoft Store 経由）と組み合わせれば、ホスト側の標準構成は OSS 経路で十分到達可能である。

ADR-DEV-001 のスコープ外（Template の内容、Scaffold CLI の実装詳細、catalog-info.yaml のスキーマは別 ADR で扱う旨の記載）に含まれる「Dev Container を成立させる前提のホスト Docker ランタイム」は、本 ADR で明示的に確定する。Paved Road の入口でランタイムが揺れると、後続の Dev Container / kind / Dapr Local 設計が選択肢の海に沈むためである。

## 決定

**Windows 11 + WSL2 環境の Docker ランタイムは、WSL2 distribution 上に WSL ネイティブ docker-ce をインストールして使用する。Docker Desktop / Rancher Desktop は採用しない。**

具体的な確定事項は以下とする。

- **distribution 標準**: Ubuntu 24.04 LTS（または同等 LTS）を WSL2 の標準 distribution とする。`/etc/wsl.conf` で `[boot] systemd=true` を必須化する。
- **Docker パッケージ**: Docker 公式 apt リポジトリ（`https://download.docker.com/linux/ubuntu`）から `docker-ce` `docker-ce-cli` `containerd.io` `docker-buildx-plugin` `docker-compose-plugin` を導入する。distribution 同梱の `docker.io` は採用しない（Buildx / compose v2 同梱の都合）。
- **systemd 統合**: `systemctl enable --now docker` で起動し、distribution 起動と同時に docker daemon が立ち上がる構成とする。`service docker start` を手動実行する旧来運用は採用しない。
- **リソース割当**: Windows 側 `%USERPROFILE%/.wslconfig` に memory / processors / swap / nestedVirtualization / vmIdleTimeout / localhostForwarding を集約する。Docker daemon 側で `--cpus` / `--memory` の二重制御は行わない。
- **kind / k3d との結合**: kind / k3d は同一 distribution 上で `docker.sock` を直接参照する構成とする。CI 検証用 `full` プロファイル以外は docker-in-docker を採用しない（IMP-DEV-DC-012 と同期）。
- **Dev Container との結合**: VS Code Remote Containers / Dev Containers 拡張が WSL2 内 `docker.sock` に接続する。Windows 側に Docker Desktop を起動しない（同時起動で distribution の dockerd 配備先が切り替わるため）。
- **Windows 側残置責務**: git credential manager（GCM Core）、SSH agent、ブラウザ、VS Code ホスト UI のみ Windows 側に残す。toolchain（Rust / Go / Node / kubectl 等）は WSL2 / Dev Container 側に集約する。
- **リポジトリ配置**: 開発リポジトリは WSL2 distribution の ext4 上（`/home/<user>/...`）に clone する。`/mnt/c/...` への配置は禁止（9p ペナルティのため）。

### スコープ

本 ADR は **Windows 11 + WSL2 ホスト**の Docker ランタイム選定のみを扱う。次は本 ADR の対象外であり、別途扱う。

- macOS ホスト・純 Linux ホスト（リリース時点は WSL2 利用率が支配的なため、必要が生じた段階で別 ADR を起票）
- Dev Container 内 toolchain の構成（[ADR-DEV-001](./ADR-DEV-001-paved-road.md) と IMP-DEV-DC-* 系列）
- kind / k3d / Dapr Local の実装詳細（IMP-DEV-DC-014 / IMP-DEV-POL-006）
- Scaffold CLI の起動環境（IMP-DEV-SO-* 系列）

## 検討した選択肢

### 選択肢 A: WSL2 distribution 内に docker-ce をネイティブ導入（採用）

- 概要: Ubuntu 24.04 LTS の WSL2 distribution に Docker 公式 apt リポジトリから `docker-ce` を導入し、systemd で起動する。
- メリット:
  - **OSS 経路でライセンス遵守が永続的に成立**する。採用側組織が拡大しても契約面の見直し不要。
  - **VM 階層が最小**（Windows host → WSL2 distribution → コンテナの 2 層）で、kind 起動時の階層数が Docker Desktop 比で 1 層減る。
  - **bind mount の I/O が同一 distro 内 ext4 直結**となり、cargo build / rust-analyzer の応答性が改善する。
  - リソース制御が `.wslconfig` 一元化され、設定の整合点が単一に保たれる。
  - `/var/lib/docker` が利用者 distro の rootfs に配置され、storage driver の状態を `du` / `find` で直接観察できる。
  - systemd で `journalctl -u docker` が利用でき、daemon ログの確認が標準的な Linux 手順に揃う。
  - Docker Inc. のテレメトリ・proprietary 拡張に依存しない構成が取れる。
- デメリット:
  - 初期セットアップで apt リポジトリ追加・GPG キー登録・group 参加・systemd 有効化の数手順を踏む必要がある（Docker Desktop は GUI ウィザード一発）。
  - Kubernetes クラスタ機能（Docker Desktop 内蔵の単一ノード Kubernetes）が利用できない。kind / k3d を別途起動する必要がある（k1s0 では IMP-DEV-POL-006 で kind / k3d 必須のため実害なし）。
  - GUI でコンテナを管理する標準ツールがない（CLI / VS Code 拡張で代替）。

### 選択肢 B: Docker Desktop for Windows

- 概要: Docker Inc. の商用製品。WSL2 backend モードで内部に専用 distribution `docker-desktop` を立て、その上で dockerd を動かす。GUI とライセンス管理を含むパッケージ製品。
- メリット:
  - GUI ウィザード一発でインストール・起動が完了する初期体験。
  - Kubernetes クラスタを Settings から有効化できる（個人開発の小規模実験向け）。
  - Docker Desktop Extensions（pgAdmin / Snyk 等の GUI 統合）が利用できる。
  - サポート契約により企業統制を受けやすい（一部組織の調達要件）。
- デメリット:
  - **商用ライセンスが採用側組織の規模を制約する**。従業員 250 名超 / 年間売上 1,000 万米ドル超で有償化、ライセンス遵守の継続的な事務負荷が発生する。
  - **VM 階層が増える**。利用者 distribution と `docker-desktop` distribution が別のため、bind mount でクロス distro 9p hop が発生し、リポジトリ操作の I/O が遅い。
  - **kind 起動時の階層が深い**。「Windows VM → docker-desktop distro → kind ノード」の 3 層となり、ネットワーク経路の debug が複雑化する。
  - リソース割当が `.wslconfig` と Docker Desktop Settings の二重管理。
  - テレメトリと自動更新が daemon に組み込まれており、企業ネットワーク制約下で更新が止まると気付きにくい。
  - 内部 VM のファイルが Windows / 利用者 distro の両側から不可視で、debug 経路が `docker exec` 一択になる。

### 選択肢 C: Rancher Desktop

- 概要: SUSE 提供の OSS デスクトップ製品。内部に k3s を抱え、container エンジンとして moby または containerd を選択可能。
- メリット:
  - OSS（Apache 2.0）であり商用ライセンス問題が無い。
  - k3s が同梱されており、Kubernetes 単一ノードクラスタを GUI で起動可能。
  - dockerd 互換 socket を提供するため Dev Container との互換性は保たれる。
- デメリット:
  - **VM 階層が Docker Desktop と同等**。利用者 distribution と Rancher Desktop 内部 VM が別のため、bind mount のクロス distro hop は解消されない。
  - **k3s が常駐する**ため、k1s0 の標準である kind と二重に Kubernetes クラスタが立ち上がりリソースを食う。kind に統一するなら k3s 同梱は逆に negative。
  - Rancher Desktop 自身の更新サイクルに依存する追加のメンテナンスポイントが増える。
  - Windows ホストで GUI を常駐させる必要があり、`.wslconfig` 単独管理の利点を活かせない。

### 選択肢 D: Podman / Podman Desktop

- 概要: Red Hat 主導の OSS。daemonless / rootless が特徴で、`docker` CLI 互換 alias を提供する。
- メリット:
  - daemonless により root 権限の常駐 daemon が不要、セキュリティ面の attack surface が小さい。
  - rootless で multi-tenant 運用に向く（個人開発では恩恵限定）。
  - OSS（Apache 2.0）であり商用ライセンス問題が無い。
- デメリット:
  - **kind / k3d との互換性が完全ではない**。kind の Podman provider は experimental ステータスが続いており、k1s0 の Paved Road（IMP-DEV-POL-006）が前提とする本番再現性を担保しきれない。
  - **Dapr CLI / Dapr sidecar の検証が docker-ce 前提**で行われており、Podman 経路で問題が発生した場合に Dapr コミュニティの一次サポートが薄い。
  - Compose 互換が `podman-compose` 経由で部分的、`docker compose` 標準と挙動差分が残る。
  - Buildx の互換性が docker-ce / Docker Desktop に劣り、multi-platform build で差分が出る。

### 選択肢 E: WSL2 + Docker Desktop の併用（共存）

- 概要: WSL2 distribution に docker-ce を入れつつ、Docker Desktop も並行インストールして場面で切替える。
- メリット: GUI が必要な場面と性能が必要な場面を使い分けられる。
- デメリット:
  - **Docker Desktop 起動時に WSL2 上の Docker context が `desktop-linux` に切替わり**、利用者の意図と異なる daemon に接続する事故が頻発する。
  - 二重管理の複雑性が単純加算で増え、トラブルシュート時に「どちらの daemon に接続しているか」を毎回確認する必要が出る。
  - ライセンス上、Docker Desktop を一度でも起動すると organization の利用実績として計上されうる（採用側組織の compliance ポリシーに抵触する可能性）。
  - Paved Road の「単一の正しい道」原則（IMP-DEV-POL-001）に反する。

## 決定理由

選択肢 A（WSL ネイティブ docker-ce）を採用する判断は、以下の比較軸で他の選択肢を退けた結果である。

- **ライセンス耐性（B / E を退けた最大の理由）**: k1s0 は採用側組織の小規模運用から拡大段階を 10 年保守する。Docker Desktop の商用ライセンス境界（250 名 / 1,000 万米ドル）は組織拡大の途上で確実に踏むため、その時点でランタイム選定をやり直す移行コストが発生する。最初から OSS 経路で揃えれば、この境界条件が永続的に発生しない。OSS であれば B（Docker Desktop）が選ばれる積極的理由は GUI のみで、それは A の不利点を上回らない。
- **VM 階層の最小化（B / C を退けた理由）**: kind を Paved Road の本番再現基盤として採用している（IMP-DEV-POL-006）以上、kind コンテナの起点となる Linux ホストはできる限り浅い階層に置きたい。A は Windows ホスト → WSL2 distro → kind の 2 層、B / C は中間に専用 VM が挟まる 3 層。階層数の差は port-forward / dapr sidecar 接続性の debug コストに直結する。
- **bind mount I/O 性能（B を退けた理由）**: Rust 開発（tier1）は cargo build / rust-analyzer の応答性が DX に直結する。同一 distro 内 ext4 直結（A）と、クロス distro 9p hop（B）の差は、初回 build / インクリメンタル build の両方で体感できる差分を生む。実測値はホスト構成に依存するが、構造的に A が優位である事実は変わらない。
- **kind / Dapr 互換性（D を退けた理由）**: Podman の rootless / daemonless は技術的に魅力だが、k1s0 が採用する Dapr / kind / Helm chart の検証は docker-ce 前提で行われている。Podman 経路で発生する互換性問題の一次サポートが薄く、Paved Road の「正しい道に乗れば Platform チームが全責任を負う」（ADR-DEV-001）という支援境界線を維持できない。
- **k3s との重複回避（C を退けた理由）**: Rancher Desktop は k3s を同梱するが、k1s0 は kind を標準としているため、k3s が常駐するメリットがない。k3d を選ぶ場面でも `k3d` CLI を直接利用すれば良く、Rancher Desktop の GUI を経由する必要はない。
- **単一の正しい道（E を退けた理由）**: Paved Road 思想（ADR-DEV-001 / IMP-DEV-POL-001）の根幹は「正しい経路を一本にし、迷いを構造的に消す」ことである。Docker Desktop と docker-ce の共存は、この思想に正面から反する。

これらの軸を総合した時、A（WSL ネイティブ docker-ce）以外を選ぶ積極的理由は、Windows ユーザーの GUI 慣れによる初期セットアップの心理的障壁緩和のみである。これは初回 1 時間程度のドキュメント整備（[`docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/`](../../05_実装/50_開発者体験設計/05_ローカル環境基盤/)）で十分に補える可逆コストであり、A の永続的優位を覆す根拠にならない。

## 影響

### ポジティブな影響

- 採用側組織の規模拡大時に Docker Desktop ライセンス境界を踏まない。事業継続性が技術選定に縛られない構造になる。
- kind / k3d / Dapr Local の検証経路が同一 distribution 内に閉じ、ネットワーク経路と権限境界が単純化する。
- リポジトリの cargo / Go / Node の I/O が同一 ext4 内で完結し、tier1 Rust 開発の応答性が構造的に改善する。
- `.wslconfig` 一元化により、リソース調整が単一ファイルレビューで完結する。新規参画者のオンボーディングで「どこを見れば資源配分が分かるか」が一意になる。
- `journalctl -u docker` で daemon ログが Linux 標準手順で取れる。インシデント対応 Runbook を Linux 一般知識に揃えられる。
- Docker Inc. のテレメトリ送信を排除でき、企業ネットワークのアウトバウンド統制が単純化する。
- ADR-DEV-001 の「正しい道一本化」原則が、ホスト Docker 層でも貫徹される。

### ネガティブな影響 / リスク

- 初期セットアップの手順数が Docker Desktop 比で増える。GUI ウィザードに慣れた Windows ユーザーには学習コストが発生する。緩和策として手順書（`docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/`）を整備する。
- Docker Desktop の Kubernetes 単一ノード機能・Extensions（pgAdmin GUI 等）が利用できない。代替を CLI / VS Code 拡張に揃える必要がある（k1s0 標準ツール群で代替可能）。
- WSL2 distribution が破損した場合、Docker Desktop のように「アプリ再インストールで復旧」というシンプルな経路がない。distribution の export / import によるバックアップ手順を Runbook 化する必要がある。
- macOS / 純 Linux ホストの開発者が将来加わった場合、本 ADR は対象外となるため別 ADR を起票する必要がある。リリース時点では WSL2 利用率が支配的なため許容するが、想定漏れリスクとして残る。
- nestedVirtualization が必須となるため、Hyper-V を無効化している組織配布端末では `.wslconfig` の `nestedVirtualization=true` が効かない可能性がある。組織 IT との事前合意が必要。

### 移行・対応事項

- [`docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/`](../../05_実装/50_開発者体験設計/05_ローカル環境基盤/) に Windows + WSL2 + docker-ce の標準構成設計書を配置する（IMP-DEV-ENV-060〜065）。
- [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md) の 119 行目「Windows / macOS は Docker Desktop または Rancher Desktop」記述を、本 ADR への参照に差し替える。
- `docs/05_実装/50_開発者体験設計/90_対応IMP-DEV索引/01_対応IMP-DEV索引.md` に新接頭辞 `ENV` を追加し、IMP-DEV-ENV-060〜065 を登録する。
- `docs/05_実装/99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md` に ADR-DEV-002 セクションを追加し、IMP-DEV-ENV 系列との直接対応を記録する。
- `docs/02_構想設計/adr/README.md` の「開発者体験」節を 2 件 → 3 件に更新する。
- `docs/04_概要設計/90_付録/02_ADR索引.md` に ADR-DEV-002 の詳細索引行を追加する。
- WSL2 distribution の export 手順（`wsl --export`）と import 手順を `docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/` の Runbook 節に Phase 2 で整備する。
- Windows 端末配布段階で組織 IT に「Hyper-V 有効化」「WSL2 利用許可」「docker-ce 経路の使用許可」を事前合意する手順を `50_オンボーディング/` に組み込む。
- macOS / 純 Linux ホストへの拡張が必要になった時点で本 ADR を Superseded せず、新 ADR（ADR-DEV-003 を予約）として独立起票する方針を `02_構想設計/adr/README.md` 末尾に注記する。

## 参考資料

- [ADR-DEV-001: 開発者体験の根幹思想として Paved Road を採用](./ADR-DEV-001-paved-road.md)
- [ADR-DIR-003: Sparse Checkout cone + partial clone を標準化](./ADR-DIR-003-sparse-checkout-cone-mode.md)
- [ADR-CICD-001: Argo CD で GitOps 配信を行う](./ADR-CICD-001-argocd.md)
- [ADR-CICD-003: Kyverno で admission ポリシーを強制する](./ADR-CICD-003-kyverno.md)
- [docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md](../../05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md)
- [docs/05_実装/50_開発者体験設計/00_方針/01_開発者体験原則.md](../../05_実装/50_開発者体験設計/00_方針/01_開発者体験原則.md)
- Microsoft Dev Blog: "Systemd support is now available in WSL"（2022-09-22 公開、systemd 導入の Stable サポート）
- Docker, Inc.: "Docker Desktop License Agreement"（従業員 250 名 / 年間売上 1,000 万米ドルの境界）
- Docker, Inc.: "Install Docker Engine on Ubuntu"（公式 apt 手順）
- WSL Configuration Reference (Microsoft Learn): `.wslconfig` の `[wsl2]` セクション仕様
