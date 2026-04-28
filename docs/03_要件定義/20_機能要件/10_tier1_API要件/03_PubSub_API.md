# PubSub API

本書は、tier1 が公開する PubSub API の機能要件を定義する。tier2/tier3 のイベント配信・購読を、Apache Kafka（Strimzi Operator）バックエンドで提供する。

## API 概要

業務イベント（申請完了、承認、キャンセル等）の非同期配信、サービス間の疎結合連携、監査イベントの集約を対象とする。at-least-once 配信を基本とし、重複配信に対して tier2 側は冪等に実装することを前提とする。exactly-once は本 API では保証しない。

内部実装は Dapr PubSub Building Block の Go SDK を利用し、バックエンドは Strimzi Operator 上の Kafka（KRaft モード、Zookeeper 不要）。

## 機能要件

### FR-T1-PUBSUB-001: Publish / Subscribe 基本機能

**現状**: tier2 が Kafka Producer / Consumer を直接使うと、ブローカー接続、シリアライズ、consumer group、offset 管理を個別に実装する必要がある。言語別の Kafka クライアントライブラリは機能差があり、挙動が揃わない。

**要件達成後**: `k1s0.PubSub.Publish(topic, event)`、`Subscribe(topic, handler)` で配信と購読を提供する。tier1 ファサードが接続・シリアライズ・offset 管理を吸収。イベントは Protobuf または JSON 形式で配信され、W3C Trace Context は自動注入される。

**崩れた時**: 言語別にクライアント実装がバラつき、イベント形式・トレース連携が揃わず、下流の監査・分析で相関が取れない。

**受け入れ基準**:

- Go / C# / Rust / Python の各 SDK で同名の API を提供
- Publish は非ブロッキング、完了通知はコールバックまたは Future 経由
- Subscribe はバックグラウンドで実行され、handler 完了時点で offset コミット
- イベントに `tenant_id`、`trace_id`、`event_id`、`published_at` が自動付与される

### FR-T1-PUBSUB-002: at-least-once 配信保証

**業務根拠**: BR-PLATGOV-002（業務イベント欠損は請求漏れ等の金銭的影響を引き起こすため）。

**現状**: Kafka は設定次第で at-most-once / at-least-once / exactly-once が選べるが、デフォルト設定は環境により異なり、実運用で「本当に at-least-once か」が検証しにくい。社内既存システムの業務イベント（経費精算・承認通知）では、月次で 5〜10 件のイベント欠損が報告されており、うち 1〜2 件は請求漏れ（平均影響額 5 万円/件）に直結している。年間換算で約 120 万円の機会損失が発生している計算。

**要件達成後**: tier1 が at-least-once をデフォルトとし、consumer の handler 完了後に offset コミットを実施する。handler 中のクラッシュは再配信となる。tier2 側は handler 内で冪等性を担保する（`event_id` でのデデュープを推奨）。年間 120 万円規模の請求漏れリスクが構造的に解消され、年次の業務監査で「欠損 0 件」を維持することで、経理部門の補正工数（月 8 時間・年間 96 時間）も削減される。

**崩れた時**: 重要イベント（承認、決済）が失われるリスクが顕在化する。tier2 開発者は独自のリトライ・冪等化を実装する必要が生じ、20 チーム × 8 人時の重複工数（160 人時）が発生。実装品質のバラつきにより、アプリ間で「あのアプリはなぜイベントが届かないのか」という相関不能な不具合レポートが月 5 件程度発生し、障害調査の長時間化を招く。監査部門からは「請求漏れの根本原因特定が困難」として重大指摘対象となる可能性がある。

**受け入れ基準**:

- handler 中のパニック / 例外は offset 未コミットとなり、次回再配信される
- Kafka Producer 側は `acks=all`、`enable.idempotence=true` を強制
- `event_id` は tier1 が UUID v7 で自動生成（時系列ソート可能）

### FR-T1-PUBSUB-003: Consumer Group 管理

**現状**: Kafka Consumer Group の命名、メンバーシップ管理、rebalance 発生時の挙動を tier2 開発者が理解する必要がある。Dapr は consumer group をアプリ単位で自動命名するが、tier1 的に揃えたい。

**要件達成後**: Consumer Group 名は `k1s0.<tenant_id>.<service_name>` 形式で tier1 が自動付与する。tier2 は Subscribe 時にこの命名を意識しない。rebalance 時の挙動は Kafka のデフォルト（sticky assignor）で統一。

**崩れた時**: Consumer Group の命名衝突で複数アプリがイベントを奪い合う、rebalance 暴走で offset が揺れる、といった事故が発生する。

**受け入れ基準**:

- 同一 service_name の複数インスタンスは同一 Consumer Group として動作
- Consumer Group の再割り当て（rebalance）中のイベントロストが発生しない

### FR-T1-PUBSUB-004: Dead Letter Queue

**現状**: handler が繰り返し失敗するイベントは、Kafka 上で再配信ループを起こし Consumer Group を停滞させる。Dapr の DLQ 機能は Component 設定次第。

**要件達成後**: handler が連続 3 回失敗したイベントは自動的に `k1s0.<tenant_id>.dlq.<service_name>` トピックへ転送される。DLQ トピックは情シス SRE が専用ダッシュボードで監視する。再処理は SRE が手動で起動する Runbook 手順で行う。

**崩れた時**: 失敗イベントで Consumer が停滞し、下流業務が詰まる。SRE が Kafka コマンドで offset を手動進める羽目になる。

**受け入れ基準**:

- 失敗回数閾値（デフォルト 3）を Component YAML で設定可能
- DLQ 転送時に元イベント、失敗理由、試行回数を保持
- DLQ トピックの滞留数を Prometheus で可視化、閾値超過でアラート

### FR-T1-PUBSUB-005: トピック命名規約強制（tenant 分離）

**現状**: テナント越境を防ぐには、トピック名に `tenant_id` を含める規約を徹底する必要があるが、tier2 実装者が書き忘れると隔離が崩れる。

**要件達成後**: Publish / Subscribe 時のトピック名は `k1s0.<tenant_id>.<domain>.<event_type>` 形式で tier1 が自動展開する。tier2 は `<domain>.<event_type>` のみ指定し、`tenant_id` は JWT クレームから自動挿入される。他テナントのトピックへの Publish / Subscribe は Kafka ACL で拒否される。

**崩れた時**: テナント越境イベント配信で情報漏えいが発生し、監査で重大インシデント扱いとなる。

**受け入れ基準**:

- tier2 コードにテナント ID を含むトピック名が書かれていない（CI で検出）
- 他テナントのトピック名を指定すると `K1s0Error.Forbidden` を返す
- Kyverno ポリシーで tenant_id を含まないトピック作成を拒否

## 入出力仕様

本 API の機械可読な契約骨格（Protobuf IDL）は [40_tier1_API契約IDL/03_PubSub_API.md](../40_tier1_API契約IDL/03_PubSub_API.md) に定義されている。SDK 生成・契約テストは IDL 側を正とする。以下は SDK 利用者向けの疑似インタフェースであり、IDL の `PubSubService` RPC と意味論的に対応する。

```text
k1s0.PubSub.Publish(
    domain: string,
    event_type: string,
    event: Proto message | JSON,
    options?: {
        partition_key?: string,  // 順序保証のパーティションキー
        headers?: map<string, string>
    }
) -> (event_id: string, error: K1s0Error?)

k1s0.PubSub.Subscribe(
    domain: string,
    event_type: string,
    handler: func(event, context) -> error?,
    options?: {
        max_concurrency?: int,   // default 10
        batch_size?: int         // default 1
    }
) -> Subscription
```

エラー型には `TopicNotFound`、`Forbidden`（テナント越境）、`BrokerUnavailable` を追加。

## 受け入れ基準（全要件共通）

- イベントサイズ上限 1MB（Kafka メッセージ上限に合わせる）
- Publish / Subscribe の全処理で W3C Trace Context が伝搬され、trace から consumer 処理までつながる
- 順序保証が必要な場合は partition_key で同一パーティション割り当て
- Consumer Lag を Prometheus で可視化、閾値超過でアラート

## 段階対応

- **リリース時点**: 未提供
- **リリース時点**: FR-T1-PUBSUB-001、002、003、005（Go / C# SDK）
- **リリース時点**: FR-T1-PUBSUB-004（DLQ）追加
- **採用後の運用拡大時**: Python / Rust SDK、Schema Registry 連携検討

## 関連非機能要件

- **NFR-B-PERF-005**: Publish レイテンシ p99 < 50ms
- **NFR-A-FT-002**: Kafka ブローカー 1 台障害時の継続性（3 ブローカー構成、採用側のマルチクラスタ移行時）
- **NFR-E-AC-003**: tenant_id 越境配信の禁止
- **NFR-C-NOP-001**: Consumer Lag の監視と可視化
