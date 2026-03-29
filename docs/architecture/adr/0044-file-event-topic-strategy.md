# ADR-0044: file-rust Kafka イベントトピック設計方針

## ステータス

承認済み

## コンテキスト

外部技術監査（M-21）にて、file-rust サービスの Kafka 設定と `create-topics.sh` のトピック定義に不整合が検出された。

- **file-rust の設定**: `topic_events: "k1s0.system.file.events.v1"` という単一の汎用トピックを使用し、ヘッダー（`event_type`）でアップロード・削除などのイベント種別を区別する設計
- **create-topics.sh の定義**: `k1s0.system.file.uploaded.v1` と `k1s0.system.file.deleted.v1` の2つのイベント別トピックが定義されていたが、file-rust が実際に使用する `k1s0.system.file.events.v1` は存在しなかった

この不整合により、file-rust が Kafka にイベントを発行しようとした際に「トピックが存在しない」エラーが発生する状態だった。

## 決定

**汎用トピック方針（file-rust の既存設計）を採用する。**

`k1s0.system.file.events.v1` をメイントピックとして create-topics.sh および topics.yaml に追加する。
既存の `k1s0.system.file.uploaded.v1` / `k1s0.system.file.deleted.v1` は将来のイベント種別分離用として維持するが、現時点では file-rust はこれらに直接送信しない。

## 理由

1. **実装と設定の最短整合**: file-rust の Rust 実装は `KafkaConfig::topic_events` という単一フィールドを参照しており、送信先を分岐させるための追加実装が不要
2. **ヘッダー方式の一般性**: Kafka ヘッダーによる `event_type` 区別は Saga/Outbox パターンとも整合しており、コンシューマーが必要なイベントのみフィルタリング可能
3. **将来の分割容易性**: 将来コンシューマー増加によりトピック分割が必要になった場合、ADR 改定で `k1s0.system.file.uploaded.v1` / `deleted.v1` に切り替えることができる（create-topics.sh にはすでに定義済み）

## 影響

**ポジティブな影響**:
- M-21 監査指摘が解消され、file-rust サービスの Kafka イベント発行が正常に機能する
- create-topics.sh と topics.yaml の双方に `events.v1` トピックが定義され整合性が保たれる

**ネガティブな影響・トレードオフ**:
- `k1s0.system.file.events.v1` への全イベント集約により、コンシューマー側でのヘッダーフィルタリングが必要になる
- `k1s0.system.file.uploaded.v1` / `deleted.v1` は現時点では空のトピックとして存在する（将来利用時まで）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A（本決定） | `events.v1` 汎用トピックを追加し file-rust の既存設計を維持 | 実装変更最小、整合容易 |
| 案 B | file-rust を `uploaded.v1` / `deleted.v1` に対応するよう修正 | Rust 実装の変更が必要。`KafkaProducer::publish` に分岐ロジック追加が必要 |

## 参考

- [外部技術監査報告書 M-21](../../../../報告書.md)
- [Kafka メッセージング設計](../../messaging/messaging.md)
- `infra/messaging/kafka/create-topics.sh`
- `infra/messaging/kafka/topics.yaml`
- `regions/system/server/rust/file/config/config.docker.yaml`

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成（M-21 監査対応） | Claude |
