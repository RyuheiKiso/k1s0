# infra/dapr/subscriptions — Dapr Subscription CRD 定義

tier1 アプリが PubSub（`infra/dapr/components/pubsub/kafka.yaml`）経由でイベント受信する際の Subscription CRD。

## 設計正典

- [`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/04_Dapr_Component配置.md`](../../../docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/04_Dapr_Component配置.md)

## 同梱 Subscription

| Subscription | topic | route | 受信 app |
|---|---|---|---|
| `audit-pii.yaml` | `audit-events` | `POST /v1/audit/ingest` | `tier1-audit-pii`（COMP-T1-AUDIT） |
| `feature.yaml` | `feature-flag-updates` | `POST /v1/feature/refresh` | `tier1-feature`（t1-state Pod の Feature handler） |

各 Subscription は `deadLetterTopic` で再試行限度超過メッセージを `<topic>-dlq` topic に退避する設計（at-least-once 配信の補完）。
