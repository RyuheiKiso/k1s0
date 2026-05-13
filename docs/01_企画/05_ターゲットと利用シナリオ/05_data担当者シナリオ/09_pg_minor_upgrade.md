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

## ペルソナ要約

主役: data 担当者（シニア級）、目的: PostgreSQL マイナーアップグレードを CloudNativePG rolling upgrade で無停止実施する

## 現状業務での痛み

- PostgreSQL マイナーアップグレードで手動ダウンタイムが発生し、業務影響が出るたびに事前調整が必要
- アップグレード手順が文書管理で属人化し、担当者ごとに手順の解釈に差が生じる
- アップグレード後の動作確認が手動で、問題の発見が遅れる

## k1s0 でこう変わる

- CloudNativePG の rolling upgrade が自動実行され、マイナーアップグレードが無停止で完了する
- pg_upgrade.lock.yaml が upgrade 手順と結果を管理し、誰が実施しても同一品質が保証される
- upgrade 後の自動整合性チェックが CI に組み込まれ、問題が即時検知される

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

## 個人 KPI / 達成感

- マイナーアップグレードのダウンタイム 0 分を達成でき、無停止運用の達成感を得られる
- upgrade 後の SLO 維持率を Perses で定量確認でき、upgrade 品質の向上を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 半日（upgrade 計画確認 1h + rolling upgrade 実行 2h + 動作確認 1h）
- 関与人数: 2〜3 名（data 担当者・ops 担当者・dual reviewer）
- コスト感: 低。CloudNativePG が rolling upgrade を自動管理するため手動作業が最小化される

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | data 担当者 | pg_upgrade.lock.yaml で upgrade 対象 version を確認し rolling upgrade を開始 | `PostgreSQL rolling upgrade 開始 / 対象 version 確認` |
| 15分 | data 担当者 | CloudNativePG が各 pod を順次 upgrade し SLO 維持を Perses で確認 | `rolling upgrade 進行中 / SLO 維持確認` |
| 30分 | data 担当者 | 全 pod upgrade 完了を確認し整合性チェックを実行 | `upgrade 完了 / 整合性チェック green` |
| 1d | dual reviewer | upgrade 結果と lock.yaml を確認し sign-off | `sign-off 完了` |

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

## 失敗パターン (anti-pattern)

- 手動停止 upgrade: CloudNativePG rolling upgrade を使わず手動でクラスタを停止するとダウンタイムが発生する
- upgrade 後確認省略: 整合性チェックなしで upgrade を完了とすると潜在問題が後から発覚する

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [schema migration](./01_schema_migration.md) — upgrade に伴う schema 変更が必要な場合のシナリオ
- [restore drill](./03_restore_drill.md) — upgrade 後の restore drill 実施シナリオ
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — 5 preservation_class の正典定義と restore_window SLO
