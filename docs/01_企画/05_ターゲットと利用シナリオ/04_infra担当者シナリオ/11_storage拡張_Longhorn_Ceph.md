---
id: plan.infra.scenario_storage_expansion
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

# storage 拡張 Longhorn / Ceph

## 一文方針

infra 担当者が Longhorn block storage（PVC）の容量拡張と Rook+Ceph object storage の OSD 追加を GitOps 経由で実施し、storage 二系統分離原則と 5 preservation_class の replication 要件を維持する。

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Perses dashboard の `ceph-capacity` パネルを確認中に、Ceph object storage の使用率が 78% を超えたアラートが前夜に Mattermost `#infra-alert` に届いていることに気付く。手元には cluster_inventory.lock.yaml と OpenTofu の ceph_osd_count 設定ファイル、Mattermost 越しに data 担当者と dual reviewer がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: Longhorn / Ceph ボリューム拡張を自動化しストレージ運用の属人化を排除する

## 現状業務での痛み

- Longhorn / Ceph のボリューム拡張が手動操作で、拡張タイミングが担当者の主観判断に依存する
- ストレージ容量の閾値アラートがなく、容量不足が本番障害として発覚するまで気付かれない
- 拡張操作手順が文書管理で属人化し、担当者が変わると操作ミスが発生する

## k1s0 でこう変わる

- Perses のストレージ使用率が閾値（80%）に達した時点で自動 alert が発行され、事前に対処できる
- ボリューム拡張が GitOps manifest の更新で完結し、手動 kubectl 操作が不要になる
- storage.lock.yaml が拡張履歴を記録し、容量管理の追跡が可能になる

## Trigger（発火条件）

- storage 使用率が閾値（Longhorn: 80% / Ceph: 75%）を超えた時
- 新機能追加で storage 要件が増加した時

## 想定頻度 / 典型きっかけ

想定頻度: 四半期〜年次。典型きっかけ: 「製造業 pack FA の SCADA テレメトリ収集が増加し、Ceph object storage の使用率が 78% を超えた。OSD を 3 台追加する必要が生じた」「ClickHouse の tiered storage で block PVC が不足し、Longhorn の StorageClass で PVC を 500Gi → 1Ti に拡張する必要が生じた」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: data 担当者（storage 要件の確認）
- 承認: dual reviewer（infra 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Perses dashboard（ceph-capacity / longhorn-capacity パネル）| 使用率確認 / OSD 追加 IaC 更新 / Argo CD 適用 / rebalance 監視 |
| 関与（data）| シニア | 本社 IT 室 / リモート | CloudNativePG / Kafka dashboard | storage 要件確認 / rebalance 完了後の DB replication 確認 |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | IaC PR レビュー / cluster_inventory.lock.yaml sign-off |

## 個人 KPI / 達成感

- ストレージ容量不足インシデント 0 件を達成でき、予防的な容量管理の達成感を得られる
- 拡張操作の mean time to complete が短縮され、運用効率の改善を数値で確認できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（alert 設定 2h + GitOps 拡張手順確立 4h + lock.yaml 設定 1h）
- 関与人数: 2〜3 名（infra 担当者・ops 担当者・dual reviewer）
- コスト感: 低。alert と GitOps 自動化で継続運用コストが大幅削減される

## 前提

- [ストレージ方針](../../../03_概要設計/05_infra設計方針/03_ストレージ方針.md)（Longhorn block + Rook+Ceph object の 2 系統分離）が確立済み
- OpenTofu で storage 構成が宣言的に管理済み

## 流れ

### Longhorn block PVC 拡張フロー

1. Perses dashboard（`longhorn-capacity` パネル）で現在の PVC 使用率を確認する
2. 拡張が必要な PVC を特定し、StorageClass の `allowVolumeExpansion: true` を確認する
3. PersistentVolumeClaim の `spec.resources.requests.storage` を更新する IaC（Kustomize / Helm values）を修正して PR を作成する
4. Argo CD 経由で staging cluster に先行適用し、PVC 拡張が正常完了することを確認する
5. 本番 cluster に適用し、`cluster_inventory.lock.yaml` の storage 使用量を更新する
6. dual reviewer sign-off を取得する

### Rook+Ceph OSD 追加フロー

1. Perses dashboard（`ceph-capacity` パネル）で現在の OSD 使用率と CRUSH map を確認する
2. 追加する OSD のノードと disk を決定し、OpenTofu IaC で `ceph_osd_count` を更新する
3. PR を作成し dual reviewer sign-off 後に Argo CD で適用する
4. `ceph status` で OSD が `up` / `in` になり、data rebalance が完了することを確認する（rebalance 中は Perses で進捗を監視）
5. CRUSH map が意図通りに更新されていることを確認する（rack awareness が維持されていること）
6. `cluster_inventory.lock.yaml` の Ceph capacity を更新する
7. dual reviewer sign-off を取得する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | Perses でストレージ使用率 alert を設定し GitOps 拡張 manifest を準備 | `storage alert 設定完了 / 拡張 manifest 準備` |
| alert 受信 | infra 担当者 | 使用率 80% alert を受信し GitOps manifest の volume size を更新 | `storage alert 受信 / 拡張 manifest 更新 PR #NNN` |
| PR merge 後 | infra 担当者 | Argo CD が volume 拡張を apply し拡張完了を Perses で確認 | `volume 拡張完了 / 使用率 65% / storage.lock.yaml 更新` |
| 1d | dual reviewer | 拡張設定と lock.yaml を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **SCADA テレメトリ（大容量）**: センサーテレメトリのオブジェクト蓄積は Ceph object storage の最大消費者であり、OSD 追加後の CRUSH map rack awareness が維持されていることを `ceph status` で確認してから SCADA namespace を再起動する。
- **品質検査**: 品質検査ログは Longhorn block PVC に永続化されており、PVC 拡張中の data integrity が保たれていることを integration test で確認してから本番の品質検査業務に拡張後 PVC を適用する。

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- infra 強制機構: [../../../04_詳細設計/02_強制機構/04_infra強制機構.md](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)
- 関連 OSS: Longhorn（block storage）/ Rook+Ceph（object storage）/ OpenTofu（IaC）/ Argo CD（GitOps）/ Perses（monitoring）

## 期待結果 / 観測指標

- artifact: `cluster_inventory.lock.yaml` に更新後 storage 容量が記録済み
- state: Longhorn PVC が拡張済み（`kubectl get pvc -A | grep <name>` で新 capacity を確認）
- state: Ceph OSD が all `up/in`（`ceph osd stat` で確認）、rebalance 完了（`ceph -s` で `HEALTH_OK`）
- slo: storage 使用率が閾値以下に低下（Perses `longhorn-capacity` / `ceph-capacity` で確認）
- sign-off: dual reviewer（infra 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **Ceph OSD 追加後に rebalance が 24h 経過しても完了しない**: data 担当者に Mattermost `#data-incident` で報告（**SLA: 24h 以内**）。CRUSH map の設定ミスを確認し修正。
- **storage 二系統分離違反（block / object が同一ディスクを共有）**: 即時修正。ops 担当者に Mattermost `#infra-incident` で報告（**SLA: 1h 以内**）。**postmortem 期限: 2 営業日以内**。

## 失敗パターン (anti-pattern)

- 容量不足後の事後対応: alert なしで満杯になってから拡張すると pod が eviction され SLO 違反になる
- 手動 kubectl の直接拡張: GitOps を迂回した拡張は drift detection が検知する

## 関連参照

- [infra 担当者シナリオ index](./README.md) — infra 担当者シナリオ全体の構成
- [node lifecycle](./09_node_lifecycle.md) — OSD 追加には新 node が必要な場合のシナリオ
- [infra 設計方針 ストレージ方針](../../../03_概要設計/05_infra設計方針/03_ストレージ方針.md) — Longhorn / Ceph 2 系統分離の設計指針
- [ClickHouse tiered storage 運用（data-12）](../05_data担当者シナリオ/12_ClickHouse_tiered_運用.md) — hot tier（Longhorn）容量拡張後、ClickHouse の warm tier policy を data 担当者が調整する
