---
id: arch.infra.chaos_engineering_policy
axis: infra
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.infra.infra_index
covered_by:
  defense_in_depth_layers: [C, D]
  proof_classes: []
---

# infra Chaos 工学方針

## 一文方針
- 全 chaos 実演は Litmus で実装、cadence は 13 軸 + meta の各 spec の drill cadence に従う。chaos test green が 1.0.0 ship blocker。

## 至高路線における立ち位置
- Chaos Mesh 採用しない（Litmus L1+ 単一深耕）。v2 候補
- hand-crafted shell script chaos 禁止
- production cluster での無計画 chaos 禁止、scope と blast radius を必ず宣言

## Litmus 採用範囲
- L1+ primary: Litmus
- 用途:
    - [クラスタ位相適合仕様](../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md) の zone_partition / cluster_partition / region_partition_passive / region_partition_active scenario
    - [データ保全適合仕様](../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) の restore_drill
    - [SLO 適合仕様](../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md) の chaos_injection_validates_alert
    - [テナント容量適合仕様](../../04_詳細設計/01_適合仕様/09_テナント容量適合仕様.md) の noisy_neighbor_isolation property
    - [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) の pair_target 移行 rehearsal
    - [鍵管理適合仕様](../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md) の key rotation drill
    - [認証適合仕様](../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) の AuthContext propagation drill

## chaos experiment 階層

| 階層 | 内容 |
|---|---|
| 階層 1: Pod 単位 | pod-delete / pod-cpu-hog / pod-memory-hog / pod-network-loss / pod-network-latency / pod-io-stress |
| 階層 2: node 単位 | node-cpu-hog / node-memory-hog / node-drain / node-taint / node-restart |
| 階層 3: zone 単位 | zone-network-partition（network policy で zone 間通信遮断）|
| 階層 4: cluster 単位 | cluster-network-partition / cluster-control-plane-outage |
| 階層 5: region 単位 | region-network-partition / region-dns-failure |

## scenario の declarative 化
- 各 chaos experiment は ChaosEngine + ChaosExperiment manifest として codify、Argo CD で配信
- scenario の input は yaml で declare、build artifact が Litmus manifest を生成
- scenario 実行結果は `failover_drill.lock.yaml` / `restore_drill.lock.yaml` に書き戻し（build artifact）

## cadence

| topology_class | cadence (day) |
|---|---|
| `v1_single_zone` | 180 |
| `v1_multi_zone_per_cluster` | 90 |
| `v1_multi_cluster_active_active` | 60 |
| `v1_multi_region_active_passive` | 30 |
| `v1_multi_region_active_active` | 14 |
| data preservation_class | preservation_class.restore_drill_cadence_days に従う |

- cadence 超過は Kyverno admission policy で「新規 deploy block」を発動

## chaos game day
- 月次 game day: 制御された環境で人間オペレータが chaos シナリオを実演、runbook の習熟度を測定
- blast radius: staging cluster を default、production cluster での実演は v1_multi_region_active_active 以上で quarterly のみ

## assertion / 観測
- 各 scenario は 13 SLO の recording rule を assertion 対象
- assertion 失敗 = 13 error budget 消費 + drill red 記録
- assertion success = drill green 記録、`failover_drill.lock.yaml` に last_green_at を更新

## chaos と progressive delivery の連動
- Argo Rollouts AnalysisTemplate に chaos experiment を組込み、canary 段階で chaos injection、resilience 検証後に promote
- SLO 違反検出時は Argo Rollouts が automatic rollback

## rollback / safety
- 全 chaos experiment は scope を namespace / pod selector で限定、cluster-wide blast radius は明示的 game day 以外禁止
- chaos experiment timeout: default 10 分、超過で auto-stop
- emergency stop: Argo Events 経由で全 ChaosEngine を suspend する panic button

## chaos audit
- 全 experiment 実行は `infra.chaos.experiment.event` として 09 観測可能性 SoR に bulk insert
- cadence 超過 / drill red も同 audit signal で記録

## 強制機構との bind
- 本方針は [infra 強制機構](../../04_詳細設計/02_強制機構/04_infra強制機構.md) の以下経路で物理 enforce される:
    - 層 C: Litmus chaos test の自動 replay
    - 層 D: Kyverno `block-on-failover-drill-red` admission policy

## 採用しない選択肢
- Chaos Mesh: Litmus L1+ 単一深耕、v2 候補
- hand-crafted shell script chaos: 禁止
- production cluster での無計画 chaos: 禁止

## 関連参照
- [infra 設計方針 index](README.md)
- [オートスケール方針](06_オートスケール方針.md)
- [test 故障注入検証方針](../10_test設計方針/05_故障注入検証方針.md)
- [クラスタ位相適合仕様](../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
