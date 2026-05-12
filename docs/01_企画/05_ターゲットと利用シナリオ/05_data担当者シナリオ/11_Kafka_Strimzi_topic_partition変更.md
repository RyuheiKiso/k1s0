---
id: plan.data.scenario_kafka_topic_partition
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, D]
  proof_classes: []
---

# Kafka / Strimzi topic・partition 変更

## 一文方針

data 担当者が Strimzi KafkaTopic CRD で管理する Kafka topic の追加・partition 数変更・retention / compaction policy 変更を GitOps 経由で実施し、Apicurio Registry での schema 登録と `kafka_topic.lock.yaml` への記録を完結させる。

## Trigger（発火条件）

tier2 担当者から新規 Domain Event の Kafka topic 追加依頼が届いた時、または既存 topic の partition 数変更・retention policy 変更・compaction policy 変更が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 月次〜四半期（新規 Domain Event 追加 / tier2 業界 pack 拡張に追従）。典型きっかけ: 「tier2 担当者が製造ライン稼働監視の新 Domain Event を追加し、対応 Kafka topic `manufacturing.line.status.v1` と partition 8 の作成を依頼してきた」「既存 topic `orders.created.v1` の throughput 増加に伴い partition 4 → 8 への増加が必要になった」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: tier2 担当者（Domain Event schema の提供 / Apicurio schema 登録の確認）
- 関与: ops 担当者（partition 変更後の Kafka consumer lag 監視）
- 承認: dual reviewer（data 担当者 2 名、PR author 不可）

## 前提

- Strimzi Kafka Operator が cluster に deploy 済みで、KafkaTopic CRD が利用可能
- Apicurio Registry が稼働しており、Avro schema の登録が可能な状態
- `kafka_topic.lock.yaml` が `data/kafka/` 配下に存在し、全 topic の current state が記録済み
- partition 数の**減少**は Kafka の制約で不可能（増加のみ）。減少が必要な場合は topic 再作成（tier2 担当者と事前調整必須）
- 変更する topic を消費している全 Consumer Group の lag が正常範囲内であること

## 流れ

1. tier2 担当者から受領した要件を確認する（topic 名 / partition 数 / retention.ms / cleanup.policy / compaction 有無）
2. 新規 topic の場合: `kafka/topics/<topic-name>.yaml`（KafkaTopic manifest）を作成し PR を提出する
   - `strimzi.io/kafka-cluster: <cluster-name>` ラベルを付与
   - `spec.partitions` / `spec.replicas` / `spec.config.retention.ms` / `spec.config.cleanup.policy` を設定
3. 既存 topic の partition 増加の場合: manifest の `spec.partitions` を更新し PR を提出する
   - Consumer Group の partition assignment が自動リバランスされることを ops 担当者と確認
4. Apicurio Registry に Avro schema を登録する（tier2 担当者が schema ファイルを提供）
   - compatibility strategy: `BACKWARD`（デフォルト）を確認し、tier2 担当者が意図する互換性レベルと一致するか確認する
   - schema ID を `kafka_topic.lock.yaml` の `schema_registry_id` フィールドに記録する
5. GitOps（Argo CD）経由で Strimzi Operator に apply する
   - KafkaTopic CRD の `status.conditions[type=Ready]` が `True` になることを確認する
6. 変更後の動作確認:
   - staging で tier2 の Domain Event 送受信 integration test（Testcontainers + embedded Kafka）を実行し全 test green を確認
   - partition 変更の場合: Consumer Group の lag が変更前と同等以下であることを Perses で確認（30 分観察）
7. `kafka_topic.lock.yaml` を更新し（topic 名 / partition 数 / retention / schema_registry_id / compaction policy）、dual reviewer sign-off を取得する

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- data 強制機構: [../../../04_詳細設計/02_強制機構/05_data強制機構.md](../../../04_詳細設計/02_強制機構/05_data強制機構.md)
- 関連 OSS: Kafka（Strimzi）/ Apicurio Registry（schema registry）/ Argo CD（GitOps）/ Perses（Consumer lag 監視）

## 期待結果 / 観測指標

- artifact: `kafka_topic.lock.yaml` に対象 topic の partition 数 / retention / schema_registry_id が更新済み
- artifact: Apicurio Registry に Avro schema が登録済みで `schema_registry_id` が lock.yaml と一致
- ci: staging での tier2 Domain Event 送受信 integration test が all green（`kafka-topic-conformance` CI job）
- runtime: KafkaTopic CRD の `status.conditions[type=Ready] = True`（Strimzi Operator の reconcile 完了）
- slo: partition 変更後 30 分で Consumer Group lag が変更前の水準以下（Perses `kafka-consumer-lag` パネル）
- sign-off: dual reviewer（data 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **KafkaTopic CRD が `Ready = False` のまま**: Strimzi Operator のログを確認し、infra 担当者と協力して Operator の再起動またはマニフェスト修正を実施。Mattermost `#data-incident` に通報（**SLA: 30 分以内**）。
- **Apicurio Registry への schema 登録で互換性違反が発生**: tier2 担当者と schema の互換性 strategy を再調整する。`BACKWARD` で受け入れられない場合は `FULL_TRANSITIVE` への変更または新 subject で登録する（tier2 担当者の確認必須）。
- **partition 増加後に Consumer lag が増大**: ops 担当者と共同でリバランス状況を確認し、consumer 側のスケールアップまたは consumer group の設定変更を実施（**SLA: 1h 以内**）。
- **staging integration test fail**: tier2 担当者に Mattermost `#data-incident` で即時通報（**SLA: 15 分以内**）。本番適用を中止し原因を調査する。

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [schema migration](./01_schema_migration.md) — Domain Event の Avro schema 変更が伴う場合の expand-contract 手順
- [Outbox / atomic 三表書込障害対応](./08_Outbox_atomic三表書込障害対応.md) — Kafka publish 経路の障害時対応
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — 5 preservation_class の正典定義と retention policy の設計根拠
