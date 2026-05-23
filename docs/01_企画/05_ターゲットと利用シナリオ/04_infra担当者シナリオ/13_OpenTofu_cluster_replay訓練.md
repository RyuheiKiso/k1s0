---
id: plan.infra.scenario_opentofu_replay_drill
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, E]
  proof_classes: []
---

# OpenTofu cluster replay 訓練

## 一文方針

infra 担当者が OpenTofu を使って cluster 全体を隔離環境で replay（再構築）し、「不可逆性の禁止: cluster 廃棄は OpenTofu replay で再構築可能」の原則を年次で物理証明する。

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Backstage のカレンダーで「年次 OpenTofu replay 訓練 / 本日開始」を確認し、隔離環境のセットアップを開始する。手元には本番の opentofu_replay.lock.yaml と terraform.tfstate バックアップ、Mattermost 越しに ops 担当者・data 担当者・dual reviewer がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: OpenTofu IaC のドリフト検知と replay 訓練で infrastructure-as-code の信頼性を維持する

## 現状業務での痛み

- IaC 変更のドリフト検知がなく、手動操作による本番環境の変更が IaC と乖離しても気付かれない
- IaC の replay（destroy + apply）を実際に試していないため、災害復旧時に手順の欠陥が初めて発覚する
- ドリフトの原因追跡が手動で、何がいつ変更されたかの特定に時間がかかる

## k1s0 でこう変わる

- OpenTofu drift detection が CI で定期実行され、IaC と本番環境の乖離が自動検知される
- replay 訓練が定期 cadence で必須化され、disaster recovery 時の手順信頼性が継続的に確認される
- opentofu.lock.yaml が replay 訓練の結果を記録し、IaC 信頼性の推移が可視化される

## Trigger（発火条件）

- 年次 replay 訓練の cadence 到来時（infra 設計方針「段階的 release 禁止: replay 可能であること」が原則）

## 想定頻度 / 典型きっかけ

想定頻度: 年次。典型きっかけ: 「infra 設計方針の「不可逆性の禁止」原則を検証する年次 drill として、本番環境の OpenTofu state から隔離 cluster を完全再構築し、全 topology_class drill が green であることを確認した」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（replay 中の SLO 監視）/ data 担当者（data layer の replay 確認）
- 承認: dual reviewer（infra 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Backstage カレンダー / opentofu_replay.lock.yaml | state コピー / terraform plan・apply / topology drill 実施 / Kyverno・GitOps 確認 |
| 関与（ops）| シニア | 本社 IT 室 / リモート | Perses dashboard | 隔離 cluster での SLO 監視 / replay 結果の観測 |
| 関与（data）| シニア | 本社 IT 室 / リモート | CloudNativePG / Kafka dashboard | data layer（DB / Kafka）の replay 後正常稼働確認 / replication lag 確認 |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | replay 結果 PR レビュー / opentofu_replay.lock.yaml sign-off |

## 個人 KPI / 達成感

- IaC ドリフト 0 件が CI で定量確認でき、infrastructure 管理精度の向上を実感できる
- replay 訓練 green cadence の達成率を lock.yaml で確認できるようになり、DR 準備の達成感を得られる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 日（drift detection CI 設定 4h + replay 訓練手順確立 4h + lock.yaml 設定 2h）
- 関与人数: 2〜3 名（infra 担当者・ops 担当者・dual reviewer）
- コスト感: 低〜中。CI 自動化後は継続コストが定期訓練の実施のみになる

## 前提

- 本番 cluster の OpenTofu state（`terraform.tfstate`）が GitOps で最新に保たれている
- `cluster_inventory.lock.yaml` に全 node / OSS バージョンが記録済み
- 隔離環境（separate cloud account or separate cluster）が用意済み

## 流れ

1. 本番 OpenTofu state を隔離環境にコピーする（`terraform state pull > isolated_state.json`）
2. 隔離環境で `terraform plan` を実行し、本番 state との差分が意図した範囲内であることを確認する
3. 隔離環境で `terraform apply` を実行してクラスタを再構築する
   - k8s cluster（control plane + worker node）
   - 全 platform OSS（Argo CD / Backstage / Kyverno / OpenBao / Litmus 等）
   - storage（Longhorn / Rook+Ceph）
4. 再構築した隔離 cluster で 5 topology_class の failover drill を実施する（[topology_drill_failover シナリオ](./02_topology_drill_failover.md)参照）
5. 隔離 cluster で全 Kyverno policy が適用されていることを確認する（`kubectl get clusterpolicies -A`）
6. 隔離 cluster で GitOps sync が動作することを確認する（Argo CD が manifests を apply できること）
7. replay 結果を `opentofu_replay.lock.yaml` に記録する（再構築時間 / 全 drill 結果 / 差分が 0 件であることの証拠）
8. dual reviewer sign-off を取得する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | OpenTofu drift detection を CI に設定し初回 drift scan を実行 | `drift detection 設定完了 / 初回 scan: drift N 件` |
| 4h | infra 担当者 | 検出された drift を IaC に反映し drift 0 件を確認 | `drift 解消完了 / IaC 同期確認` |
| 1d | infra 担当者 | replay 訓練を staging で実施し opentofu.lock.yaml に結果を記録 | `replay 訓練 green / lock.yaml 更新` |
| 1d+2h | dual reviewer | drift detection 設定と replay 結果を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **FA（設備操作）**: FA 業務の操作指示システムは cluster の復元性に最も依存しており、replay 訓練で FA namespace の Pod 群が `terraform apply` 後に仕様通りに起動することを topology drill で検証して DR 対応能力を証明する。
- **受注（最重要業務）**: 受注業務は事業継続の根幹であるため、replay 後の隔離 cluster で受注 API が正常応答し Kyverno policy が全適用されていることを確認し、cluster 廃棄シナリオでも受注業務を復旧できることを年次で物理証明する。

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- infra 強制機構: [../../../04_詳細設計/02_強制機構/04_infra強制機構.md](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)
- 関連 OSS: OpenTofu（IaC / cluster replay）/ Argo CD（GitOps validation）/ Litmus（chaos drill）

## 期待結果 / 観測指標

- artifact: `opentofu_replay.lock.yaml` に年次 replay green エントリが記録済み
- state: 隔離 cluster で 5 topology_class drill が全件 green
- diff: `terraform plan` の差分が 0 件（または意図した差分のみ）
- sign-off: dual reviewer（infra 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **`terraform apply` が途中で fail（resource 作成失敗）**: fail した resource と原因を記録し、IaC を修正してから再実行（**SLA: replay 期間内 = 1 週間以内**）。fail を放置すると 1.0.0 ship blocker。**postmortem 期限: 3 営業日以内**。
- **topology_class drill が fail**: infra 設計方針の「段階的 release 禁止」に従い、drill が green になるまで本番 cluster の 1.0.0 ship を保留。

## 失敗パターン (anti-pattern)

- 手動 terraform apply の本番直接実行: IaC を迂回した変更は drift detection が即時検知する
- replay 訓練の省略: 訓練なしの IaC は disaster recovery 時に手順の欠陥が本番で発覚する

## 関連参照

- [infra 担当者シナリオ index](./README.md) — infra 担当者シナリオ全体の構成
- [topology drill / failover](./02_topology_drill_failover.md) — replay 後に実施する topology drill シナリオ
- [infra 設計方針 IaC 方針](../../../03_概要設計/05_infra設計方針/08_IaC方針.md) — OpenTofu / cluster replay の設計指針
