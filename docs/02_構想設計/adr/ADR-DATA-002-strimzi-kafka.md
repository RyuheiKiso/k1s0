# ADR-DATA-002: メッセージングに Strimzi Kafka を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / データ基盤チーム / 運用チーム

## コンテキスト

tier1 の PubSub API（FR-T1-PUBSUB-001〜005）、Workflow API のイベント駆動連携、Audit API のイベント配信、tier2/tier3 間の非同期連携には、スループット・永続性・At-least-once 配信を備えたメッセージング基盤が必須。オンプレミスで Kubernetes ベースの選択肢は Kafka 系か NATS / RabbitMQ / Pulsar に絞られる。

制約条件は以下の通り。

- オンプレミス、Kubernetes ネイティブ
- スループット 10,000 メッセージ/秒を目標
- メッセージ損失率 < 0.001%（at-least-once）
- 運用負荷最小化（採用側の小規模運用を想定）
- Dapr PubSub Building Block がネイティブに対応（多くのブローカーが対応）

Kafka は金融・大規模分析基盤で採用実績が厚く、Strimzi は Kafka の Kubernetes Operator で CNCF Graduated の成熟度を持つ。

## 決定

**メッセージングは Strimzi Operator で管理する Apache Kafka クラスタとする。**

- Apache Kafka 3.7+（KRaft モード、Zookeeper レス）
- 3 ブローカー最小構成、replication factor 3
- Strimzi Cluster Operator で宣言的管理
- Dapr PubSub Building Block の Kafka Component で tier2/tier3 へ透過化
- トピック自動生成は無効、Topic CRD で明示的管理（誤作成防止）
- At-least-once を基本、重要イベントは idempotency key で重複制御（PubSub API 仕様参照）
- 暗号化は転送時 TLS、保管時は LUKS（下位レイヤ）
- MirrorMaker2 で DR 用クラスタへのレプリケーション（採用側のスケール拡大時）

## 検討した選択肢

### 選択肢 A: Strimzi Kafka（採用）

- 概要: CNCF Graduated、Kafka を Kubernetes 上で運用する公式 Operator
- メリット:
  - Kafka の業界実績と Strimzi の運用成熟度の両立
  - KRaft モードで Zookeeper 依存を排除、運用コンポーネント削減
  - Topic / User / KafkaConnect を CRD で宣言的管理
  - Prometheus メトリクス、Grafana ダッシュボード標準提供
  - 多くのクライアントライブラリ（Go / Java / Rust / Python 等）
- デメリット:
  - Kafka の運用知識（partition 設計、consumer group、rebalance）が必要
  - JVM 前提のため、メモリ使用量が他選択肢より多い

### 選択肢 B: NATS / NATS JetStream

- 概要: CNCF Incubating、Go 製で軽量
- メリット: 軽量、運用シンプル、レイテンシ低い
- デメリット:
  - JetStream の永続性・耐久性が Kafka より薄い
  - 大規模データ分析用途との統合（Flink 等）が弱い

### 選択肢 C: RabbitMQ

- 概要: AMQP プロトコル、運用実績豊富
- メリット: ルーティング機能が柔軟、学習曲線緩やか
- デメリット:
  - 高スループット時のスケーリングに制約（ストリーム型には Kafka が優位）
  - Dapr PubSub の RabbitMQ Component はサポートしているが Kafka ほどの情報量・採用事例はない

### 選択肢 D: Apache Pulsar

- 概要: CNCF Graduated、階層ストレージで大規模対応
- メリット: Geo-replication、Multi-tenancy 標準対応
- デメリット:
  - Operator の成熟度が Strimzi より劣る（2026 時点）
  - BookKeeper 依存でコンポーネントが増える
  - Dapr の Pulsar Component は存在するが、採用事例は限定的

## 帰結

### ポジティブな帰結

- Kafka の業界実績で採用・育成が容易（市場の Kafka エンジニアを活用可能）
- Strimzi の宣言的管理で GitOps 親和性高
- MirrorMaker2 による DR クラスタ構成が採用側のスケール拡大時に選択可能
- Dapr PubSub で tier2/tier3 に透過化（内部変更時に tier2/tier3 コード無修正）

### ネガティブな帰結

- JVM ベースでメモリ使用量が他ブローカーより多い（3 ブローカーで 24GB メモリ目安）
- Kafka の partition 設計は業務初期に決定が難しい、後から変更困難
- Consumer Lag の継続的監視と HPA 連動が運用設計として必要（FMEA RPN 84）

## 実装タスク

- Strimzi Cluster Operator の Helm Chart バージョンを固定、Argo CD で管理
- KafkaTopic CRD を Backstage Software Template 化（tier2/tier3 がセルフサービスでトピック作成）
- partition 設計ガイドを Golden Path に収録（DX-GP-001）
- Consumer Lag 監視ダッシュボードを標準提供
- DR レプリケーション（MirrorMaker2）構成を採用側のマルチクラスタ移行時に POC

## 参考文献

- Strimzi 公式: strimzi.io
- Apache Kafka 3.7 Release Notes
- CNCF Graduated Projects
- Dapr PubSub Kafka Component Docs
