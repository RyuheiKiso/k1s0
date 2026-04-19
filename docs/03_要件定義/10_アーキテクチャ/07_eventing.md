# ARC-EVT: イベンティング要件

本ファイルは、Kafka をイベントバックボーンとし、Dapr Pub/Sub で抽象化したうえで、Dapr Workflow / Temporal の 2 基盤に振り分けるイベント駆動アーキテクチャの要件を定義する。

イベンティングは tier1 が tier2 / tier3 に提供する最も重要な抽象の 1 つであり、ここが設計ミスすると「イベント喪失・重複処理・順序逆転」という本番障害を直接的に引き起こす。一方で Kafka / Dapr / Temporal を tier2 / tier3 に直接露出すると、基盤知識を持たない業務アプリ開発者が at-least-once 配信や冪等性の考慮を各自実装する羽目になり、品質が分散する。

本ファイルの要件は、Kafka 固定・Pub/Sub 抽象・ワークフロー振り分け・Schema Registry・at-least-once と冪等性・DLQ（Dead Letter Queue）の 6 論点をカバーする。Workflow 振り分けの複雑化は `COM-RSK-002` として High リスク認定済みであり、本ファイルでも重点的に扱う。

---

## 前提

- [`../00_共通/04_risk.md`](../00_共通/04_risk.md) `COM-RSK-002` Workflow 振り分け複雑化
- [`01_tier1.md`](./01_tier1.md) `ARC-T1-002` Dapr 隠蔽 / `ARC-T1-006` ワークフロー振り分け
- [`05_api_sdk_contract.md`](./05_api_sdk_contract.md) `ARC-API-003` Protobuf 後方互換
- [`../../01_企画/企画書.md`](../../01_企画/企画書.md) 13.5 章 ワークフロー振り分け

---

## 要件本体

### ARC-EVT-001: Kafka をイベントバックボーンとして採用

- 優先度: MUST（後付け変更は tier2 / tier3 全アプリの改修を要するため Phase 1a で確定必須）
- Phase: Phase 1a
- 関連: `COM-CON-004`, `OPS-REL-*`

現状、イベントバックボーンの選択（Kafka / NATS JetStream / RabbitMQ / Redis Streams）は JTC 環境の既存資産と運用知見の観点から Kafka が最有力だが、確定していないと tier2 / tier3 側が実装方針を決められない。

本要件が満たされた世界では、Kafka が k1s0 のイベントバックボーンとして固定される。OSS ライセンスは Apache 2.0 で `COM-CON-004` に適合する。Phase 1a 時点では Strimzi（Kubernetes operator）経由で self-managed 構成とし、Phase 2 以降の規模拡大で AWS MSK / Confluent Cloud 等への移行は可否判断する。tier2 / tier3 は Kafka に直接アクセスせず、`ARC-EVT-002` の Dapr Pub/Sub 抽象経由でのみ利用する。

崩れた場合、tier2 / tier3 のコードに Kafka クライアント SDK が直接 import され、将来のバックボーン変更で全アプリの改修が必要になる。Phase 2 の MSK 移行判断で凍結期間が発生する。

**受け入れ基準**

- Kafka が Strimzi 経由で Phase 1a に導入されている
- tier2 / tier3 から Kafka クライアントの直接 import が CI で拒否される
- Phase 2 でのマネージド移行判断の評価軸が文書化されている

**検証方法**

- tier2 / tier3 の依存解析で Kafka クライアント不在を確認
- Strimzi の Operator ヘルスチェック

---

### ARC-EVT-002: Pub/Sub 抽象は Dapr Pub/Sub を介する

- 優先度: MUST（Kafka 直接利用では Dapr 隠蔽原則 `ARC-T1-002` が崩れる）
- Phase: Phase 1a
- 関連: `ARC-T1-002`, `ARC-EVT-001`

現状、tier2 / tier3 が Kafka のトピック名・パーティション戦略・プロデューサ設定を直接扱うと、Kafka 運用の知識が必須となり、tier2 オンボーディング時間が大幅に延びる。

本要件が満たされた世界では、tier2 / tier3 は tier1 の Pub/Sub 抽象 API（Dapr Pub/Sub component 経由）のみを使う。トピック名は tier1 側が命名規約で管理し、tier2 / tier3 は論理的な「ビジネスイベント名」で送信・購読する。Dapr Pub/Sub の backing store を将来 Kafka 以外（例: NATS JetStream）に切り替える場合も tier2 / tier3 のコード変更は発生しない。

崩れた場合、tier2 コードに Kafka 固有設定が散在し、運用変更（パーティション数変更・retention 変更）のたびにアプリ改修が必要となる。

**受け入れ基準**

- tier2 / tier3 が Dapr Pub/Sub 抽象のみを利用する
- トピック命名規約が tier1 側で集中管理されている
- Dapr Pub/Sub component の backing store 差し替え手順が Runbook にある

**検証方法**

- CI の依存解析で Kafka クライアント不在を確認
- Dapr component YAML のスキーマ検査

---

### ARC-EVT-003: Dapr Workflow vs Temporal の振り分け基準

- 優先度: MUST（`COM-RSK-002` High リスク。振り分けが曖昧だと運用工数前提が崩壊）
- Phase: Phase 1b
- 関連: `COM-RSK-002`, `ARC-T1-006`

現状、Dapr Workflow と Temporal は機能が重なる部分があり、「どちらをどの業務に使うか」を設計段階で決めないと、開発者ごとに選択が分岐する。運用工数 0.5 人月/年の前提が下振れする確度 70%（`COM-RSK-002`）と査読で指摘されている。

本要件が満たされた世界では、振り分け基準は単一の決定ツリーに集約される。基準は「実行時間 5 分未満 + 人的承認なし → Dapr Workflow、それ以外 → Temporal」を起点にし、境界ケース（5 分 Saga + 人的承認など）は明示的な拡張ルールとして OPA / Rego で記述する。tier2 からは tier1 の `StartWorkflow` API に渡す `workflow_class` に従い振り分け先が決定され、基盤側は不可視になる。MVP-1a で 20+ の実ワークフローパターンをシミュレーションし、決定ツリーの妥当性を検証する。

崩れた場合、開発者ごとに選択が分岐し、運用チームが両基盤の障害対応手順を保守する必要が生じ、企画書の 2 名運用前提が破綻する。

**受け入れ基準**

- 振り分け基準が決定ツリーとして単一定義されている
- 振り分けロジックが OPA / Rego で実装されている
- MVP-1a で 20+ ワークフローパターンのシミュレーションが完了している
- tier2 側から基盤の差異が不可視である

**検証方法**

- OPA の conftest による単体テスト
- 月次の振り分け判断ダッシュボードレビュー

---

### ARC-EVT-004: イベントスキーマ管理（Schema Registry）

- 優先度: MUST（スキーマ不整合は本番でしか気づけない・気づいた時は遅い）
- Phase: Phase 1a
- 関連: `ARC-API-003`, `COM-CON-002`

現状、Kafka に流れるイベントのスキーマが送信側・受信側で暗黙的に管理されると、片側のスキーマ変更が受信側の Marshal エラーを引き起こす。Phase 2 以降の大規模展開で特に顕在化する。

本要件が満たされた世界では、イベントスキーマは Protobuf で定義され、Confluent Schema Registry（または互換実装）に登録される。送信側は Protobuf Marshal したバイナリを Kafka に流し、受信側は Schema Registry から取得したスキーマで Unmarshal する。スキーマ変更は `ARC-API-003` と同じ後方互換規則（フィールド番号再利用禁止等）に従い、breaking change は CI で拒否される。

崩れた場合、tier2 A がスキーマ変更を本番反映した瞬間、tier2 B の受信側で Marshal エラーが発生し、メッセージが DLQ に滞留する。

**受け入れ基準**

- 全イベントスキーマが Protobuf で定義されている
- Schema Registry で集中管理されている
- buf breaking と同等の後方互換検査が CI で通過する

**検証方法**

- Schema Registry のスキーマ登録履歴レビュー
- CI の互換性検査通過

---

### ARC-EVT-005: At-least-once 配信と冪等性の扱い

- 優先度: MUST（冪等性が壊れると金融業務で二重請求・二重引き落としが発生）
- Phase: Phase 1a
- 関連: `SEC-DAT-*`, `ARC-RUL-*`

現状、Kafka / Dapr Pub/Sub は at-least-once 配信を保証するため、受信側で冪等性を担保しないと、同一イベントの複数処理で業務データが破壊される。

本要件が満たされた世界では、全イベントに `event_id`（ULID）が必須フィールドとして付与され、受信側は `event_id` を基に冪等性キーとして処理済み判定を行う。処理済み判定ストアは tier1 が Redis / Postgres で提供し、tier2 / tier3 側のアプリコードは `IdempotentHandler` ラッパー経由でハンドラを登録するだけで冪等性が担保される構造とする。重複検出期間は 7 日間を標準とする。

崩れた場合、ネットワーク再送により金融業務で二重請求・二重引き落としが発生し、顧客補償と監査対応で数千万円規模のインシデントとなる。

**受け入れ基準**

- 全イベントに `event_id`（ULID）必須である
- tier1 が冪等性処理ストアを提供する
- `IdempotentHandler` ラッパーが SDK に組み込まれている
- 重複検出期間が 7 日間である

**検証方法**

- SDK の冪等性 E2E テスト
- tier3 向けコード例の冪等性検査

---

### ARC-EVT-006: Dead Letter Queue（DLQ）運用

- 優先度: MUST（処理不能イベントの放置は監査証跡欠落・業務データ欠落を招く）
- Phase: Phase 1b
- 関連: `ARC-EVT-005`, `OPS-INC-*`

現状、処理不能イベントがトピック内で無限リトライされると、バックプレッシャで正常イベントまで遅延する。

本要件が満たされた世界では、最大リトライ回数（標準 5 回）を超えたイベントは DLQ トピックに退避される。DLQ には原イベント・失敗理由・最終処理時刻が保存され、運用ダッシュボードで一覧可能となる。DLQ のイベントは運用チームが原因調査後、手動再投入または棄却判断を行う。DLQ のサイズ閾値超過時はアラートが発報される。

崩れた場合、処理不能イベントが正常処理をブロックし、tier1 API の p99 が間接的に悪化する。DLQ なしでの棄却は監査証跡欠落として指摘される。

**受け入れ基準**

- 全 Pub/Sub トピックに DLQ が設定されている
- DLQ 運用ダッシュボードが提供されている
- DLQ サイズ閾値超過アラートが定義されている

**検証方法**

- DLQ 発生時の訓練を四半期 1 回実施
- アラート発報のテスト

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| ARC-EVT-001 | Kafka をバックボーンとして採用 | MUST | 1a |
| ARC-EVT-002 | Dapr Pub/Sub 抽象の介在 | MUST | 1a |
| ARC-EVT-003 | Workflow 振り分け基準 | MUST | 1b |
| ARC-EVT-004 | Schema Registry によるスキーマ管理 | MUST | 1a |
| ARC-EVT-005 | At-least-once + 冪等性 | MUST | 1a |
| ARC-EVT-006 | DLQ 運用 | MUST | 1b |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 6 | ARC-EVT-001, 002, 003, 004, 005, 006 |

---

## 関連図

イベント流路（tier3 → tier1 Pub/Sub 抽象 → Kafka → 受信 tier2 → Workflow 振り分け）を示す drawio 図は後続タスクで追加する。
