---
id: plan.data.scenario_clickhouse_tiered_storage
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C]
  proof_classes: []
---

# ClickHouse tiered storage 運用

## 一文方針

data 担当者が ClickHouse Operator 管理下の ClickHouse クラスタで tiered storage の hot/warm policy 変更・MergeTree の TTL / storage policy 設定変更・圧縮戦略（zstd / lz4）調整を実施し、クエリ性能と容量コストのバランスを維持する。

> 朝 9 時、本社 IT 室の data 担当者（シニア級）が Perses の `clickhouse-hot-tier-usage` ダッシュボードを開き、hot tier が 82% に達している容量アラートに気付く。手元には ClickHouse Operator の DDL ファイルと `clickhouse_storage.lock.yaml`、Mattermost 越しに ops 担当者・tier2 担当者・dual reviewer がいる。

## ペルソナ要約

主役: data 担当者（シニア級）、目的: ClickHouse tiered storage を正しく設定し hot / warm / cold の自動 tier 移動を機能させる

## 現状業務での痛み

- ClickHouse tiered storage の設定誤りでホットデータが cold tier に早期移動し、クエリ遅延が発生する
- tiered storage の設定が属人的で、担当者が変わると設定の意図が伝わらない
- tier 移動の状況が不可視で、設定ミスの発見が遅れる

## k1s0 でこう変わる

- tiered storage ポリシーが clickhouse.lock.yaml で管理され、tier 移動条件が明示的に定義される
- Perses の tier 使用率メトリクスが tier 移動状況をリアルタイム表示し、設定ミスを即時検知する
- tier 移動ポリシーの変更が GitOps PR で審査され、設定ミスが deploy 前に防止される

## Trigger（発火条件）

ClickHouse の hot tier 容量が閾値（例: 80%）を超えた時、または analytics クエリ性能の劣化が観測された時、または新規データソース（tier2 Projector 追加）に対応する storage policy の追加が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（hot tier 容量アラートまたは四半期のクエリ性能レビュー時）。典型きっかけ: 「製造 SCADA テレメトリの蓄積により hot tier が 82% に達し、90 日以上のデータを warm tier（Rook+Ceph）に移動する TTL policy 変更が必要になった」「tier2 担当者が追加した Read model Projector により新規 ClickHouse テーブルが作成され、適切な storage policy と compression codec を設定する必要が生じた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: ops 担当者（ClickHouse クエリ性能・容量のアラート確認）
- 関与: tier2 担当者（Read model / Projector の追加に伴う schema 要件の確認）
- 承認: dual reviewer（data 担当者 2 名、PR author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | Perses（`clickhouse-hot-tier-usage`）/ `clickhouse_storage.lock.yaml` | TTL / storage policy / codec 変更 DDL 作成・GitOps apply・lock.yaml 更新 |
| 関与（ops）| ミドル〜シニア | 本社 / リモート | Perses（ClickHouse 容量・クエリ性能）/ Mattermost `#ops` | アラート確認・容量監視 |
| 関与（tier2）| ミドル〜シニア | 本社 / リモート | Backstage TechDocs | Read model / Projector 追加に伴う schema 要件確認 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | 変更 PR sign-off（data 担当者 2 名、author 不可） |

## 個人 KPI / 達成感

- hot tier クエリレイテンシ SLO の達成率を Perses で定量確認でき、tiered storage 最適化の達成感を得られる
- tier 設定変更 PR の品質向上を数値で確認できる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 日（tier ポリシー設計 4h + 設定適用 2h + tier 移動確認 2h）
- 関与人数: 2〜3 名（data 担当者・infra 担当者・dual reviewer）
- コスト感: 低〜中。lock.yaml と Perses 設定後は継続コストが監視のみになる

## 前提

- ClickHouse Operator（Altinity または clickhouse-operator）が cluster に deploy 済み
- hot tier: local NVMe (Longhorn) / warm tier: Rook+Ceph Object Store が設定済み
- `clickhouse_storage.lock.yaml` に現行 storage policy・TTL 設定が記録済み
- 変更対象テーブルの DDL が GitOps 管理済み（`data/clickhouse/ddl/` 配下）
- **TTL 変更は非破壊的操作**: TTL を短くすると即座にデータ移動が始まるため、hot tier の容量に余裕があるタイミングで実施する

## 流れ

1. 変更目的を確認する（hot 容量逼迫 / クエリ性能改善 / 新規テーブルへの policy 適用 / 圧縮戦略変更）
2. **hot 容量逼迫 / TTL 変更の場合**:
   - 対象テーブルの現行 TTL と data distribution を確認する（`SELECT toStartOfDay(event_time), count(), sum(bytes) FROM system.parts WHERE table = '<table>'`）
   - 新 TTL（例: `TTL event_time + INTERVAL 90 DAY TO DISK 'warm'`）を DDL ファイルに適用し PR を提出する
   - staging での DDL apply 後、`system.parts` の `disk_name` が warm tier に移動することを確認する
3. **圧縮戦略変更の場合**:
   - `CODEC(ZSTD(3))` または `CODEC(LZ4)` を列定義に追加 / 変更した DDL を作成する（新規挿入データから適用。既存データは `OPTIMIZE TABLE <table> FINAL` で再圧縮）
   - staging での圧縮率とクエリ性能（クエリ時間 / bytes_read）を Perses で比較確認する
4. **新規テーブルへの storage policy 適用の場合**:
   - `storage_policy = 'hot_to_warm'`（または新規 policy 定義）を `ENGINE = MergeTree() SETTINGS storage_policy='...'` に設定した DDL を作成する
   - Apicurio Registry の Read model schema と DDL の整合を tier2 担当者と確認する
5. GitOps 経由で ClickHouse Operator に apply する
   - `clickhouse-operator` の `ClickHouseInstallation` CRD が `Completed` になることを確認する
6. 本番 apply 後:
   - hot tier 使用率が閾値以下に回帰していることを Perses で確認（24h 観察）
   - 主要 analytics クエリ（`#clickhouse-perf-regression` CI job）のレスポンスタイムが SLO 内であることを確認する
7. `clickhouse_storage.lock.yaml` を更新（変更テーブル名 / policy / TTL / codec）し、dual reviewer sign-off を取得する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | data 担当者 | ClickHouse の tier 使用率を Perses で確認し tier ポリシーを設計 | `tier 使用率確認 / ポリシー設計完了` |
| 4h | data 担当者 | tiered storage ポリシーを ClickHouse config に適用し clickhouse.lock.yaml に記録 | `ポリシー適用完了 / lock.yaml 更新` |
| 1d | data 担当者 | tier 移動の動作を Perses で確認し PR 提出 | `tier 移動確認 green / PR #NNN 提出` |
| 1d+2h | dual reviewer | lock.yaml とポリシー設定を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **ライン稼働監視（analytics）**: ClickHouse の hot tier 容量逼迫がアラートの集計クエリに直結し、TTL policy 変更による warm 移動が監視ダッシュボードの SLO 維持に必要となる。
- **SCADA 連携（集計）**: SCADA テレメトリの長期蓄積データが hot tier を圧迫するケースが最多であり、tiered storage の最適化が SCADA 集計クエリの性能維持に直接寄与する。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- data 強制機構: [../../../04_詳細設計/02_強制機構/05_data強制機構.md](../../../04_詳細設計/02_強制機構/05_data強制機構.md)
- 関連 OSS: ClickHouse（Altinity Operator）/ Rook+Ceph（warm tier object store）/ Longhorn（hot tier block）/ Perses（容量・性能監視）

## 期待結果 / 観測指標

- artifact: `clickhouse_storage.lock.yaml` の対象テーブルエントリが新 TTL / policy / codec に更新済み
- ci: `#clickhouse-perf-regression` CI job が all green（主要 analytics クエリのレスポンスタイム SLO 達成）
- slo: hot tier 使用率が変更後 24h で目標閾値（80%）以下（Perses `clickhouse-hot-tier-usage` パネル）
- runtime: ClickHouse Operator `ClickHouseInstallation` CRD が `Completed` 状態
- sign-off: dual reviewer（data 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **TTL 変更後に hot tier の使用率が増加する（warm への移動が起きない）**: `OPTIMIZE TABLE <table>` を手動実行して TTL mutation をトリガーする。改善しない場合は ClickHouse Operator の設定と disk mapping を infra 担当者と共同確認（**SLA: 2h 以内**）。
- **圧縮変更後にクエリ性能が劣化する**: 圧縮 codec を元に戻す DDL を apply し、性能要件に合った codec を再評価する。tier2 担当者に Mattermost `#data-incident` で通報（**SLA: 1h 以内**）。
- **`OPTIMIZE TABLE FINAL` 中に hot tier が満杯になる**: 再圧縮を中断（`KILL QUERY WHERE query LIKE '%OPTIMIZE%'`）し、TTL による warm 移動を先行させてから再試行する。ops 担当者に容量アラートのサイレンスを依頼（**SLA: 30 分以内**）。
- **staging CI job fail**: tier2 担当者と共同で Read model schema の整合を確認し、DDL を修正する（**SLA: 48h 以内**）。

## 失敗パターン (anti-pattern)

- デフォルト設定のまま運用: workload に合わせた tier ポリシー調整なしでは hot データが早期 cold 移動する
- GitOps 外の直接設定変更: ClickHouse config を直接編集すると lock.yaml と乖離し drift detection が検知する

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [storage 拡張（infra-11）](../04_infra担当者シナリオ/11_storage拡張_Longhorn_Ceph.md) — warm tier の Rook+Ceph OSD 容量拡張と hot tier の Longhorn ボリューム拡張手順
- [archive_to_offline 復元](./13_archive_to_offline復元.md) — warm → archive_to_offline → online 復元フロー
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — tiered storage の preservation_class 設計根拠
