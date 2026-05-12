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

## Trigger（発火条件）

- 年次 replay 訓練の cadence 到来時（infra 設計方針「段階的 release 禁止: replay 可能であること」が原則）

## 想定頻度 / 典型きっかけ

想定頻度: 年次。典型きっかけ: 「infra 設計方針の「不可逆性の禁止」原則を検証する年次 drill として、本番環境の OpenTofu state から隔離 cluster を完全再構築し、全 topology_class drill が green であることを確認した」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（replay 中の SLO 監視）/ data 担当者（data layer の replay 確認）
- 承認: dual reviewer（infra 担当者 2 名、変更 PR の author 不可）

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

## 関連参照

- [infra 担当者シナリオ index](./README.md) — infra 担当者シナリオ全体の構成
- [topology drill / failover](./02_topology_drill_failover.md) — replay 後に実施する topology drill シナリオ
- [infra 設計方針 IaC 方針](../../../03_概要設計/05_infra設計方針/08_IaC方針.md) — OpenTofu / cluster replay の設計指針
