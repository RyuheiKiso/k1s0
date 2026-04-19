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

**現状**: tier2 が OpenTelemetry SDK を直接使うと、span 生成、親子関係、attributes 付与のボイラープレートが多い。他のサービス呼び出しで context 伝搬が手動。

**要件達成後**: `k1s0.Telemetry.StartSpan("operation")` で span を開始、`with` / `using` / `defer` で自動終了。親 span は context から自動取得。他 tier1 API 呼び出しでは子 span が自動生成される。

**崩れた時**: span の親子関係が崩れ、Tempo で「このリクエストはどのサービスを経由したか」を追えなくなる。

**受け入れ基準**:
- Go / C# / Rust / Python SDK で同名 API
- span の自動親子関係（context 伝搬）
- attribute 上限（1 span あたり 128）を超えると警告
- Tempo で traceId を引くとリクエスト全体のツリーが可視化

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
- 優先度 SHOULD（Phase 1b で評価）

## 入出力仕様

```
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

## Phase 対応

- **Phase 1a**: FR-T1-TELEMETRY-001、002、003（Go SDK、Collector）
- **Phase 1b**: FR-T1-TELEMETRY-004（Pyroscope）、C# SDK
- **Phase 1c**: Python / Rust SDK
- **Phase 2+**: eBPF ベース自動計装、SLO ダッシュボード自動生成

## 関連非機能要件

- **NFR-C-MON-001**: 監視基盤（Prometheus / Tempo / Loki / Pyroscope）の構成要件
- **NFR-B-PERF-006**: 計装オーバヘッド < 10ms
- **NFR-A-CONT-005**: Collector 障害時の tier2 稼働継続
- **NFR-E-MON-001**: 計装データへの tenant_id 必須付与
