---
id: plan.infra.scenario_node_lifecycle
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# node lifecycle（追加 / 故障対応）

## 一文方針

infra 担当者が k8s node の追加（pool 拡張）/ 故障対応（cordon + drain + replace）を GitOps 経由で実施し、5 topology_class の SLO と workload の継続稼働を維持する。

> 深夜 3 時、自宅 on-call の infra 担当者（シニア級）が Mattermost `#infra-alert` の push 通知で「control-plane-node-02 NotReady / etcd quorum 2/3」に気付く。手元にはラップトップの Perses dashboard と cluster_inventory.lock.yaml、Mattermost 越しに ops 担当者がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: node lifecycle 管理（drain / cordon / maintenance）を GitOps runbook で安全に実施する

## 現状業務での痛み

- node drain 手順が文書のみで管理され、緊急時に正しい手順で実行できるか不明確
- drain 中の pod eviction が正しく行われているかリアルタイムで確認できない
- maintenance 完了後の node 復帰確認が手動で、復帰漏れが SLO 影響を起こす

## k1s0 でこう変わる

- node lifecycle の runbook が Backstage TechDocs に定義され、手順が常に最新の状態に維持される
- Perses が drain 中の pod eviction 状況をリアルタイム表示し、問題を即時検知できる
- node 復帰後の health check が CI で自動実行され、復帰確認の抜け漏れがなくなる

## Trigger（発火条件）

- node pool の capacity が上限（85% 以上）に近づいた時
- node が NotReady 状態になった時
- KEDA による scale out が node 不足で失敗した時

## 想定頻度 / 典型きっかけ

想定頻度: 月次〜四半期（通常の capacity 追加）/ イベント駆動（node 故障時）

典型きっかけ: 「製造業 pack の FA 業務が増加し worker node の CPU 使用率が 90% を超えて KEDA の scale out が node 不足で失敗した。worker node を 3 台追加する必要が生じた」「control plane node の 1 台が hardware 故障で NotReady になり、etcd quorum が 2/3 になった」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）
- 承認: dual reviewer（infra 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 自宅 on-call / 本社 IT 室 | Mattermost `#infra-alert` / Perses dashboard | NotReady 検知 / cordon + drain / OpenTofu replace / etcd quorum 回復確認 |
| 関与（ops）| シニア | 自宅 on-call / 本社 IT 室 | Perses dashboard（SLO パネル）| node 故障中 SLO 監視 / workload 影響 alert |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | IaC PR レビュー / cluster_inventory.lock.yaml sign-off |

## 個人 KPI / 達成感

- node drain の mean time to complete が閾値以内であることを Perses で確認でき、運用効率の達成感を得られる
- drain 中の SLO 違反 0 件を達成でき、安全な node 管理の品質を定量確認できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（runbook 更新 2h + Perses 監視設定 2h + health check CI 設定 2h）
- 関与人数: 2〜3 名（infra 担当者・ops 担当者・dual reviewer）
- コスト感: 低。runbook 整備が主な作業で自動化後は手動作業が最小化される

## 前提

- `cluster_inventory.lock.yaml` に現在の node 構成が記録済みであること
- OpenTofu で node pool が宣言的に管理済みであること
- 全 5 topology_class のトポロジ状態が `topology_drill.lock.yaml` に記録済みであること

## 流れ

### [Node 追加フロー]

1. Perses dashboard で現在の node リソース使用率を確認し、追加が必要な node 数を見積もる
2. OpenTofu の IaC（`infra/k8s-cluster/node-pools.tf`）で `worker_node_count` を更新し PR を作成する
3. dual reviewer sign-off を取得し Argo CD 経由で適用する（kubectl 直接操作は禁止）
4. 新 node が `Ready` 状態になることを確認し、`cluster_inventory.lock.yaml` を更新する
5. KEDA の scale out が新 node で成功することを確認する
6. topology_class の 5 種全てが新 node 構成後も drill green であることを次回 drill cadence で確認する

### [Node 故障対応フロー]

1. NotReady node を確認し、故障種別を分類する（hardware 故障 / OS hang / kubelet crash）
2. 影響 workload の SLO が維持されているかを Perses で確認する
3. 故障 node を `kubectl cordon <node>` で新規 scheduling を停止する（GitOps 経由の操作を優先、緊急時のみ直接 kubectl）
4. `kubectl drain <node> --ignore-daemonsets --delete-emptydir-data` で既存 Pod を他 node に退避する
5. node の replace を OpenTofu で実施（IaC で `lifecycle = replace_triggered_by` を追加して apply）
6. 新 node が `Ready` になり etcd quorum が 3/3 に回復することを確認する
7. `cluster_inventory.lock.yaml` を更新し、dual reviewer sign-off を取得する
8. 故障原因を調査し、再発防止策を postmortem に記録する（**postmortem 期限: 2 営業日以内**）

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | node_lifecycle.lock.yaml で対象 node を確認し cordon を実行 | `@infra node cordon 実行 / 対象: node-X / T+0` |
| 10分 | infra 担当者 | kubectl drain で pod eviction を開始し Perses で進捗を確認 | `drain 開始 / eviction 進捗 N/M pods` |
| 30分 | infra 担当者 | 全 pod eviction 完了を確認し maintenance 作業を開始 | `drain 完了 / maintenance 開始` |
| 2h | infra 担当者 | maintenance 完了 → uncordon → health check green を確認し lock.yaml を更新 | `node 復帰 / health check green / lock.yaml 更新` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **ライン稼働監視（リアルタイム）**: control plane node 故障による etcd quorum 低下は全業務の API server を不安定にし、リアルタイム監視データのストリーミングが途切れるため、etcd quorum 回復を最優先 SLA として対応する。
- **警報配信**: node NotReady 中に警報配信 Pod が再スケジュールされるまでの間、製造ライン停止アラートが欠落するリスクがあるため、drain 前に警報配信 Pod の PDB を確認し最低 1 replica の稼働を保証する。

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- infra 強制機構: [../../../04_詳細設計/02_強制機構/04_infra強制機構.md](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)
- 関連 OSS: OpenTofu（IaC）/ Argo CD（GitOps）/ KEDA（autoscale）/ Perses（SLO monitoring）

## 期待結果 / 観測指標

[node 追加フロー]
- artifact: `cluster_inventory.lock.yaml` に新 node エントリが記録済み
- ci: KEDA scale out が新 node で成功（Perses `keda-scaling-success-rate` ≥ 99.9% / 計測 window: 24h）
- slo: node 追加後 24h の可用性 SLO ≥ 99.9%（Perses `cluster-availability` パネル）
- sign-off: dual reviewer（infra 担当者 2 名）sign-off 完了

[node 故障対応フロー]
- artifact: `cluster_inventory.lock.yaml` が更新済み（故障 node が削除・replace node が追加）
- state: etcd quorum が 3/3 に回復（`kubectl get po -n kube-system -l component=etcd` 全件 Running）
- slo: 故障対応後 24h の可用性 SLO ≥ 99.9%
- sign-off: dual reviewer（infra 担当者 2 名）sign-off 完了 + postmortem 提出（2 営業日以内）

## 失敗時の挙動 / escalation

- **etcd quorum が過半数割れ（control plane 2 台以上 NotReady）**: ops 担当者 + security 担当者に Mattermost `#infra-incident` で即時通報（**SLA: 5 分以内**）。Backstage runbook `etcd-quorum-recovery` を起動。1.0.0 ship blocker 認定。**postmortem 期限: 2 営業日以内**。
- **drain 中に PDB（PodDisruptionBudget）が drain を阻止**: PDB の設定を確認し、disruption 許容数を一時的に増やすか workload を手動でスケールダウンする。tier2 担当者に影響業務を Mattermost `#tier2-incident` で通知（**SLA: 1h 以内**）。
- **IaC apply 後に node が Ready にならない**: OpenTofu rollback を実施し ops 担当者 + infra 担当者で Mattermost `#infra-incident` に集合（**SLA: 30 分以内**）。

## 失敗パターン (anti-pattern)

- PDB 無視の強制 drain: --force フラグで PDB を無視した drain は SLO 違反を引き起こす
- drain 後の lock.yaml 未更新: 操作記録がないと次回の maintenance 時に前回状態が不明になる

## 関連参照

- [infra 担当者シナリオ index](./README.md) — infra 担当者シナリオ全体の構成と 5 topology_class 一覧
- [cluster upgrade](./01_cluster_upgrade.md) — node ライフサイクルと同時に k8s バージョン更新が必要な場合のシナリオ
- [topology drill / failover](./02_topology_drill_failover.md) — node 追加後の topology drill 実行シナリオ
- [infra 設計方針](../../../03_概要設計/05_infra設計方針/README.md) — クラスタ構成方針（node type 5 種類）の SoT
