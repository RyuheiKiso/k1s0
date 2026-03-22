# ADR-0016: Kafka KRaft モード移行計画

## ステータス

承認済み

## コンテキスト

現在、k1s0 の Kafka クラスター（Strimzi 管理）は ZooKeeper モードで稼働している。
バージョンは Kafka 3.6.1 であり、`kafka-cluster.yaml` にて ZooKeeper 3ノードが定義されている。

Apache Kafka コミュニティは KIP-833 において、**Kafka 4.0 で ZooKeeper サポートを完全に削除**することを決定した。
KRaft（KIP-500: Kafka Raft Metadata）はメタデータ管理を Kafka 自身の Raft コンセンサスで行う方式であり、
Kafka 3.3 以降で本番利用可能（GA）となっている。

移行を行わない場合、Kafka 4.0 へのバージョンアップが不可能となり、セキュリティパッチや
新機能の恩恵を受けられなくなるリスクがある。

## 決定

Strimzi の `KafkaNodePool` リソースと `UseKRaft` feature gate を使用して、
Kafka クラスターを ZooKeeper モードから **KRaft モード**に移行する。

移行は以下のフェーズで実施する：

1. **準備フェーズ**: Strimzi バージョンを KRaft GA 対応版（0.38 以降）に更新
2. **移行フェーズ**: `KafkaNodePool` を作成し、`UseKRaft` feature gate を有効化してローリング移行
3. **完了フェーズ**: ZooKeeper クラスターを削除し、`kafka-cluster.yaml` を KRaft 専用設定に更新

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
- Strimzi バージョンアップが前提条件となる
- 移行手順の検証のために、ステージング環境での事前テストが必須

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| ZooKeeper 維持 | 現状維持。Kafka 3.x 系に留まる | Kafka 4.0 以降サポート外。将来的にセキュリティパッチを受けられなくなる |
| 新規 KRaft クラスター構築 | ZooKeeper クラスターと並行して KRaft クラスターを新規構築し、データを移行 | 移行期間中の二重管理コスト、トピックデータ移行の複雑さ、ダウンタイムリスクが高い |
| MSK / Confluent Cloud 移行 | マネージド Kafka サービスへの移行 | ベンダーロックイン、コスト増大、現行の Strimzi ベース運用ノウハウが活かせない |

## 参考

- [KIP-500: Replace ZooKeeper with a Self-Managed Metadata Quorum](https://cwiki.apache.org/confluence/display/KAFKA/KIP-500%3A+Replace+ZooKeeper+with+a+Self-Managed+Metadata+Quorum)
- [KIP-833: Drop support for ZooKeeper mode in Kafka 4.0](https://cwiki.apache.org/confluence/display/KAFKA/KIP-833%3A+Drop+support+for+ZooKeeper+mode+in+Kafka+4.0)
- [Strimzi: Migrating from ZooKeeper to KRaft](https://strimzi.io/docs/operators/latest/deploying.html#con-kraft-mode-str)
- [infra/messaging/kafka/kafka-cluster.yaml](../../../infra/messaging/kafka/kafka-cluster.yaml)
