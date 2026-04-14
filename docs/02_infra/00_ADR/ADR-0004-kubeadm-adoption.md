# ADR-0004: Kubernetes ディストリビューションとして kubeadm を採用

- ステータス: Accepted
- 起票日: 2026-04-14
- 決定日: 2026-04-14
- 起票者: kiso ryuhei
- 関係者: インフラチーム / 起案者 / 決裁者

## コンテキスト

k1s0 は **オンプレミス閉域ネットワーク** での稼働を前提とし、**稟議ハードルの低減** と **ベンダーロックインの回避** を中核原則として掲げる。
Kubernetes の構築にあたり、ディストリビューション (kubeadm / k3s / k0s / マネージド K8s 等) を選定する必要がある。

主要な制約は以下のとおり。

- Phase 1a (MVP-0) は VM 1 台での single control-plane 構成。Phase 1b (MVP-1a) で 3 ノード Control Plane HA 構成へ拡張する。
- パブリッククラウド (EKS / AKS / GKE) は稟議の難易度・データ持ち出しの観点から採用しない。
- バス係数 2 を実証するため、起案者以外の協力者が **環境を再構築できる** ことが必須。
- Phase 2 以降で Istio / Kafka / Tempo 等のエコシステム OSS を導入する予定があり、上流互換性の確実性が重視される。

## 決定

**kubeadm をメインディストリビューションとして採用** する。
具体的には以下の構成とする。

- **Phase 1a**: kubeadm 1 ノード (single control-plane)。
- **Phase 1b**: kubeadm 3 ノード HA (stacked etcd) を **kubespray** で自動構築。
- **Control Plane VIP**: kube-vip (kubespray の `kube_vip_enabled: true` で自動構成)。
- **オンプレ向け LoadBalancer**: MetalLB (L2 モード) で Service type=LoadBalancer に VIP を払い出す。
- **インフラ再現**: OpenTofu + cloud-init で VM プロビジョニングを自動化し、構築手順を完全コード化する。

k3s をサブプラン (障害時の代替検討) として位置づけるが、第一選択は kubeadm とする。

## 検討した選択肢

### 選択肢 A: k3s

- 概要: 軽量 Kubernetes ディストリビューション。組み込み・エッジ向け実績が豊富だが、本番採用例も多い。
- メリット: 単一バイナリで導入が容易。フットプリントが小さい。SQLite/組み込み etcd で運用がシンプル。
- デメリット: 上流 Kubernetes との微妙な差分 (一部コンポーネントの差し替え) があり、Phase 2 以降のエコシステム OSS との互換性検証コストが発生する可能性がある。

### 選択肢 B: k0s

- 概要: 別系統の軽量 Kubernetes ディストリビューション。
- メリット: 単一バイナリで導入容易。
- デメリット: コミュニティ規模が k3s / kubeadm に比べ小さい。本プロジェクトの長期運用想定に対し、エコシステム成熟度の観点でリスクが残る。

### 選択肢 C: マネージド K8s (EKS / AKS / GKE)

- 概要: クラウドベンダー提供のマネージド K8s。
- メリット: Control Plane の運用が不要。スケールが容易。
- デメリット: 本プロジェクトの「オンプレ閉域・稟議ハードル低減・ベンダーロックイン回避」原則に正面から反する。

### 選択肢 D: kubeadm + kubespray (採用)

- 概要: CNCF 公式の構築ツール kubeadm を kubespray (Ansible) で HA 自動化。
- メリット: 上流 Kubernetes と完全に整合し、エコシステム OSS の互換性リスクが最小。kubespray が HA 構成を自動化。
- デメリット: 軽量ディストリより運用が複雑。HA 構成では etcd の管理理解が必須。

## 決定理由

- **CNCF 公式推奨フローでエコシステム最大**: kubeadm は Kubernetes 公式の構築ツールであり、上流との差分が最小。Phase 2 以降に導入する Istio / Kafka / Tempo 等の互換性確認コストが最も小さい。
- **HA 構築の自動化が成熟している**: kubespray により 3 ノード HA (stacked etcd) を Ansible で再現可能。手作業構築によるバス係数 1 リスクを回避できる。
- **OpenTofu + cloud-init で再現性確保**: VM プロビジョニングから Kubernetes 構築までを完全コード化することで、**バス係数 2 実証** (協力者が独立して環境再構築) の前提を満たす。
- **オンプレ前提の補助 OSS が揃う**: kube-vip (Control Plane VIP) と MetalLB (Service LB) で、ベアメタル環境で必要な VIP 払い出しが完結する。これらは kubeadm との組み合わせ実績が豊富。
- **k3s はサブプラン**: 運用に行き詰まった場合の代替として有効性を認めるが、上流互換性の観点で第一選択とはしない。

## 影響

### ポジティブな影響

- 上流 Kubernetes / CNCF エコシステムとの整合性が最大化され、Phase 2 以降の OSS 導入コストが最小になる。
- kubespray + OpenTofu でインフラ全体がコード化され、バス係数 2 実証の前提を満たす。
- 人材市場における kubeadm/CNCF スキルの広さを活用でき、人材確保リスクを抑えられる。

### ネガティブな影響 / リスク

- k3s / k0s に比べて運用要素 (etcd HA / kubespray の Ansible 知識など) が多い。
  - 緩和策: kubespray により HA 構築を自動化し、運用 Runbook を Phase 1c で整備する。
- etcd 3 ノード HA の Raft 管理を理解した運用が必要。
  - 緩和策: 障害シミュレーション (フェイルオーバー演習) を Phase 1c の DoD に含め、起案者不在でも対処可能にする。
- Phase 2 で Istio 導入時に Pod / リソース消費が増える。
  - 緩和策: VM スペックを Phase 2 で 8 vCPU / 16 GB / 500 GB SSD に拡張する計画 (フェーズ計画に明記済み)。

### 移行・対応事項

- Phase 1a: kubeadm 1 ノード構成のスクリプト/手順を整備する。
- Phase 1b: kubespray inventory / OpenTofu module / cloud-init テンプレートを整備し、3 ノード HA を構築する。
- 構築・拡張・縮退・破棄の手順を Runbook 化する (Phase 1c で完成)。
- k3s をサブプランとして位置づけることを 02_要件 / Runbook に記載する。

## 参考資料

- [`../../01_企画/02_アーキテクチャ/03_配置形態.md`](../../01_企画/02_アーキテクチャ/03_配置形態.md) — オンプレ配置形態と Control Plane HA
- [`../../01_企画/04_技術選定/`](../../01_企画/04_技術選定/) — 実行基盤中核 OSS の選定根拠
- [`../../01_企画/07_ロードマップと体制/00_フェーズ計画.md`](../../01_企画/07_ロードマップと体制/00_フェーズ計画.md) — Phase 1a 〜 1b の構成段階
- [ADR-0003](./ADR-0003-kustomize-helm-strategy.md) — Kustomize + Helm 使い分け
