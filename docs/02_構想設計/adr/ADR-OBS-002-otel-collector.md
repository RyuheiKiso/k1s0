# ADR-OBS-002: テレメトリ収集に OpenTelemetry Collector を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / 運用チーム

## コンテキスト

tier1 Log API / Telemetry API はアプリからメトリクス・ログ・トレースを受け取り、バックエンド（Loki / Mimir / Tempo、ADR-OBS-001）に配送する必要がある。アプリ側で各バックエンドの独自プロトコルを実装するのは、1）アプリ側コードが肥大化、2）バックエンド変更時にアプリを変更する必要、3）観測データの加工・サンプリング・ルーティングをアプリごとに実装する非効率、の 3 つの問題を抱える。

OpenTelemetry（CNCF Incubating → Graduated 進行中）は、メトリクス・ログ・トレースの統一プロトコル（OTLP）と Collector を提供し、この問題を業界標準として解決している。

## 決定

**テレメトリ収集・配送は OpenTelemetry Collector（CNCF、Apache 2.0）を採用する。**

- OpenTelemetry Collector（contrib 版）を DaemonSet で各 Node に配置（Agent モード）
- 加えて StatefulSet として Gateway モードの Collector を集中配置
- Agent → Gateway → Backend（Loki/Mimir/Tempo）の 2 段構成
- アプリ側は OTLP（gRPC）で Agent に送信
- Gateway で Processor を適用（PII マスキング、サンプリング、属性正規化、テナント属性付与）
- メトリクス: OTLP → Mimir（Prometheus Remote Write 経由）
- ログ: OTLP → Loki（Loki Exporter）
- トレース: OTLP → Tempo
- tier1 Log API / Telemetry API の内部実装は OTel SDK でラップ、tier2/tier3 には k1s0 API で抽象化

## 検討した選択肢

### 選択肢 A: OpenTelemetry Collector（採用）

- 概要: CNCF プロジェクト、業界標準
- メリット:
  - ベンダー中立、将来のバックエンド変更に対応
  - Processor による加工・フィルタリングが豊富（PII マスキング、サンプリング、ルーティング）
  - Multiple Exporter でバックエンド分散が容易
  - OTLP（OpenTelemetry Protocol）が言語 SDK で標準サポート
- デメリット:
  - Collector 自体の運用（HA、スケール）が必要
  - Contrib 版は多数の Receiver/Processor/Exporter を含み、バージョンアップでの互換性注意

### 選択肢 B: Fluent Bit + Prometheus Agent + Tempo Agent

- 概要: ログ・メトリクス・トレースそれぞれ専用 Agent
- メリット: 個別最適化
- デメリット:
  - Agent 3 種類運用、設定分散
  - 加工ロジックを共通化しにくい

### 選択肢 C: Grafana Alloy（旧 Grafana Agent）

- 概要: Grafana Labs の統合 Agent
- メリット: Grafana LGTM との親和性最大
- デメリット:
  - Grafana エコシステムへの依存深化
  - ベンダー中立性が OTel Collector より劣る

### 選択肢 D: アプリから直接バックエンドへ書込み

- 概要: Collector を置かない
- メリット: 構成シンプル
- デメリット:
  - バックエンド変更時にアプリ全改修
  - 加工・サンプリングをアプリごとに実装

## 帰結

### ポジティブな帰結

- ベンダー中立性でバックエンド変更可能性を維持
- PII マスキング（NFR-G-CLS-002）を Collector Gateway で一元適用
- サンプリング戦略を中央管理、アプリ変更不要でサンプリング率調整可能
- OpenTelemetry Semantic Conventions に則った属性命名統一

### ネガティブな帰結

- Collector 自体の HA 構成が必要（Agent / Gateway の 2 段）
- Contrib 版 Collector のバージョンアップ注意点（非互換変更 history を確認）
- Processor の設定ミスで観測データが失われるリスク、staging 検証必須

## 実装タスク

- OpenTelemetry Collector の Helm Chart バージョン固定、Argo CD 管理
- Agent DaemonSet と Gateway StatefulSet の構成、HA 化
- Processor 標準セット: PII マスキング、tenant_id 付与、tail sampling、attribute normalization
- tier1 Log / Telemetry API 内部実装で OTel SDK 使用、tier2/tier3 クライアント SDK にラップ
- Backstage テンプレートで tier2/tier3 アプリに OTel 自動計装を標準化
- サンプリング率を Feature Flag（ADR-FM-001）で動的調整

## 関連 ADR

- ADR-TEST-006（観測性 E2E）— OTel Collector が tier1→2→3 を貫通する trace_id を伝播していることを検証 1（trace propagation）で機械検証

## 参考文献

- OpenTelemetry 公式: opentelemetry.io
- OpenTelemetry Collector: github.com/open-telemetry/opentelemetry-collector
- OpenTelemetry Semantic Conventions
