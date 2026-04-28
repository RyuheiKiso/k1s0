# Telemetry API

本書は、tier1 が公開する Telemetry API の機能要件を定義する。tier2/tier3 の分散トレース・メトリクス・プロファイリングを、OpenTelemetry SDK のラッパとして提供する。

## API 概要

Metrics（Prometheus）、Traces（Grafana Tempo）、Profiling（Pyroscope）の 3 系統の計装 API を統一する。各言語 SDK は OpenTelemetry の上に k1s0 固有ラッパを被せ、自動計装と手動計装の両方を提供する。Collector は DaemonSet で配置し、tier2 からはエンドポイント構成を意識しない。

## 機能要件

### FR-T1-TELEMETRY-001: Metrics 計装 API

**現状**: tier2 が Prometheus クライアントライブラリを直接使うと、メトリクス命名、ラベル設計、counter / gauge / histogram の使い分けが属人的になる。tenant_id ラベルの付与忘れで、複数テナントのメトリクスが混ざる事故が発生する。

**要件達成後**: `k1s0.Telemetry.Counter("metric_name", labels)`、`Histogram(...)`、`Gauge(...)` で統一 API を提供する。tenant_id ラベルは SDK が自動付与。メトリクス名は `k1s0_<domain>_<metric>_<unit>` 形式を推奨（例: `k1s0_state_get_latency_seconds`）。

**崩れた時**: メトリクス命名・ラベル設計が部門ごとにバラつき、ダッシュボード再利用ができない。tenant_id 混在で監査対応にクエリコストがかかる。

**受け入れ基準**:

- counter / histogram / gauge の 3 タイプを提供
- tenant_id ラベルの自動付与
- メトリクス名・ラベルのバリデーション（予約語、cardinality 上限）

### FR-T1-TELEMETRY-002: Traces 計装 API

**業務根拠**: BR-PLATOPS-003（分散システムにおける障害原因特定時間の構造的短縮）。

**現状**: tier2 が OpenTelemetry SDK を直接使うと、span 生成、親子関係、attributes 付与のボイラープレートが多い。他のサービス呼び出しで context 伝搬が手動。社内既存マイクロサービスの障害事例 30 件を分析した結果、「複数サービス経由の不具合」は平均 4.2 時間の原因特定時間を要し、うち 60% が「どのサービスで問題が発生したかの切り分け」に費やされている。分散トレースが部分的にしか効いていない状態で、調査コストは障害 30 件 × 4.2h × 60% = 75.6 人時/件の切り分け工数が累積する。

**要件達成後**: `k1s0.Telemetry.StartSpan("operation")` で span を開始、`with` / `using` / `defer` で自動終了。親 span は context から自動取得。他 tier1 API 呼び出しでは子 span が自動生成される。リクエスト全体のサービス経路が Tempo 上で完全に可視化され、切り分け工数 60% が解消。業界ベンチマーク（CNCF Observability Survey 2024）では完全な分散トレース導入で MTTR が 30〜50% 短縮される。

**崩れた時**: span の親子関係が崩れ、Tempo で「このリクエストはどのサービスを経由したか」を追えなくなる。切り分け作業の属人化が続き、重大障害時は SRE / 各チーム合同のウォールーム対応が長時間化する。

**動作要件**:

- Go / C# / Rust / Python SDK で同名 API
- span の自動親子関係（context 伝搬）
- attribute 上限（1 span あたり 128）を超えると警告
- Tempo で traceId を引くとリクエスト全体のツリーが可視化

**品質基準**:

- span 生成オーバヘッドは NFR-B-PERF-006（10ms 以下）に従う
- W3C Trace Context 伝搬の完全性を契約テストで検証（tier1 API 経由の span 切断ゼロ）

### FR-T1-TELEMETRY-003: OpenTelemetry Collector 経由配信

**現状**: tier2 アプリが直接 Prometheus / Tempo / Loki にメトリクス・トレース・ログを送ると、バックエンドの切替で全アプリ改修が必要になる。ネットワーク経路が直結で、アクセス制御が煩雑。

**要件達成後**: 全ての計装データは OpenTelemetry Collector（DaemonSet）に送信され、Collector がバックエンドに転送する。バックエンド切替は Collector 設定変更のみで済む。tier2 アプリは Collector のエンドポイント URL を環境変数経由で取得。

**崩れた時**: バックエンド切替（例: Loki → 別ログ基盤）の都度 tier2 全アプリ改修が発生し、数人月の工数ロスが発生する。

**受け入れ基準**:

- Collector は DaemonSet で全 Node に配置
- tier2 SDK は localhost:4318（OTLP）に送信
- Collector の batch / retry / filtering 設定で tier2 負荷を抑制

### FR-T1-TELEMETRY-004: Profiling（Pyroscope）連携

**現状**: tier2 のパフォーマンス問題調査（CPU 使用率 / メモリ消費）は、ログ・トレースから推測するしかなく、根本原因のコード箇所特定に時間がかかる。

**要件達成後**: Pyroscope の Continuous Profiling を tier2 アプリで有効化し、Tempo の Traces-to-Profiles 連携で、特定 span の実行時プロファイルをコード行レベルで参照可能にする。

**崩れた時**: パフォーマンス問題の根本原因特定が人手の勘頼りとなり、修正までのリードタイムが長期化する。

**受け入れ基準**:

- Go / C# / Rust / Python の各 Pyroscope エージェントを SDK に同梱
- Tempo の span から Profiling へのジャンプが可能
- 優先度 SHOULD（リリース時点 で評価）

## 入出力仕様

本 API の機械可読な契約骨格（Protobuf IDL）は [40_tier1_API契約IDL/08_Telemetry_API.md](../40_tier1_API契約IDL/08_Telemetry_API.md) に定義されている。SDK 生成・契約テストは IDL 側を正とする。以下は SDK 利用者向けの疑似インタフェースであり、IDL の `TelemetryService` RPC と意味論的に対応する（SDK 内で OpenTelemetry SDK に橋渡しし、OTLP で IDL に準拠した送信を行う）。

```text
// Metrics
k1s0.Telemetry.Counter(name: string, labels?: map<string, string>) -> Counter
k1s0.Telemetry.Histogram(name: string, buckets?: float[], labels?) -> Histogram
k1s0.Telemetry.Gauge(name: string, labels?: map<string, string>) -> Gauge

counter.Inc(value: float, labels?)
histogram.Observe(value: float, labels?)
gauge.Set(value: float, labels?)

// Traces
span = k1s0.Telemetry.StartSpan(name: string, options?: {
    parent?: Context,
    attributes?: map<string, any>
})
span.SetAttribute(key: string, value: any)
span.AddEvent(name: string, attributes?)
span.RecordError(error: any)
span.End()

// Profiling
k1s0.Telemetry.StartProfiling(profile_name: string) -> ProfileSession
profileSession.Stop()
```

## 受け入れ基準（全要件共通）

- 計装データの送信が tier2 業務処理 p99 に 10ms 以上影響しない
- Collector 障害時は tier2 でバッファリング（一定時間内）、あふれたら破棄
- tenant_id 必須付与（NFR-E-MON-001 連携）
- cardinality 上限 1,000 ラベル値 / メトリクス（Prometheus の性能保護）

## 段階対応

- **リリース時点**: FR-T1-TELEMETRY-001、002、003（Go SDK、Collector）
- **リリース時点**: FR-T1-TELEMETRY-004（Pyroscope）、C# SDK
- **リリース時点**: Python / Rust SDK
- **採用後の運用拡大時**: eBPF ベース自動計装、SLO ダッシュボード自動生成

## 関連非機能要件

- **NFR-C-NOP-001**: 監視基盤（Prometheus / Tempo / Loki / Pyroscope）の構成要件
- **NFR-B-PERF-006**: 計装オーバヘッド < 10ms
- **NFR-A-CONT-005**: Collector 障害時の tier2 稼働継続
- **NFR-E-MON-001**: 計装データへの tenant_id 必須付与
