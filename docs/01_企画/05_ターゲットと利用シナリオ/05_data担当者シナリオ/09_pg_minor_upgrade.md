---
id: plan.data.scenario_pg_minor_upgrade
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, E]
  proof_classes: []
---

# PostgreSQL minor version upgrade（CloudNativePG）

## 一文方針

data 担当者が CloudNativePG 上の PostgreSQL minor version upgrade（例: 15.x → 16.x）を実施し、5 preservation_class の replication topology と data 整合性を維持する。

> 朝 9 時、本社 IT 室の data 担当者（シニア級）が Perses の CloudNativePG dashboard を確認し、infra 担当者から届いていた「CVSS 7.5 CVE の security patch（PostgreSQL 15.6 → 15.8）適用依頼」Mattermost メッセージに気付く。手元には CloudNativePG cluster manifest と `cluster_inventory.lock.yaml`、Mattermost 越しに infra 担当者・tier2 担当者・dual reviewer がいる。

## Trigger（発火条件）

PostgreSQL minor version の EOL 到来 / CloudNativePG が新 PostgreSQL minor version のサポートを開始した時 / security patch が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 年次〜不定期（PostgreSQL の EOL サイクルに追従）。典型きっかけ: 「PostgreSQL 15 の security patch（15.6 → 15.8）が公開され、CVSS 7.5 の CVE を修正する upgrade が必要になった」「CloudNativePG operator が PostgreSQL 16 の L1+ サポートを宣言し、15 → 16 の major version upgrade 計画を作成することになった」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: infra 担当者（Kubernetes 側の CloudNativePG operator 更新）
- 関与: tier2 担当者（upgrade 後の migration 動作確認）
- 承認: dual reviewer（data 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | Perses / CloudNativePG dashboard | staging upgrade 実施・動作確認・本番 apply・lock.yaml 更新 |
| 関与（infra）| シニア | 本社 IT 室 / リモート | Argo CD / Kyverno | CloudNativePG operator 更新・k8s 側設定変更 |
| 関与（tier2）| ミドル〜シニア | 本社 / リモート | Backstage TechDocs | upgrade 後の migration 動作確認・integration test 実行 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | 変更 PR sign-off（data 担当者 2 名、author 不可） |

## 前提

- CloudNativePG Cluster リソースが `cluster_manifest/cnpg-cluster-<name>.yaml` として GitOps 管理済み
- `preservation_class.lock.yaml` に現行 DB cluster の class が記録済み
- 既存 migration が全て forward-only で適用済み（`schema_migration.lock.yaml` で確認）
- 進行中の schema_migration（シナリオ 01）がないこと。expand-contract pattern の中途で pg upgrade を開始すると schema 整合が崩れる恐れがある。

## 流れ

1. CloudNativePG operator が upgrade 対象 PostgreSQL version をサポートしているか確認する（CloudNativePG compatibility matrix）
2. staging cluster の CloudNativePG Cluster manifest で `imageName: postgres:<new-version>` を更新し PR を作成する
3. staging での upgrade: CloudNativePG が rolling upgrade を自動実施する（primary → replica の順に順次再起動）
   - upgrade 中の replication lag が SLI 閾値（例: 30 秒）以内に収まることを Perses で監視
   - primary の failover が不要なことを確認（rolling upgrade では primary は最後に再起動）
4. staging での動作確認:
   - 全 migration が upgrade 後も正常適用されることを確認（`sqlx migrate status` で all applied）
   - tier2 integration test（Testcontainers）を staging DB に向けて実行し、全 test green を確認
   - [保全方針](../../../03_概要設計/06_data設計方針/03_保全方針.md) の restore_drill を実施（upgrade 後の backup からの restore が可能か確認）
5. 本番 cluster への適用: CloudNativePG Cluster manifest を GitOps 経由で更新する
   - `v1_zone_replicated` 以上の class は upgrade 中も replicas が利用可能な状態を維持する
   - upgrade 完了を `cluster_inventory.lock.yaml` と `preservation_class.lock.yaml` に記録する
6. dual reviewer sign-off を取得する

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **受注管理**: PostgreSQL upgrade 中の replication lag SLI 逸脱は受注処理の read 可用性に直接影響するため、rolling upgrade の各フェーズで lag 監視が最重要となる。
- **SCADA 連携**: SCADA テレメトリの PostgreSQL 書込経路が upgrade 中も維持されることで、製造ライン状態の欠損なき記録が保証される。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- data 強制機構: [../../../04_詳細設計/02_強制機構/05_data強制機構.md](../../../04_詳細設計/02_強制機構/05_data強制機構.md)
- 関連 OSS: CloudNativePG（PostgreSQL Kubernetes operator）/ Barman Cloud（backup）/ Perses（replication lag 監視）

## 期待結果 / 観測指標

- artifact: `cluster_inventory.lock.yaml` の `cnpg_version` と `pg_version` フィールドが新バージョンに更新済み
- artifact: `preservation_class.lock.yaml` の replication topology が upgrade 後も正確に記録済み
- ci: tier2 integration test（Testcontainers）が upgrade 後の staging DB で all green
- drill: restore_drill が upgrade 後に green（`restore_drill.lock.yaml` に新エントリ追加済み）
- slo: replication lag SLI が upgrade 中 ≤ 30 秒を維持（Perses `pg-replication-lag` パネル）
- sign-off: dual reviewer（data 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **replication lag が upgrade 中に SLI 閾値を超過**: 超過が 1 分以上継続する場合は upgrade を中断し CloudNativePG の rollback（旧 version に戻す）を実施。ops 担当者に Mattermost `#data-incident` で通報（**SLA: 5 分以内**）。**postmortem 期限: 3 営業日以内**。
- **upgrade 後に migration が fail する**: tier2 担当者に Mattermost `#data-incident` で即時連絡（**SLA: 15 分以内**）。本番 upgrade を中止し staging で原因調査。migration の forward-only 原則により rollback は不可のため、修正 migration を作成（**SLA: 48h 以内**に修正 PR）。
- **restore_drill が upgrade 後に fail**: 1.0.0 ship blocker 認定。ops 担当者に Mattermost `#data-drill-fail` で通報（**SLA: 30 分以内**）。Backstage ticket `pg-upgrade-drill-fail-<version>` を起票し、次 drill までに CloudNativePG backup 設定を修正（**SLA: 5 営業日以内**）。

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [schema migration](./01_schema_migration.md) — upgrade に伴う schema 変更が必要な場合のシナリオ
- [restore drill](./03_restore_drill.md) — upgrade 後の restore drill 実施シナリオ
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — 5 preservation_class の正典定義と restore_window SLO
