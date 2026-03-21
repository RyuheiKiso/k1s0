# Runbook インデックス

アラート発火時の対応手順書一覧。アラートの `runbook_url` annotation からリンクされる。

## 使い方

1. Alertmanager 通知の `runbook_url` リンクを開く
2. 該当 Runbook の「初動対応」セクションを確認する（5分以内に実施）
3. 状況に応じて「詳細調査」→「復旧手順」へ進む
4. エスカレーション基準に達した場合は上位にエスカレーション

新規 Runbook を作成する場合は `_template.md` を参照すること。

## 共通 Runbook 一覧

| アラート名 | ファイル | 重要度 |
|-----------|---------|--------|
| エラー率高騰 | [common/high-error-rate.md](./common/high-error-rate.md) | critical / warning |
| レイテンシ高騰 | [common/high-latency.md](./common/high-latency.md) | warning |
| Pod 再起動頻発 | [common/pod-restart.md](./common/pod-restart.md) | critical / warning |
| DB プール枯渇 | [common/db-pool-exhaustion.md](./common/db-pool-exhaustion.md) | critical |
| Kafka コンシューマーラグ | [common/kafka-consumer-lag.md](./common/kafka-consumer-lag.md) | warning |
| サービスダウン | [common/service-down.md](./common/service-down.md) | warning |
| SLO バーンレート critical | [common/slo-burn-rate-critical.md](./common/slo-burn-rate-critical.md) | critical |
| SLO バーンレート warning | [common/slo-burn-rate-warning.md](./common/slo-burn-rate-warning.md) | warning |
| TLS 証明書期限切れ | [common/certificate-expiring.md](./common/certificate-expiring.md) | critical / warning |

## インシデント管理

インシデント対応ワークフロー・エスカレーションパス・ポストモーテムについては
[../インシデント管理設計.md](../インシデント管理設計.md) を参照。

## テンプレート

新規 Runbook 作成の雛形: [_template.md](./_template.md)
