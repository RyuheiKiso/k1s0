---
id: plan.data.scenario_restore_drill
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

# restore drill

## 一文方針

preservation_class 別の drill cadence に従い 4 種の restore drill を staging 隔離環境で実施し、restore_window 仕様との比較によって AND-gate（全 drill green でないと 1.0.0 ship 不可）を維持する。

> 月曜朝 9 時、本社 IT 室の data 担当者（シニア級）が `restore_drill.lock.yaml` を確認し、`v1_cross_region_replicated` の 30 日 drill cadence 到来を Backstage runbook 上で確認する。手元には Backstage runbook と Perses ダッシュボード、Mattermost 越しに infra 担当者・tier2 担当者・dual reviewer がいる。

## Trigger（発火条件）

drill cadence（preservation_class 別: 14〜180 日）の到来時、または DR 演習の指示があった時。

## 想定頻度 / 典型きっかけ

想定頻度: class 別 14〜180 日。典型きっかけ: 「v1_cross_region_replicated の drill cadence（30 日）が到来し、region failover シミュレーションを Backstage runbook から起動した」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: infra 担当者（network 分断模擬などインフラ操作が必要な場合）
- 関与: tier2 担当者（隔離環境での integration test 実行）
- 関与: dual reviewer（`restore_drill.lock.yaml` sign-off）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | `restore_drill.lock.yaml` / Perses / Backstage runbook | drill 種別選択・staging restore 実施・drill 結果記録 |
| 関与（infra）| シニア | 本社 IT 室 / リモート | Argo CD / Kyverno | network 分断模擬・インフラ操作支援 |
| 関与（tier2）| ミドル〜シニア | 本社 / リモート | Backstage TechDocs | 隔離環境での integration test 実行 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | `restore_drill.lock.yaml` sign-off |

## 前提

- 4 種の drill が Backstage runbook に登録済み
  - `full restore`: backup / snapshot からの完全復元
  - `cross-region failover`: region 切断を模擬した自動 failover 確認
  - `crypto-erase`: DEK revoke 後の data 不可読化確認
  - `archive_to_offline`: cold data の offline 転送と online copy の切り離し確認
- **AND-gate**: 全 drill green でないと 1.0.0 ship 不可の規約が確立済み
- `restore_drill.lock.yaml` が存在し、drill 結果の追記フォーマットが定義済み
- preservation_class 別の restore_window 仕様が定義済み（例: `v1_zone_replicated` = 60 sec）

## 流れ

1. Backstage runbook から対象 drill 種別と対象 preservation_class を選択し、drill 開始時刻を記録する
2. staging 隔離環境に対象 data store の snapshot / backup を restore する
3. restore 後の data 整合性を application layer で確認する。tier2 の integration test を隔離環境で実行し、全ケースが pass することを確認する
4. **cross-region failover drill**: cloud provider の region 切断を模擬する → CloudNativePG / Strimzi の failover 自動化が期待通りに動作することを確認する
5. drill 所要時間を計測し、対象 preservation_class の restore_window 仕様と比較する
6. 所要時間が仕様内なら **green** とし、次のステップへ進む。仕様超過なら **fail** とし、原因調査と改善 PR を必須とする
7. drill 結果（green / fail / 所要時間 / 原因）を `restore_drill.lock.yaml` に追記し、dual reviewer sign-off を得る

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **警報配信**: RTO 要件が最も厳しい業務の一つであり、restore_drill での cross-region failover drill 結果が直接的な警報配信の継続性保証に連動する。
- **SCADA 連携**: SCADA テレメトリデータの restore_window が仕様内に収まることの確認は、製造ライン稼働監視の連続性維持に直結する。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: CloudNativePG / Strimzi / Rook+Ceph / Backstage

## 期待結果 / 観測指標

- 4 種の drill が全て green（restore_window 仕様内）
- `restore_drill.lock.yaml` に drill 結果が追記済み
- dual reviewer 2 名の sign-off が記録済み
- AND-gate が維持されており、1.0.0 ship blocker が解除状態

## 失敗時の挙動 / escalation

- **drill fail（restore_window 超過）**: 1.0.0 ship blocker として Backstage ticket 起票。次 drill までに改善計画を data 担当者が提出（SLA: 5 営業日以内）。ops 担当者に Mattermost `#data-drill-fail` で通報。
- **integration test fail**: data 不整合が存在する可能性があるため、drill を中断し data 担当者と tier2 担当者が協力して原因を特定する。
- **drill 中に staging 環境が破壊された**: staging 環境を再構築し、drill を再実施する。本番には影響しない。

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — restore_drill AND-gate の設計思想と preservation_class 別 drill cadence の根拠
- [preservation_class 変更シナリオ](02_preservation_class変更.md) — class 変更に伴う drill cadence 更新と AND-gate 通過の手順
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
