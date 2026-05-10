---
id: arch.infra.infra_index
axis: infra
phase: architecture
kind: index
status: draft
depends_on:
  - detail.infra.cluster_topology_conformance
  - detail.infra.clock_integrity_conformance
  - detail.infra.infra_enforcement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# infra 設計方針 index

## 一文方針
- 本フォルダは infra 軸（5 階層中の物理 / 仮想基盤 + 制御平面）の概要設計を集約する。infra は tier1 / tier2 / tier3 / data 全てが動作する物理基盤を提供し、各軸の defense-in-depth 層 E（物理 enforcement）の最終 safety net を担う。

## 至高路線における立ち位置
- 「文章運用に頼らず物理 enforcement に転写する」最終 safety net 提供層
- RLS FORCE / etcd quorum / Ceph CRUSH map / Kafka rack awareness / Kyverno admission policy / cosign signed tag は全て infra が最終的に強制
- 13 軸の各 spec の defense-in-depth 層 E は infra の物理基盤に転写される
- infra は 13 軸の 1 軸（topology）の保有層であると同時に、全 13 軸の defense-in-depth 層 E を実装する基盤層という二重の役割

## 至高路線における原則
- **単一 OSS 深耕**: cluster 1 つに対して採用する OSS は 1 種類に絞り、横並びの選択肢は持たない
- **宣言的真**: cluster 状態 / topology / OSS バージョン / drill 履歴は GitOps（Argo CD）と build artifact（lock.yaml）でのみ表現
- **不可逆性の禁止**: cluster 廃棄は OpenTofu replay で再構築可能であること
- **段階的 release 禁止**: 1.0.0 時点で 5 topology_class 全てが drill green であること
- **「現実的に運用が楽」を理由に scope を縮小しない**

## 配下ドキュメント

| 番号 | ドキュメント | 主題 |
|---|---|---|
| 01 | [クラスタ構成方針](01_クラスタ構成方針.md) | k8s minor version skew ≤ 1、upgrade window ≤ 30 日、5 node type、storage 二系統分離 |
| 02 | [ネットワーク方針](02_ネットワーク方針.md) | Calico CNI + MetalLB BGP + kube-vip + Istio + Envoy Gateway + HTTP/3 + WebTransport |
| 03 | [ストレージ方針](03_ストレージ方針.md) | Longhorn block + Rook+Ceph object（storage 二系統分離）+ ClickHouse tiered |
| 04 | [セキュリティ方針](04_セキュリティ方針.md) | 5 段階防御（image / runtime / network / secret / policy）+ Kyverno admission |
| 05 | [GitOps 配信方針](05_GitOps配信方針.md) | Argo CD + Argo Rollouts + Argo Events + ApplicationSet generator |
| 06 | [オートスケール方針](06_オートスケール方針.md) | KEDA + Cluster Autoscaler、scaling 指標の階層 |
| 07 | [Chaos 工学方針](07_Chaos工学方針.md) | Litmus + 5 階層 chaos experiment + drill cadence |
| 08 | [IaC 方針](08_IaC方針.md) | OpenTofu + Kustomize + Helm + Tilt の 4 レイヤ |

## 上位フェーズへの依存
- [提供スコープ](../../02_要件定義/01_スコープ/01_提供スコープ.md): infra 軸の責務範囲
- [非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md): 委譲先
- [可用性災害対策](../../02_要件定義/03_非機能要件/02_可用性災害対策.md): infra + data + ops の cross 要件
- [OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md): infra 軸採用 OSS

## 下位フェーズへの委譲
- [クラスタ位相適合仕様](../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md): 5 topology_class + 5 failover orchestrator + drill cadence
- [時刻整合適合仕様](../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md): 5 clock_integrity_class（PTP / NTP / HLC / leap second 戦略）
- [infra 強制機構](../../04_詳細設計/02_強制機構/04_infra強制機構.md): 5 層 defense-in-depth + 25+ Kyverno admission policy
- [infra 運用 UI](../../04_詳細設計/04_運用UI開発者体験/01_infra運用UI.md)

## 横断軸との bind
- [data 設計方針](../06_data設計方針/README.md): preservation_class と topology_class の double bind
- [security 設計方針](../07_security設計方針/README.md): 段階 1〜5 の物理 enforcement
- [ops 設計方針](../08_ops設計方針/README.md): chaos drill / progressive delivery の orchestration

## 関連参照
- [親フォルダ index](../README.md)
- [規約層 index](../../00_format/README.md)
