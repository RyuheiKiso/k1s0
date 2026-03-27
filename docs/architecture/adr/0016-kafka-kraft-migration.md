# ADR-0016: Kafka KRaft モード移行計画

## ステータス

承認済み（2026-03-26 更新）

## コンテキスト

現在の環境状況は以下のとおり：

| 環境 | 状態 | Kafka バージョン | モード |
|------|------|-----------------|--------|
| docker-compose（開発） | **KRaft 移行済み** | apache/kafka:3.8.0 | KRaft |
| K8s 本番（Strimzi） | ZooKeeper モード稼働中 | 3.6.1 | ZooKeeper |

docker-compose 開発環境は `apache/kafka:3.8.0` イメージを使用し、ZooKeeper なしの KRaft 単一ノード構成で稼働中。
K8s 本番環境は Strimzi 管理の Kafka クラスターであり、`kafka-cluster.yaml` にて ZooKeeper 3ノードが定義されている。

Apache Kafka コミュニティは KIP-833 において、**Kafka 4.0 で ZooKeeper サポートを完全に削除**することを決定した。
KRaft（KIP-500: Kafka Raft Metadata）はメタデータ管理を Kafka 自身の Raft コンセンサスで行う方式であり、
Kafka 3.3 以降で本番利用可能（GA）となっている。

移行を行わない場合、Kafka 4.0 へのバージョンアップが不可能となり、セキュリティパッチや
新機能の恩恵を受けられなくなるリスクがある。

## 決定

Strimzi の `KafkaNodePool` リソースと `UseKRaft` feature gate を使用して、
K8s 本番環境の Kafka クラスターを ZooKeeper モードから **KRaft モード**に移行する。

### 実施フェーズ

| フェーズ | 対象環境 | 内容 | ステータス |
|----------|----------|------|-----------|
| Phase 1 | docker-compose | KRaft 化（apache/kafka:3.8.0 単一ノード） | **完了** |
| Phase 2 | K8s staging | Strimzi KRaft サポートの GA 確認・ステージング環境移行 | 2026-Q2 予定 |
| Phase 3 | K8s prod | 本番環境移行 | 2026-Q3 予定 |

**Phase 2 前提条件**:
1. Strimzi バージョンを KRaft GA 対応版（0.38 以降）に更新
2. `KafkaNodePool` CRD の導入確認
3. ステージング環境でのローリング移行検証

**Phase 3 前提条件**:
1. Phase 2 でのステージング環境移行が完了し、2週間以上安定稼働を確認
2. ZooKeeper クラスターのリタイア手順を文書化

## 理由

- **将来性**: Kafka 4.0 以降で ZooKeeper サポートが削除されるため、継続利用には KRaft 移行が必須
- **運用簡素化**: ZooKeeper クラスター（3ノード）が不要となり、管理対象コンポーネントが減少する
- **パフォーマンス**: KRaft モードはコントローラーのフェイルオーバー時間が大幅に短縮される（ZooKeeper 方式比）
- **Strimzi サポート**: Strimzi は ZooKeeper から KRaft へのインプレース移行ツールを提供しており、
  ダウンタイムなしでの移行が可能

## 影響

**ポジティブな影響**:

- ZooKeeper クラスター（3ノード × 20Gi）が不要となり、インフラコストが削減される
- メタデータ管理のレイテンシが改善される
- Kafka 4.0 以降へのバージョンアップが可能になる
- 障害ポイントが減少し、クラスター全体の可用性が向上する

**ネガティブな影響・トレードオフ**:

- 移行作業中はクラスター設定変更が制限される（ローリング再起動が必要）
- `kafka-cluster.yaml` の大幅な構造変更が必要（`KafkaNodePool` リソース追加、`zookeeper` セクション削除）
- Strimzi Operator の `KafkaNodePool` CRD への移行が必要
- ZooKeeper クラスター（3 Pod）のリタイアにより関連マニフェスト・監視設定の更新が必要
- Strimzi バージョンアップが前提条件となる
- 移行手順の検証のために、ステージング環境での事前テストが必須

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| ZooKeeper 維持 | 現状維持。Kafka 3.x 系に留まる | Kafka 4.0 以降サポート外。将来的にセキュリティパッチを受けられなくなる |
| 新規 KRaft クラスター構築 | ZooKeeper クラスターと並行して KRaft クラスターを新規構築し、データを移行 | 移行期間中の二重管理コスト、トピックデータ移行の複雑さ、ダウンタイムリスクが高い |
| MSK / Confluent Cloud 移行 | マネージド Kafka サービスへの移行 | ベンダーロックイン、コスト増大、現行の Strimzi ベース運用ノウハウが活かせない |

## 追記: Kafka TLS 専用化（M-7 監査対応）

**更新日:** 2026-03-26

### 変更内容

外部技術監査（M-7）の指摘を受け、K8s 本番環境の Kafka リスナー構成を変更した。

| 変更前 | 変更後 |
|--------|--------|
| plain リスナー（port 9092, TLS なし）+ TLS リスナー（port 9093） | TLS リスナー（port 9093）のみ |

### 変更理由

- ゼロトラストアーキテクチャの観点から、クラスター内部通信であっても平文通信は許容しない
- plain リスナー経由での盗聴・なりすましリスクを排除する
- NetworkPolicy 側でも port 9092 のアクセスを廃止し、9093 のみを許可するよう統一する

### 影響範囲

| ファイル | 変更内容 |
|---------|---------|
| `infra/terraform/modules/messaging/main.tf` | `plain` リスナー定義を削除 |
| `infra/kubernetes/network-policies/messaging.yaml` | Ingress allow で 9092 を削除、9093 のみ許可 |
| `infra/kubernetes/network-policies/system.yaml` | Egress allow で 9092 を削除、9093 のみ許可 |
| `infra/kubernetes/network-policies/business.yaml` | Egress allow で 9092 を削除、9093 のみ許可 |
| `infra/kubernetes/network-policies/service.yaml` | Egress allow で 9092 を削除、9093 のみ許可 |

### 開発環境との差異

docker-compose 開発環境（`apache/kafka:3.8.0` KRaft 構成）は PLAINTEXT を継続使用する。
これは開発者の利便性とローカル証明書管理の複雑さを避けるためであり、
本番 K8s 環境との意図的な差異として文書化する。

---

## 参考

- [KIP-500: Replace ZooKeeper with a Self-Managed Metadata Quorum](https://cwiki.apache.org/confluence/display/KAFKA/KIP-500%3A+Replace+ZooKeeper+with+a+Self-Managed+Metadata+Quorum)
- [KIP-833: Drop support for ZooKeeper mode in Kafka 4.0](https://cwiki.apache.org/confluence/display/KAFKA/KIP-833%3A+Drop+support+for+ZooKeeper+mode+in+Kafka+4.0)
- [Strimzi: Migrating from ZooKeeper to KRaft](https://strimzi.io/docs/operators/latest/deploying.html#con-kraft-mode-str)
- [infra/messaging/kafka/kafka-cluster.yaml](../../../infra/messaging/kafka/kafka-cluster.yaml)
