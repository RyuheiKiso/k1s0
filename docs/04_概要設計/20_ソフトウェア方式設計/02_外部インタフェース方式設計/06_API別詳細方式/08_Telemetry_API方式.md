# 08. Telemetry API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応し、tier1 が tier2 / tier3 へ公開する `k1s0.public.telemetry.v1.Telemetry` サービスの外部インタフェース詳細を固定化する。共通契約（認証 / トレース / テナント伝搬 / 冪等 / エラー / レスポンスヘッダ）は [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001〜016 を参照とし、本ファイルは Telemetry API 固有の責務・メソッド・属性スキーマ・サンプリング方針・バックエンド振り分けを扱う。

## 本ファイルの位置付け

Telemetry API は業務ロジックから見ると「プラットフォームが与える観測性抽象」の一翼を担う。tier1 が OTLP（OpenTelemetry Protocol）互換の受信口を公開し、その先で Tempo（トレース）/ Prometheus（メトリクス）/ Pyroscope（プロファイル）に振り分ける構成は [ADR-OBS-001] で固定した。tier2 / tier3 は OpenTelemetry 公式 SDK を使う限り、バックエンドの差し替えを意識せず計装でき、tier1 側の OTel Collector で集約・前処理・サンプリングが一元化される。

Log API が「人間可読の事象列」を担うのに対し、Telemetry API は「機械可読の分散計測値」を担う。両者は trace_id で相関し、Grafana では同一 trace_id でログ / トレース / メトリクスを横串検索できる状態を合格基準とする。本ファイルは OTLP 互換性を損なわない範囲で tier1 固有の契約（テナント属性必須 / サンプリング方針 / サイズ上限）を加え、観測性プラットフォームのコスト爆発を抑える設計を固定する。

## サービス定義と公開メソッド

Telemetry API は Protobuf サービス `k1s0.public.telemetry.v1.Telemetry` として定義し、以下 4 メソッドを公開する。メソッド粒度は OTLP の 3 シグナル（Traces / Metrics / Logs → 本 API では Profiles）に対応し、tier1 独自の Stream モードを補助的に追加する。

**設計項目 DS-SW-EIF-360 Telemetry サービスのメソッド粒度と OTLP 互換性**

4 メソッドは `SubmitTraces` / `SubmitMetrics` / `SubmitProfiles` / `SubmitStream` で構成する。前 3 つは OTLP の `opentelemetry.proto.collector.<signal>.v1.Export<Signal>ServiceRequest` を内包 (oneof ではなく wrap) しており、OTel 公式 SDK のエクスポータを `https://api.k1s0.internal.example.jp/grpc/k1s0.public.telemetry.v1.Telemetry/SubmitTraces` に向けるだけで動作する互換モードを成立させる。`SubmitStream` はサーバサイドストリーミングで、長時間の計装タスク（ベンチマーク中の連続メトリクス送信）を効率化するための tier1 独自拡張である。OTLP 非互換の拡張を独立メソッドに切り出すことで、OTel 公式 SDK 経由の流入を純正 OTLP として扱える。

**設計項目 DS-SW-EIF-361 OTel SDK 経由での自動互換モード**

tier2 / tier3 の推奨経路は OTel 公式 SDK（Go / Rust / TypeScript / .NET）の OTLP Exporter を tier1 エンドポイントに向ける構成である。この場合、アプリ側の計装コードは `tracer.Start(ctx, "span_name")` / `meter.Counter("metric_name").Add(ctx, 1)` のみで完結し、tier1 固有のメタデータ（tenant_id / trace_id 伝搬）は SDK 付属の Resource Detector と Baggage Propagator が自動投入する。手動で Telemetry API を呼び出す経路は計装ライブラリ自作時や非 OTel 互換言語（Phase 2 想定の COBOL ブリッジ等）でのみ使用し、アプリ開発者向けの第一選択肢は SDK 経由に統一する。

## Resource 属性と必須フィールド

OTLP Resource は「どのサービスのどのインスタンスが送信したか」を識別する属性セットであり、tier1 側の集計・課金・テナント分離の全てがこの属性群に依存する。OTLP 仕様上 Resource は任意属性だが、本 API では 4 属性を必須化する。

**設計項目 DS-SW-EIF-362 Resource 属性の必須化**

必須 Resource 属性は `service.name` / `service.version` / `deployment.environment` / `k1s0.tenant.id` の 4 つである。前 3 つは OpenTelemetry Semantic Conventions 1.28 準拠、最後の `k1s0.tenant.id` は tier1 拡張属性で `k1s0_tenant_id` の snake_case も互換として許容する。欠落時は `INVALID_RESOURCE` エラーを返し、OTel Collector の processor stage で reject する。Phase 1a は `service.name` のみ必須で残り 3 属性を warning、Phase 1b から 4 属性全必須に昇格させ、Phase 1a 期間中に tier2 / tier3 側で計装を整備する時間を確保する。

| 属性名 | 型 | 必須 | 根拠 |
|--------|----|------|------|
| service.name | string | 必須 | 集計単位の最小粒度、欠落時は `unknown_service` に落ちコスト分析不能 |
| service.version | string（SemVer） | 必須 | デプロイ切り替え時の比較、インシデント切り分けに不可欠 |
| deployment.environment | string（dev/pre/prod） | 必須 | 本番メトリクスと開発メトリクスの混入を防ぐ |
| k1s0.tenant.id | string（UUID） | 必須 | テナント別課金・RLS 相当の集計分離、法務 SLA 根拠 |
| service.instance.id | string | 推奨 | Pod 単位障害の追跡、欠落でも集計は可能 |
| host.name | string | 推奨 | Node 単位の相関、K8s DownwardAPI で自動投入可能 |

## トレース属性と ECS 対応

トレースのスパン属性は Grafana Tempo 側で検索インデックスを張る対象となり、属性命名の不統一は検索不能と直結する。tier1 は OTel Semantic Conventions を正として採用し、社内 ECS フィールドとの対応表を合わせて運用する。

**設計項目 DS-SW-EIF-363 スパン属性の Semantic Conventions 強制**

HTTP / gRPC / DB のスパン属性は OTel Semantic Conventions 1.28 準拠を強制する。`http.request.method` / `http.response.status_code` / `rpc.system` / `rpc.service` / `rpc.method` / `db.system` / `db.namespace` を必須とし、旧名（`http.method`、`http.status_code` 等）は Collector の processor stage で新名へ自動変換する。trace_id / span_id / parent_span_id は W3C Trace Context 準拠 16/8/8 byte で、Log API と Audit-Pii API に同一 trace_id で相関する。社内 ECS（Elastic Common Schema）マッピング（例: `http.request.method` → `http.request.method` 同名、`rpc.service` → `service.target.name`）は [../../30_共通機能方式設計/11_可観測性方式.md](../../30_共通機能方式設計/) に 40 行の対応表として別掲する。

## メトリクス型と OpenMetrics exposition

メトリクスは OTLP Metric Data Point の 5 型（Sum / Gauge / Histogram / ExponentialHistogram / Summary）を受け付け、Prometheus 側の OpenMetrics exposition format に変換して永続化する。ヒストグラムの bucket 戦略は SLO 計測の精度を直接左右するため方式設計で固定する。

**設計項目 DS-SW-EIF-364 メトリクス型とヒストグラム bucket 方針**

受け入れ型は Counter（OTLP Sum monotonic）/ UpDownCounter（Sum non-monotonic）/ Gauge / Histogram / ExponentialHistogram / Summary の 6 型。推奨は ExponentialHistogram で、p50/p95/p99 を低コストで算出できる。Prometheus 変換時は `_bucket` / `_sum` / `_count` の 3 メトリクスに展開する。Histogram の固定 bucket は `[1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000]` ms を tier1 推奨値とし、API レイテンシ SLO（p99 500ms）を含む 1ms〜10s のダイナミックレンジを 12 bucket でカバーする。テナント別集計は exemplar（高次元 trace_id + tenant_id）として添付し、集計クエリ側で分離する。

## プロファイル（pprof 互換）

プロファイル情報はトレース・メトリクスでは捕捉できない CPU / Heap / Goroutine の詳細を継続取得するため、Pyroscope と OTel Profiling Signal（2026 Q1 GA 予定）で構成する。Phase 1b 時点ではまだ OTel Profiling が GA していないため Pyroscope 独自プロトコルを内部で採用する。

**設計項目 DS-SW-EIF-365 Profiles は pprof 互換と Pyroscope 連携**

`SubmitProfiles` は pprof 互換のバイナリ形式（`profile.proto`、gzip 圧縮）を受け付け、OTel Collector の Pyroscope exporter 経由で Pyroscope サーバへ保存する。プロファイル種類は `cpu` / `alloc_space` / `inuse_space` / `goroutine` / `mutex` / `block` の 6 種、tier1 クライアント SDK はデフォルトで cpu と alloc_space を 10 分間隔で取得する。OTel Profiling Signal が GA 後（Phase 2）に互換レイヤを追加し、アプリ側コード変更なしで移行する方針は [ADR-OBS-002] に記録する。Phase 1a では提供せず、Phase 1b から限定公開する。

## サンプリング方針

フル計装は観測性コスト（Tempo / Prometheus のストレージ）を破綻させる。本番トラフィックでは代表性を保ちつつストレージを削減するサンプリング設計が SLO 遵守と同等に重要である。

**設計項目 DS-SW-EIF-366 head + tail の 2 段サンプリング**

本番環境はクライアント SDK 側で head sampling 10%（`TraceIdRatioBased(0.1)`）、tier1 OTel Collector 側で tail sampling をエラースパン / 遅延スパン（> p95 × 2）/ 重要エンドポイントに対して 100% 維持する。head 10% の根拠は、Grafana Tempo の sizing guide で 1,000 spans/sec × 1KB × 30 日 = 2.6TB ストレージを起点に、JTC での許容予算 300GB/月 × 30% 余裕から逆算した値である。Pre / Dev は 100% サンプリングとし開発時の計装品質を担保する。メトリクスはサンプリングせず全量集計、プロファイルは 10 分間隔の時間サンプリングとシグナルごとに方針を分離する。

**設計項目 DS-SW-EIF-367 tail sampling ルールと tier1 側 Collector 実装**

tail sampling は OTel Collector の `tail_sampling` processor で実装する。ルールは `(1) status_code = ERROR のスパンは 100% 保持`、`(2) root span の duration > 500ms は 100% 保持（SLO 超過）`、`(3) HTTP route が /payments / /checkout / /auth のスパンは 100% 保持（ビジネスクリティカル）`、`(4) それ以外は head sampling 結果に従う` の 4 段評価である。tail 判定のための決定遅延は 5 秒でバッファリングし、完全なトレースツリーを評価対象にする。判定遅延分のメモリ消費（1,000 trace/sec × 5s × 10KB = 50MB）は Collector Pod のリソース計画 [../../50_非機能方式設計/06_キャパシティ計画.md](../../50_非機能方式設計/) に組み込む。

## 非同期エンキューと本線性能

Telemetry API の p99 は 5ms（DS-SW-EIF-013）。これを満たすため、受信から永続化までの経路を非同期化し、本線は受信バッファへのエンキューで完結させる。

**設計項目 DS-SW-EIF-368 受信 → エンキュー → 永続化の分離**

`SubmitTraces` / `SubmitMetrics` は受信後に tier1 の Go 実装 facade-telemetry pod 内 in-memory ring buffer（デフォルト 10,000 件）へエンキューし、即座に OK を返す。バックグラウンドの goroutine プールが OTel Collector へ gRPC バッチ送信（50ms / 1,000 件のどちらか先）する。受信時点での QoS は at-most-once（リングバッファ溢れ時はドロップしメトリクス `telemetry_dropped_total` でカウント）とし、トレース欠落許容度は tail sampling で補正する方針である。受信 p99 5ms の測定点は「`SubmitTraces` 受信 → OK 応答」であり、永続化完了時刻ではない点を SLO 文書で明示する。

## バックエンド振り分け

tier1 は OTel Collector を単一エッジとして受け、そこから Tempo / Prometheus / Pyroscope の 3 バックエンドへ分岐する。この分岐は OTel Collector のパイプライン定義（`receivers` / `processors` / `exporters`）で宣言的に行い、tier1 公開 API 側にバックエンド固有のロジックを漏らさない。

**設計項目 DS-SW-EIF-369 OTel Collector パイプライン構成**

受信は `otlp` receiver（gRPC 4317 / HTTP 4318）、処理は `memory_limiter` → `batch` → `attributes`（tenant_id 必須化）→ `tail_sampling`（トレースのみ）→ `resource` の順、送信は `otlp/tempo`（トレース）/ `prometheusremotewrite`（メトリクス）/ `pyroscope`（プロファイル）の 3 exporter に fan-out する。Collector 自体は StatefulSet として 3 レプリカ構成し、gateway 役（アプリ側 SDK → Collector）と collector 役（Collector → バックエンド）の 2 段構えで耐障害性を確保する。Phase 1a は Tempo + Prometheus のみ、Phase 1b で Pyroscope を追加、段階的に信号タイプを拡張する。

## サイズ上限と QoS

OTLP は仕様上メッセージサイズの上限を定めないが、無制限受信は Collector の OOM を招く。tier1 では受信サイズ上限を明示的に設定し、超過時のふるまいを固定する。

**設計項目 DS-SW-EIF-370 バッチサイズ上限とフロー制御**

1 RPC のバッチサイズ上限は 4MB（gRPC max_receive_message_length）とする。この値は OTel Collector 公式推奨の 4MB、Tempo の ingester 1 request 上限 4MB と整合させた結果である。超過時は gRPC `RESOURCE_EXHAUSTED` / `INVALID_ARGUMENT` を返し、クライアント SDK は自動でバッチを半分に分割して再送する。クライアント SDK のデフォルトバッチは 512KB / 512 spans / 10s のいずれか先で flush するため、通常運用で 4MB 超過は発生しない。1 スパン属性数は 128 個、1 属性値サイズは 4KB に Collector 側で制限し、異常計装によるカーディナリティ爆発を防ぐ。

**設計項目 DS-SW-EIF-371 テナント別クォータとレート制限**

テナント別に 1,000 spans/sec / 10,000 data points/sec / 1 MB/sec のレート制限を Envoy Gateway の rate-limit filter で設定する。超過時は gRPC `RESOURCE_EXHAUSTED` + `X-K1s0-RateLimit-Reset` ヘッダを返す。このクォータは [../../../03_要件定義/30_非機能要件/](../../../03_要件定義/30_非機能要件/) の NFR-B-CAP-003（テナント別キャパシティ公平性）に対応し、1 テナントの計装暴走が他テナントの観測性を阻害する事態を防ぐ。超過時のエラーコードは `QUOTA_EXCEEDED`（K1s0Error の詳細 detail）として標準化する。

## エラーコード

Telemetry API 固有のエラーは K1s0Error の `code` フィールドに以下 5 種を登録する。共通エラー（`AUTH_*` / `INTERNAL_*`）は [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) DS-SW-EIF-006 を参照する。

**設計項目 DS-SW-EIF-372 Telemetry 固有エラーコード**

| コード | gRPC status | 発生条件 | 根拠 |
|--------|-------------|----------|------|
| `TELEMETRY_QUOTA_EXCEEDED` | RESOURCE_EXHAUSTED | テナント別レート上限超過 | DS-SW-EIF-371 クォータ |
| `TELEMETRY_INVALID_RESOURCE` | INVALID_ARGUMENT | 必須 Resource 属性欠落 | DS-SW-EIF-362 必須化 |
| `TELEMETRY_SCHEMA_MISMATCH` | INVALID_ARGUMENT | OTLP スキーマバージョン非対応 | 互換性維持のため v1 固定 |
| `TELEMETRY_BATCH_TOO_LARGE` | RESOURCE_EXHAUSTED | 1 RPC > 4MB | DS-SW-EIF-370 上限 |
| `TELEMETRY_BACKEND_UNAVAILABLE` | UNAVAILABLE | Collector → バックエンド送信失敗が閾値超過 | アプリ側リトライ判断用 |

## テナント別集計とコスト配賦

tier1 は将来のテナント別課金（NFR-F-COST-002）に備え、Resource 属性 `k1s0.tenant.id` を全データポイントの exemplar に含め、集計クエリ側で分離可能にする。これは属性を metric label に昇格させるとカーディナリティが爆発するため、exemplar（高次元トレース情報） 経由で集計する工夫である。

**設計項目 DS-SW-EIF-373 テナント ID を exemplar に限定する**

tenant_id はスパン属性としては保持するが、Prometheus メトリクスのラベルには昇格させない。代わりに Prometheus の exemplar（2021 年導入）に tenant_id を添付し、Grafana の exemplar 表示から個別トレースを辿る経路を提供する。これにより Prometheus の時系列カーディナリティ爆発（テナント 1,000 × メトリクス 100 = 100,000 系列）を回避しつつ、特定テナントの挙動追跡が可能になる。テナント別集計が必要な場合は Tempo の `{resource.k1s0.tenant.id="<id>"}` での TraceQL クエリを第一選択とし、Prometheus での集計は四半期バッチ（Postgres 集計テーブル）で代替する。

## フェーズ別提供範囲

Phase 別に提供シグナルと機能を段階公開する。観測性基盤の運用成熟度（SRE チームの人員 / アラート設計 / ダッシュボード整備）に追従させるためである。

**設計項目 DS-SW-EIF-374 フェーズ別提供範囲**

Phase 1a（MVP-0）: `SubmitTraces` / `SubmitMetrics` のみ、サンプリング head 10% 固定、tail sampling 無効、テナント exemplar 無効。Tempo / Prometheus 構成。Phase 1b（MVP-1a）: `SubmitProfiles` 追加、tail sampling 有効化、exemplar 有効化、Pyroscope 追加。Phase 1c（MVP-1b）: `SubmitStream` 有効化、OTel Profiling Signal 互換層の試験運用。Phase 2: OTel Profiling Signal の本番切り替え、マルチリージョン Tempo / Mimir への移行検討（ADR-OBS-003 として別途起票）。

## 対応要件一覧

本ファイルは Telemetry API 公開インタフェースの詳細方式設計であり、以下の要件 ID に対応する。

- FR-T1-TELEMETRY-001〜FR-T1-TELEMETRY-004（Telemetry API 機能要件、Metrics / Traces / OTel Collector / Profiles）
- FR-T1-TELEMETRY-001（OTLP 互換 Metrics 計装 API）/ FR-T1-TELEMETRY-002（Traces 計装 API）/ FR-T1-TELEMETRY-003（OpenTelemetry Collector 経由配信）/ FR-T1-TELEMETRY-004（Profiles / Pyroscope 連携）
- FR-EXT-MON-001（監視基盤連携：Grafana Tempo / Prometheus / Pyroscope）
- NFR-B-PERF-001（Telemetry p99 5ms 非同期エンキュー）
- NFR-B-CAP-003（テナント別キャパシティ公平性）
- NFR-C-OBS-001〜003（可観測性要件：分散トレース / メトリクス / プロファイル）
- NFR-F-COST-002（テナント別コスト配賦）
- ADR 参照: ADR-TIER1-001（Go+Rust 分担、Telemetry は Go）/ ADR-TIER1-002（Protobuf gRPC 必須）/ ADR-OBS-001（OTel Collector 集約）/ ADR-OBS-002（Pyroscope 採用）
- 共通契約: DS-SW-EIF-001〜016（[../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md)）
- 本ファイルで採番: DS-SW-EIF-360 〜 DS-SW-EIF-374
