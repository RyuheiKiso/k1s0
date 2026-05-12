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

## Trigger（発火条件）

ClickHouse の hot tier 容量が閾値（例: 80%）を超えた時、または analytics クエリ性能の劣化が観測された時、または新規データソース（tier2 Projector 追加）に対応する storage policy の追加が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（hot tier 容量アラートまたは四半期のクエリ性能レビュー時）。典型きっかけ: 「製造 SCADA テレメトリの蓄積により hot tier が 82% に達し、90 日以上のデータを warm tier（Rook+Ceph）に移動する TTL policy 変更が必要になった」「tier2 担当者が追加した Read model Projector により新規 ClickHouse テーブルが作成され、適切な storage policy と compression codec を設定する必要が生じた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: ops 担当者（ClickHouse クエリ性能・容量のアラート確認）
- 関与: tier2 担当者（Read model / Projector の追加に伴う schema 要件の確認）
- 承認: dual reviewer（data 担当者 2 名、PR author 不可）

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

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [storage 拡張（infra-11）](../04_infra担当者シナリオ/11_storage拡張_Longhorn_Ceph.md) — warm tier の Rook+Ceph OSD 容量拡張と hot tier の Longhorn ボリューム拡張手順
- [archive_to_offline 復元](./13_archive_to_offline復元.md) — warm → archive_to_offline → online 復元フロー
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — tiered storage の preservation_class 設計根拠
