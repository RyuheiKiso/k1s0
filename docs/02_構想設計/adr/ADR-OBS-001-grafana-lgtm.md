# ADR-OBS-001: 観測性に Grafana LGTM スタックを採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / 運用チーム / SRE

## コンテキスト

k1s0 はマイクロサービスの集合であり、運用には Metrics / Logs / Traces の 3 本柱（Three Pillars of Observability）が必須となる。NFR-C-NOP-001（運用工数）、NFR-I-SLI-001（SLI 計測）、OR-INC-002（一次対応フロー）、FMEA の各故障モード検出など、観測性は全ての運用要件の前提となる。

制約条件は以下の通り。

- オンプレミス完結
- PromQL / LogQL / TraceQL などクエリ言語の統一学習コスト最小化
- OpenTelemetry 準拠
- 長期保管（メトリクス 13 ヶ月、ログ 1 年、トレースは短期でも可）
- AGPL 対応（Grafana 本体が AGPL のため ADR-0003 に従い分離）

候補は Grafana LGTM（Loki/Grafana/Tempo/Mimir）、ELK Stack、Prometheus + Jaeger + Kibana、Datadog / New Relic 等の商用 SaaS。

## 決定

**観測性スタックは Grafana LGTM（Loki / Grafana / Tempo / Mimir）を採用する。**

- **Loki**: 構造化ログ保管、LogQL クエリ（AGPL-3.0 / MIT デュアルライセンス、MIT 側で使用可）
- **Grafana**: ダッシュボード（AGPL-3.0、ADR-0003 に従い分離）
- **Tempo**: 分散トレース、TraceQL クエリ（AGPL-3.0）
- **Mimir**: 長期メトリクス保管、PromQL 互換（AGPL-3.0）
- OpenTelemetry Collector（ADR-OBS-002）経由でデータ投入
- tier1 Log API / Telemetry API の内部実装バックエンドとして利用

AGPL 対応として、各コンポーネントは独立した Pod で稼働させ、アプリからは OTLP / Prometheus Remote Write の公開プロトコルでのみ通信する。本体コードへの組込み、カスタムプラグインの埋込みは禁止。

## 検討した選択肢

### 選択肢 A: Grafana LGTM（採用）

- 概要: Grafana Labs の統合観測性スタック
- メリット:
  - 3 本柱が同一 UI（Grafana）で見られる、運用者の学習一元化
  - Kubernetes ネイティブ、Helm Chart 成熟
  - 水平スケール可能（Mimir は TB 級対応）
  - オブジェクトストレージ（MinIO、ADR-DATA-003）をバックエンドにできる
  - OpenTelemetry ネイティブ
- デメリット:
  - AGPL 対応が必要（ADR-0003 準拠）
  - Mimir/Tempo/Loki それぞれの運用知識が必要
  - コンポーネント数が多い（4 種類）

### 選択肢 B: ELK Stack (Elasticsearch + Logstash + Kibana)

- 概要: ログ分析で業界実績
- メリット: Elasticsearch の検索性能
- デメリット:
  - メトリクス・トレースは別スタック
  - Elasticsearch のライセンス変更（SSPL）、AGPL 類似の制約
  - 運用コスト（JVM、インデックス管理）が重い

### 選択肢 C: Prometheus + Jaeger + Kibana 個別構成

- 概要: 各専用 OSS を組合せ
- メリット: コンポーネント選択の柔軟性
- デメリット:
  - UI が分散、運用者の学習コスト増
  - 統合ダッシュボードが自力構築
  - 長期メトリクス保管は別途（Thanos / Cortex / Mimir）

### 選択肢 D: Datadog / New Relic / Splunk

- 概要: 商用 SaaS
- メリット: ベンダーサポート、UX 洗練
- デメリット:
  - オンプレ制約で SaaS は選択肢外
  - 年間コスト数千万〜億円規模
  - データを社外に送る、データ主権と整合しない

### 選択肢 E: SigNoz

- 概要: OpenTelemetry ネイティブな観測性 OSS（MIT）
- メリット: AGPL 回避、シンプル
- デメリット:
  - 採用実績がまだ薄い
  - 3 本柱統合の成熟度が LGTM より低い

## 帰結

### ポジティブな帰結

- Metrics / Logs / Traces が Grafana 単一 UI で確認可能、運用者の習熟コスト最小化
- MinIO をバックエンドに使えてストレージコスト最適化
- OpenTelemetry ネイティブで tier1 Log/Telemetry API と自然統合
- 水平スケール可能で採用側のマルチクラスタ展開でも対応

### ネガティブな帰結

- AGPL 対応の運用証跡維持（半期監査で説明必須）
- 4 コンポーネントの運用工数、Operator / Helm Chart 別管理
- Mimir のテナント機能を活用するか、単一 DB で運用するかの設計判断
- Grafana のカスタムプラグイン禁止により、プラグインが必要な要件（業務特化ダッシュボード）は別途検討

## 実装タスク

- Loki / Tempo / Mimir の Helm Chart バージョン固定、Argo CD 管理
- バックエンドストレージを MinIO に統一、バケット設計
- Grafana の SSO 統合（Keycloak、ADR-SEC-001）
- 標準ダッシュボード（tier1 API SLO、インフラ健全性、Kafka ラグ等）を初期提供
- アラートルール（Prometheus AlertManager 互換）を GitOps で管理
- AGPL 分離アーキテクチャ図を 02_構想設計/05_法務とコンプライアンス/ に配置
- 長期保管ポリシー（metrics 13 ヶ月、logs 1 年、traces 14 日）を Lifecycle 設定

## 関連 ADR

- ADR-TEST-006（観測性 E2E）— Loki / Tempo / Mimir / Grafana の機能継続検証を 5 検証（trace 貫通 / cardinality regression / log↔trace 結合 / SLO alert 発火 / dashboard goldenfile）として実装

## 参考文献

- Grafana Labs LGTM Stack: grafana.com/oss
- Prometheus 仕様
- OpenTelemetry 仕様
- AGPL-3.0 本文
