# L6 portability 検証結果（kind 以外の vanilla K8s 実装）

本書は ADR-TEST-001 で位置づけられた L6 portability 検証結果を記録する live document。`tools/qualify/portability/run.sh` の実走結果（multipass + kubeadm + Calico の 3-node cluster で k1s0 が動くか）を時系列で残す。

## 本書の位置付け

ADR-CNCF-001（vanilla K8s 維持）/ ADR-INFRA-001（kubeadm 採用）と整合する形で、kind 以外の vanilla K8s 実装で k1s0 が動作することを検証する。マネージド K8s（EKS / GKE / AKS）での実走は採用組織側の責務で、本リポジトリでは local 完結する multipass + kubeadm + Calico 経路を正典とする。

k3s 派生（k3d 含む）は ADR-CNCF-001 で「次点」と判定済のため不採用。本書に k3s 系結果は記録しない。

## 検証経路

`tools/qualify/portability/run.sh` が以下を実行する:

1. multipass で Ubuntu 24.04 VM 3 台（control-plane 1 + worker 2）起動
2. 各 VM に containerd + kubeadm/kubelet/kubectl install（pkgs.k8s.io 公式）
3. control-plane で `kubeadm init`（pod-network-cidr=192.168.0.0/16）
4. worker 2 台が `kubeadm join`
5. Calico CNI install（tigera-operator 経由、ADR-NET-001 と整合）
6. 全 node Ready 待機（最大 5 分）
7. cluster-info（kubectl version / nodes / get all / tigerastatus）を artifact 化
8. trap で multipass VM 削除（--keep-cluster で残す）

## 検証結果サマリ

### 2026-05（リリース時点 / 初期、実走前）

- **状態**: `tools/qualify/portability/run.sh` 実装完了。実走は multipass + kubectl が host に install された環境で行う必要があり、devcontainer 内では nested virtualization 制約で動かない。host OS（WSL2 / macOS / Linux）から実行する Runbook を採用初期で `ops/runbooks/RB-PORT-001-multipass-kubeadm-portability.md` として整備予定。
- **完了済**: `tools/qualify/portability/run.sh` 実装 / `tools/local-stack/up.sh --role conformance` 追加（cni layer 経由で Calico install 想定） / L6 portability 定義は本ファイルが正典（k3s 妥協を退けた）
- **次期**: 採用初期で host OS 上の手動 1 回実走、結果を本書に追記

### 2026-05-03 訂正（過去 commit `2b78364bc` の portability 主張を撤回）

過去 commit `2b78364bc` の commit message および当時の e2e 結果ログで **「portability 代替経路 PASS」** と記録したが、これは portability の定義に該当しない。本節で訂正する（当該 e2e 結果ログは e2e テスト基盤刷新で削除済）。

- **何が間違っていたか**: 「同 cluster の別 namespace に fresh helm install + e2e PASS」を portability 達成として記録した。同 K8s 実装（同 kind cluster）の中で chart を新しい release / namespace に install するだけで、**「異なる K8s 実装で manifest が動く」という portability の本質を一切検証していない**。これは **chart 再現性の証拠**であって portability ではない
- **正しい portability 定義**: ADR-CNCF-001 / ADR-TEST-001 の通り、kind 以外の vanilla K8s 実装（multipass + kubeadm 等）で k1s0 chart が動くこと
- **訂正後の真の到達点**: 本セッション時点で portability **未達**（runner 実装あり、実走前のまま）
- **本セッションでの追加発見**: host に `/dev/kvm` 存在 + `/proc/cpuinfo` に `vmx` flag を確認、当初判定「nested virt 制約で multipass 不可」は誤り。**user の sudo password 提供で `sudo snap install multipass` を 1 回実行すれば既設 run.sh が完走できる**経路を確認
- **証跡**: `tests/.portability/2026-05-02/namespace-redeploy.txt` は portability 証跡ではなく **chart 再現性証跡**として読み替える（ファイル自体は残し、本節と相互参照）
- **次期**: 同セッション後半で multipass install + run.sh 完走を実走、本物の K8s（kubeadm + Calico）で全 e2e PASS まで持っていく

## 検証結果 template（採用初期で本テンプレに従って追記）

```markdown
### YYYY-MM-DD

- **K8s version**: v1.NN.M
- **Calico version**: vX.YY.Z
- **VM 構成**: 4 GB RAM × 3 / 2 CPU × 3 / 20 GB disk × 3
- **kubeadm init 所要**: M 分
- **kubeadm join × 2 所要**: K 分
- **Calico install + 全 node Ready 所要**: L 分
- **合計所要**: 約 N 分
- **cluster-info artifact**: tests/.portability/YYYY-MM-DD/cluster-info.txt
- **conformance-link**: tests/.portability/YYYY-MM-DD/conformance-link.md
- **判定**: PASS / FAIL（失敗 step / 根本原因 / 修正対応）
```

## 採用初期での拡張

- `tools/qualify/conformance/run.sh --skip-up` を本 cluster の KUBECONFIG で実行し、L5 Sonobuoy 結果を取得 → kind cluster 結果と一致確認
- 異なる K8s version（N-2 / N-1 / N）での 3 並列実行
- 採用組織側が EKS / GKE / AKS で実走する手順を `docs/governance/PORTABILITY-FOR-ADOPTERS.md` で公開

## 関連

- ADR-TEST-001（Test Pyramid + L6 portability 例外）
- ADR-CNCF-001（vanilla K8s + CNCF Conformance 維持、k3s 退ける根拠）
- ADR-INFRA-001（kubeadm + Cluster API 採用）
- ADR-NET-001（CNI 選定、kind multi-node = Calico、portability も Calico）
- `tools/qualify/portability/run.sh`
- `tools/qualify/conformance/run.sh --skip-up`（採用初期で統合）
