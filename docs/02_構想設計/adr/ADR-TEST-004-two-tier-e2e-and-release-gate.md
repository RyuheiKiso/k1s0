# ADR-TEST-004: E2E を kind + multipass kubeadm の二層構造で実装し、release tag 強制を 3 重防御で物理化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / 開発者体験チーム

## コンテキスト

ADR-TEST-003 で qualify を 11 層に分解したが、各層が必要とする Kubernetes クラスタは大きく二系統に分けられる。**速度重視の kind**（L3 smoke / L4 standard / L7 chaos）と **本番 fidelity 重視の実 kubeadm**（L5 conformance / L9 upgrade / L10 DR）である。両者を一つのクラスタ実装でカバーしようとすると、kind だけでは LoadBalancer / 本番 CNI / 永続 PV / etcd backup-restore のような本番 fidelity が取れず、逆に kubeadm だけでは pre-push の 5 分制約や PR ごとの fast feedback が成立しない。

ADR-INFRA-001 で本番クラスタが Cluster API + kubeadm（vanilla K8s）と決定済であり、ローカル conformance を kubeadm に揃えれば本番 fidelity が一致する。ADR-CNCF-001 で「vanilla K8s + CNCF Conformance 維持」が宣言されているため、k3s / RKE2 / Talos のようなディストリビューション派生はローカルテストにも持ち込めない。残る選択肢は「kind は速度層、実 kubeadm は本番 fidelity 層」という二層構造である。

実 kubeadm をローカルに立てる手段は実用上 ① multipass（Canonical 製、Ubuntu VM をホストで動かす）② lima（macOS 寄り、WSL2 ではやや不安定）③ vagrant + VirtualBox（Windows 上では Hyper-V と競合）④ self-host bare-metal（追加ハードウェア前提）⑤ KubeVirt 等の K8s on K8s（kind の上に kubeadm cluster を立てるが循環的で複雑）の 5 案がある。ADR-DEV-002 で「Windows 11 + WSL2 + Docker Desktop」をリファレンスとしているため、WSL2 と macOS の両方で素直に動くものが望ましい。

加えて、CI を持たない以上、**release tag を切る行為そのものを qualify 強制に物理的に紐付ける**経路が必要である。`git tag v0.1.0` のような単純なコマンドが qualify を走らせずに通ってしまうと、ADR-TEST-001 の決定（qualify report が必ず release artifact に同梱される）が成立しない。CI 環境であれば「tag push を webhook で検知 → CI ジョブで qualify 走らせる → 失敗なら release ブロック」のように runner 側で強制できるが、CI なし環境では強制点を **開発者の手元** に置くしかない。手元で強制するには、自然な短絡経路（`git tag` 直接実行 / hook 無効化 / wrapper 経由しない）を全部塞ぐ必要があり、単発の hook では bypass されうる。

選定では以下を満たす必要がある:

- **二層 E2E のクラスタ実装** が WSL2 / Linux / macOS（補助機）の 3 環境で動く
- **本番 fidelity の確保** が ADR-INFRA-001（本番 = kubeadm）と一致する
- **release tag 強制経路** が `git tag` の直接実行 / hook 無効化 / wrapper bypass の 3 短絡経路すべてを塞ぐ
- **CI 移植容易性** が ADR-TEST-001 portable 制約 1（POSIX shell + Make）と整合する
- **協力者が起案者不在で立ち上げられる** Runbook 化が ADR-OPS-001 と整合する

## 決定

**E2E は kind と multipass で立てる kubeadm 3-node cluster の二層構造で実装する。** kind は速度層（L3 / L4 / L7）、multipass kubeadm は本番 fidelity 層（L5 / L9 / L10）を担当する。

| 層 | クラスタ実装 | 構成 |
|----|------------|------|
| L3 smoke | kind | 1 control-plane + 1 worker、Docker network 共有 |
| L4 standard | kind | 1 control-plane + 3 worker、CNI を Cilium に揃える（kindnet ではなく） |
| L7 chaos | kind | L4 と同構成 + Chaos Mesh helm chart |
| L5 conformance | multipass kubeadm | 3 control-plane + 2 worker（HA、stacked etcd）、Cilium / Longhorn / MetalLB の本番セット |
| L9 upgrade | multipass kubeadm | L5 と同構成、N-2 → N → N+1 のローリング |
| L10 DR | multipass kubeadm | L5 と同構成 + minio + Velero、etcd PITR 検証 |

multipass を選択する。WSL2 上では `multipass` の native binding（hyper-v provider）で 3 VM を立てる。Apple Silicon では QEMU backend を使う。Linux 直では LXD or QEMU backend を使う。3 環境すべてで `multipass launch` が同じ CLI で動作するため、`tools/qualify/cluster/multipass.sh` を環境差異吸収レイヤとして実装する。

**release tag 強制は 3 重防御で物理化する。**

1. **第 1 層: `core.hooksPath = .githooks` 強制**
   `tools/devcontainer/post-create.sh`（ADR-TEST-002）で `git config core.hooksPath .githooks` を必ず設定する。devcontainer を使う限り、`.git/hooks/` ではなく `.githooks/` 配下のフックが走る。`.githooks/` はリポジトリ管理下にあるため、フック自体を版管理できる。

2. **第 2 層: `.githooks/pre-push` で `make qualify-pre-push` 強制**
   pre-push hook が `make qualify-pre-push`（L3 smoke、5 分以内）を必ず実行し、失敗なら push を拒否する。`git push --no-verify` で hook を bypass される可能性があるため、`tools/git-wrapper/git` で `--no-verify` 引数を検出して exit 1 する wrapper を `$PATH` 先頭に配置する（devcontainer の Dockerfile で `/usr/local/bin/git` を本物の git にシンボリックリンクせず wrapper にする）。

3. **第 3 層: `tools/release/cut.sh` を `git tag` の唯一の入口とする**
   release tag を切る正規手順を `tools/release/cut.sh <version>` の wrapper だけに限定する。スクリプトは以下を順次実行:
   - `git status` クリーン確認
   - `make qualify-release`（L0–L5 + L7 + L9 + L10）を強制実行、失敗なら exit 1
   - qualify report を `tests/qualify-report/<version>/` に保存し、tar.zst で固める
   - `git tag -a <version> -m "qualify: <sha256-of-report>"` で tag のメッセージに qualify report の hash を埋め込む
   - tag push 後に GitHub Release に qualify-report.tar.zst を添付する手順を runbook で明示
   - 直接 `git tag` を叩くと第 1 層の wrapper が検出して警告を出す（完全には塞げないが、明示的に違反した行為になる）

リリース時点では tag を打つのは起案者一人だが、Phase 2 で contributor が増えた段階で `tools/release/cut.sh` の責務分担（誰が tag を切れるか）を `docs/governance/RELEASE-PROCESS.md` で明文化する。本 ADR では強制経路の物理化のみを正典化する。

## 検討した選択肢

### 選択肢 A: kind 単一層（実 kubeadm を採用しない）

- 概要: L5 conformance も L9 upgrade も L10 DR もすべて kind で実装し、本番 fidelity 検証を諦める
- メリット:
  - クラスタ実装が 1 系統で済み、起動時間も短い（kind は 30 秒〜2 分）
  - WSL2 / macOS / Linux で `kind create cluster` が同じ CLI で動く
  - 開発者の認知負荷が最小
- デメリット:
  - **本番 fidelity が原理的に取れない**: kind は kindnet（kindnet-only network）/ local-path provisioner（CSI 簡易）/ extraPortMappings による LB 模倣 で動いており、Cilium eBPF / Longhorn replication / MetalLB BGP のような本番挙動は再現できない
  - **ADR-CNCF-001 と矛盾**: vanilla K8s + CNCF Conformance を宣言しながら、L5 conformance を kind で取ると Sonobuoy 結果が「kind 上で取った Conformance」になり、本番 cluster との等価性が証明できない
  - **ADR-INFRA-001 と乖離**: 本番が kubeadm なのにローカルテストに kubeadm が無いと、起案者が本番にしかない問題に気づくのが採用組織のフィードバック後になる
  - L9 upgrade（kubelet rolling restart）/ L10 DR（etcd backup-restore）は kind では構造的に成立しない

### 選択肢 B: kind + lima の二層

- 概要: kind を速度層、lima（Linux VM on macOS / WSL2）を本番 fidelity 層として使い、lima 上で kubeadm を立てる
- メリット:
  - lima は macOS 環境では実績豊富で、Apple Silicon ネイティブ
  - QEMU backend で arm64 ネイティブ動作
  - `limactl start` で K8s 込みのテンプレートが提供される
- デメリット:
  - **WSL2 上での安定性に課題**: lima は基本的に macOS 想定で、WSL2 上で QEMU を nested virtualization で動かす経路が公式サポート外。リファレンス環境（WSL2）で primary clusters が立たないリスク
  - **kubeadm 直叩きに追加レイヤ**: lima が提供する `nerdctl + containerd` 上で kubeadm を立てる構成で、ADR-INFRA-001 の本番（systemd unit + containerd）と微妙に異なる
  - 起案者の WSL2 環境で安定動作する保証がリリース時点で無く、Phase 0 の品質が崩れるリスク

### 選択肢 C: kind + multipass kubeadm の二層（採用）

- 概要: kind を速度層、multipass で立てる Ubuntu VM 3 台 + kubeadm を本番 fidelity 層として使う
- メリット:
  - **WSL2 上で安定動作**: multipass は WSL2 上の Hyper-V provider で公式サポートされており、`multipass launch` が素直に動く
  - **macOS 上でも安定**: macOS では QEMU backend で Apple Silicon ネイティブ動作、Intel Mac では HyperKit
  - **Linux 直でも動く**: LXD or QEMU backend で公式サポート
  - **vanilla Ubuntu + kubeadm**: VM の上で本物の Ubuntu が動き、その上で kubeadm を直叩きできるため、ADR-INFRA-001 の本番（Cluster API + kubeadm on Ubuntu）とほぼ同じ構成を再現できる
  - Canonical 製で license は Apache 2.0、長期保守の観点で問題なし
- デメリット:
  - **VM 起動時間が長い**: multipass で 3 VM を立てて kubeadm init/join まで含めて 5〜10 分かかる。これは pre-push hook では現実的でないため、L5 / L9 / L10 は `make qualify-release` の長い gate に紐付ける（`make qualify`（L4 まで）には含めない）
  - **メンテナンス頻度の変動**: multipass は Canonical の OSS だが、最近のリリース頻度が rolling release ベースで予測しにくい時期がある。代替として lxd-based bootstrap も `tools/qualify/cluster/lxd.sh` として用意し、multipass が長期メンテで停滞した場合の退路を残す
  - **VM image の disk 消費**: 3 VM × 20GB = 60GB を qualify cluster だけで消費する。ADR-TEST-002 のハードウェア要件（NVMe 1TB / 空き 200GB）で吸収する

### 選択肢 D: k3s / k3d の単一層

- 概要: k3s（Rancher Labs 製の軽量 K8s ディストリビューション）または k3d（k3s in Docker）を全層で使う
- メリット:
  - 軽量で起動が速い（k3d は kind と同等、k3s は VM で 30 秒）
  - 1 binary で K8s 構成要素（kube-apiserver / kubelet / etcd / containerd）を吸収
  - HA 構成も embedded etcd でシンプル
- デメリット:
  - **vanilla K8s 派生**: k3s は SQLite or embedded etcd / Traefik 内蔵 / Flannel 内蔵 / klipper-lb 内蔵で、vanilla K8s から複数の component を入れ替えており、ADR-CNCF-001（CNCF Conformance 維持）と矛盾する
  - **採用検討者が「k3s でテストして本番は kubeadm」を懸念**: ローカル fidelity が本番と異なるため、L5 conformance の証跡が k3s の Conformance になる
  - Rancher / SUSE への依存が強まり、採用組織のスキル流用性で kubeadm に劣る

## 決定理由

選択肢 C（kind + multipass kubeadm 二層）を採用する根拠は以下。

- **本番 fidelity の確保**: ADR-INFRA-001 で本番が kubeadm、ADR-CNCF-001 で vanilla K8s + CNCF Conformance 維持と決定済。multipass + kubeadm は本番と同じ Ubuntu + kubeadm + containerd の組み合わせを VM 単位で再現できるため、L5 conformance の Sonobuoy 結果が本番 cluster の Conformance と等価性を持つ。選択肢 A / D は本番 fidelity が原理的に取れず、選択肢 B は構成が部分的に異なる
- **3 環境での安定動作**: ADR-DEV-002 のリファレンス（WSL2）と補助機（Apple Silicon）で multipass が公式サポートされており、Linux 直も網羅する。選択肢 B（lima）は macOS では強いが WSL2 で不安定なリスクが残り、Phase 0 のリリース時点で起案者の手元で安定動作する保証が薄い
- **kubeadm 直叩きの単純性**: multipass で立てた Ubuntu VM の上で `kubeadm init` を直叩きできるため、ADR-INFRA-001 の本番ブートストラップと同じ手順をローカルで実行できる。これは協力者が起案者不在で cluster を立てる Runbook（RB-OPS-002）の難易度を最小化する効果もある（バス係数 2 と整合）
- **二層分離の認知合理性**: kind = 速度・kubeadm = 本番 fidelity という分離は CNCF Graduated 級 OSS（Cilium / Istio）の慣行と一致しており、採用検討者が「kind だけだと薄い、kubeadm まで取っているのは硬派」と評価できる。選択肢 A はこの評価軸で失格、選択肢 D は ADR-CNCF-001 違反で採用検討者の信頼を失う
- **release tag 3 重防御の物理化**: `core.hooksPath` 強制 + `pre-push` hook + `cut.sh` wrapper の 3 経路で、`git tag` 直接実行 / hook 無効化 / wrapper bypass の 3 短絡経路すべてを塞ぐ。CI 環境では runner が強制点だが、CI なし環境では「開発者の手元で物理的に塞ぐ」しか方法がなく、3 重防御はその最低限の網である。単発の hook では `--no-verify` で簡単に抜けられるため、wrapper + hookspath の組み合わせが必要
- **退路の確保**: multipass が長期メンテで停滞した場合、`tools/qualify/cluster/lxd.sh` を代替として用意することで Phase 移行リスクを局所化できる。クラスタ実装の差し替えが `tools/qualify/cluster/` ディレクトリ内で完結するため、qualify 全体の書き換えにならない（ADR-TEST-001 portable 制約と整合）

## 影響

### ポジティブな影響

- L5 conformance の Sonobuoy 結果が本番 kubeadm cluster と等価性を持ち、ADR-CNCF-001 の Conformance 宣言の証跡が成立する
- L9 upgrade / L10 DR が本番手順（kubelet rolling restart / etcd backup-restore）と同じ操作で実行でき、本番リリース前に問題を検知できる
- WSL2 / macOS / Linux の 3 環境で同じ CLI（`tools/qualify/cluster/multipass.sh`）でクラスタが立ち上がるため、起案者・協力者・採用検討者の試走経路が単純化する
- release tag 強制が 3 重防御で物理化され、CI なし環境でも qualify 走行なしの release tag が原理的に切れない（少なくとも明示的な違反操作になる）
- `tools/release/cut.sh` が release tag のメッセージに qualify report の sha256 hash を埋め込むため、tag と report の対応が改ざん検知可能になる
- multipass の代替として `tools/qualify/cluster/lxd.sh` を用意することで、Canonical のメンテ動向リスクが Phase 移行で局所化される

### ネガティブな影響 / リスク

- multipass で 3 VM を立てて kubeadm init/join まで完了するのに 5〜10 分かかり、L5 / L9 / L10 の起動時間が pre-push 制約（5 分以内）を超える。これは設計上の前提（`make qualify-release` の長い gate に L5/L9/L10 を紐付ける）で吸収するが、開発者が「release qualify が遅い」と感じる体感は残る
- multipass は Canonical の OSS で、メンテナンス頻度が時期により変動する。Phase 移行で multipass が停滞した場合に lxd または vagrant への移行が必要になるリスクは残る
- VM image の disk 消費（3 VM × 20GB = 60GB）が qualify cluster だけで発生し、L8 KWOK 1000 node の追加 RAM と合わせて、ADR-TEST-002 のハードウェア要件（NVMe 1TB / RAM 32GB）の上限近くで動く。低スペックマシンでは L5 / L9 / L10 が回らない
- 3 重防御は「明示的に違反する人」を想定していない（git wrapper を `$PATH` から外して `git tag` を直接叩けば突破できる）。リリース時点では起案者一人なので強制力で十分だが、Phase 2 で contributor が増えた段階で `docs/governance/RELEASE-PROCESS.md` で社会的な強制（PR レビューでの規律遵守確認）を追加する必要がある
- multipass は Hyper-V を使う構成で、Windows + WSL2 上で nested virtualization の制約を受ける。Hyper-V が無効な Windows Home Edition では `multipass launch` が動かないため、ADR-TEST-002 のリファレンス機要件で Windows 11 Pro 以上を暗黙に要求する形になる

### 移行・対応事項

- `tools/qualify/cluster/kind.sh` を新設し、kind cluster 起動・破棄・kubeconfig 取得を抽象化する（L3 / L4 / L7 から呼ばれる）
- `tools/qualify/cluster/multipass.sh` を新設し、multipass で 3 control-plane + 2 worker の Ubuntu VM を立て、kubeadm init/join を実行する（L5 / L9 / L10 から呼ばれる）
- `tools/qualify/cluster/lxd.sh` を退路として整備し、multipass が停滞した場合の代替経路を用意する（リリース時点では実装不要、Phase 1 以降で必要時に充足）
- `.githooks/pre-push` を新設し、`make qualify-pre-push` を呼ぶラッパを設置する（ADR-TEST-003 と整合）
- `.githooks/pre-commit` を新設し、`make qualify-pre-commit` を呼ぶラッパを設置する
- `.devcontainer/post-create.sh` に `git config core.hooksPath .githooks` を追加し、devcontainer 起動時に hooksPath を強制設定する
- `tools/git-wrapper/git` を新設し、`/usr/local/bin/git` を本物の git ではなく wrapper にして `--no-verify` 引数を検出する（devcontainer Dockerfile で symlink を上書き）
- `tools/release/cut.sh` を新設し、release tag を切る唯一の入口とする。`make qualify-release` 強制 + qualify report tar.zst 化 + `git tag -a` で sha256 hash 埋め込みを統合する
- `ops/runbooks/RB-OPS-002-qualify-cluster-bootstrap.md` を新設し、kind / multipass cluster の立ち上げ手順を 8 セクション形式（ADR-OPS-001 準拠）で Runbook 化する
- `docs/governance/RELEASE-PROCESS.md` を新設し、`tools/release/cut.sh` の使い方と Phase 2 以降の責務分担を採用検討者向けに公開する
- 既存 `tests/e2e/scenarios/tenant_onboarding_test.go` を `tests/e2e/L3_smoke/` に移管する（ADR-TEST-003 の移行事項と統合）
- `infra/environments/dev/` を kind 専用、`infra/environments/staging/` を multipass kubeadm 想定に整合させる（ADR-INFRA-001 / ADR-POL-002 と整合）

## 参考資料

- ADR-TEST-001（CI 留保 + qualify portable 設計）— release tag 強制の必要性、portable 制約の整合
- ADR-TEST-002（devcontainer + HW 要件）— core.hooksPath 強制 / git wrapper の実装基盤
- ADR-TEST-003（テストピラミッド L0–L10）— 各層と二層 E2E の対応
- ADR-INFRA-001（Cluster API + kubeadm）— 本番ブートストラップとローカル multipass kubeadm の整合性根拠
- ADR-CNCF-001（CNCF Conformance）— L5 conformance が満たすべき外部基準、k3s / RKE2 を退ける根拠
- ADR-NET-001（Cilium / Calico / kindnet 使い分け）— kind の CNI を本番に揃える根拠
- ADR-OPS-001（Runbook 標準化）— RB-OPS-002 の形式根拠
- ADR-POL-002（local-stack を構成 SoT に統一）— kind cluster の構成 SoT 思想を multipass にも拡張
- multipass: multipass.run
- kind: kind.sigs.k8s.io
- Sonobuoy: sonobuoy.io
- 関連 ADR（採用検討中）: ADR-TEST-005（環境マトリクス）/ ADR-TEST-006（chaos / scale / soak）/ ADR-TEST-007（upgrade / DR）
