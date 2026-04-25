# 03. 自作 Rust 領域コンポーネント

本ファイルは IPA 共通フレーム 2013 の **ソフトウェア方式設計プロセス 7.1.2.1（ソフトウェア構造とコンポーネントの方式設計）** に対応する。tier1 自作 Rust 領域を構成する 3 コンポーネント（COMP-T1-AUDIT / COMP-T1-DECISION / COMP-T1-PII）の内部モジュール構成、ZEN Engine 統合方式、ハッシュチェーン生成方式、PII 検出パーサとマスキング暗号、tonic gRPC サーバ実装、なぜ Dapr sidecar 不要かを方式として固定化する。

## 本ファイルの位置付け

[02_Daprファサード層コンポーネント.md](02_Daprファサード層コンポーネント.md) が Go ファサード 3 Pod の内部を仕様化したのに対し、本ファイルは自作 Rust 領域 3 Pod の内部を掘り下げる。自作領域の選定根拠は構想設計 ADR-TIER1-001（Dapr Go SDK stable + 自作は Rust）に確定しており、性能（ZEN の p99 1ms）・安全性（PII parser の memory safety）・改ざん防止（Audit hash chain の crypto）の 3 軸で Rust が選ばれている。本ファイルはこの選定を実装構造として落とし込み、tonic gRPC サーバ・ZEN Engine 統合・ハッシュチェーン生成・PII パーサの具体方式を固定化する。

自作領域は Dapr の Building Block に依存しないため、Pod 構成が Dapr sidecar を要さない点が最大の差異である。この差異は、Pod 起動時間・メモリフットプリント・レイテンシ内訳のすべてに効くため、ファサード層と一緒くたに扱うと設計意図が埋もれる。本ファイルはその差異を明示して残す。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-SW-COMP-050` 〜 `DS-SW-COMP-079` の 30 件である。通番は [02_Daprファサード層コンポーネント.md](02_Daprファサード層コンポーネント.md) の DS-SW-COMP-049 から連続し、[04_コンポーネント責務一覧.md](04_コンポーネント責務一覧.md) で DS-SW-COMP-080 に継続する。

## 自作層の共通構造

3 コンポーネントはいずれも Rust edition 2024 で記述し、gRPC サーバは `tonic` + `prost`、非同期ランタイムは `tokio`（multi-thread scheduler）、ログは `tracing` + `tracing-opentelemetry`、メトリックは `prometheus` + `axum` exporter を共通基盤とする。ファサード層の 5 モジュール構成とは異なり、自作層は責務がドメイン固有（hash chain / ZEN / PII parser）のため、共通モジュールは `k1s0-common` / `k1s0-proto` / `k1s0-otel` の 3 crate に限定し、残りは各 Pod 固有の実装を持つ。

### DS-SW-COMP-050 Rust 自作層の共通基盤

全自作 Pod は `tonic 0.12+` を gRPC サーバとして、`prost 0.13+` を Protobuf シリアライザとして使用する。非同期ランタイムは `tokio 1.x`（multi-thread、worker_threads = CPU コア数）で固定し、`#[tokio::main(flavor = "multi_thread")]` を `main.rs` で指定する。Rust Edition は 2024 で統一し、MSRV（Minimum Supported Rust Version）は 1.85 以上とする。依存管理は Cargo workspace（[06_パッケージ構成_Rust_Go.md](06_パッケージ構成_Rust_Go.md) 参照）で一元化し、crate 間のバージョン不整合を避ける。

**確定段階**: リリース時点（基盤 / 本格導入）。**対応要件**: NFR-B-PERF-004、NFR-E-ENC-001、DX-TEST-\*。**参照**: 構想設計 ADR-TIER1-001。

### DS-SW-COMP-051 tonic gRPC サーバ実装方式

tonic サーバは `Server::builder()` で構築し、`http2_keepalive_interval = 30s` / `http2_keepalive_timeout = 10s` / `tcp_keepalive = 60s` を既定とする。TLS は Istio Ambient の waypoint で終端するため Pod 側では平文 HTTP/2 で listen する。Interceptor は `tonic::service::interceptor_fn` で tracing span / metric / auth の 3 段を登録する。サーバシャットダウンは `serve_with_shutdown()` で `tokio::signal::unix::SIGTERM` を受けて graceful shutdown し、インフライトリクエストの完了待ちは最大 30 秒とする。シャットダウン順序は「受付停止 → 処理完了待機 → DB/接続クローズ」の順で、詳細は [../03_内部インタフェース方式設計/03_Go_Rust間言語境界方式.md](../03_内部インタフェース方式設計/03_Go_Rust間言語境界方式.md) で補足する。

**確定段階**: リリース時点。**対応要件**: NFR-A-FT-001、NFR-A-REC-001、NFR-E-NW-001〜004。

### DS-SW-COMP-052 tokio runtime チューニング

tokio runtime は multi-thread scheduler で worker_threads を CPU コア数に一致させる（Pod の resources.limits.cpu で制約）。blocking task（例: PostgreSQL 書込、CPU bound hash 計算）は `spawn_blocking` で専用スレッドプールに逃し、async task の latency を保護する。spawn_blocking のプールは既定 512 で、`max_blocking_threads` を resources から動的に設定する。task panic は `JoinError` で補足し、Pod 全体を crash させずに該当リクエストのみエラーレスポンスに変換する（詳細は [../../30_共通機能方式設計/](../../30_共通機能方式設計/) 参照）。

**確定段階**: リリース時点。**対応要件**: NFR-B-PERF-\*、NFR-A-FT-001。

### DS-SW-COMP-053 Dapr sidecar 不要の根拠

自作 Rust 3 Pod は Dapr サイドカー（daprd）を持たない。根拠は 3 つある。第一に、ZEN Engine / hash chain / PII parser のいずれも Dapr Building Block の抽象（state / pubsub / bindings / secrets / workflow / serviceinvocation）と 1 対 1 対応せず、Dapr 経由の呼び出しで性能・安全性が損なわれる。第二に、Pod 起動時間短縮（daprd 起動 500ms を削減）とメモリフットプリント削減（daprd 128MiB を削減）の効果が大きい。第三に、Pod 内プロセスを最小化することで failure mode を減らし 採用初期 の 採用側の小規模運用を容易にする。ネットワーク観測性は Istio Ambient の ztunnel + waypoint で代替する（[../../30_共通機能方式設計/](../../30_共通機能方式設計/) 参照）。

**確定段階**: リリース時点。**対応要件**: NFR-B-PERF-006、NFR-F-ENV-\*、NFR-C-NOP-001。**参照**: 構想設計 [../../../02_構想設計/02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md](../../../02_構想設計/02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md)。

## COMP-T1-AUDIT 内部モジュール詳細

COMP-T1-AUDIT は監査イベントのハッシュチェーン永続化に特化した Pod であり、crypto + WORM append + Kafka subscribe の 3 責務を内部で統合する。

### DS-SW-COMP-054 Audit Pod の内部モジュール構成

COMP-T1-AUDIT は 5 モジュールで構成する。`kafka_consumer` モジュールは Kafka topic `k1s0.audit.events.v1` を subscribe し、コンシューマグループ `k1s0-audit` で at-least-once 受信する。`dedup` モジュールは Valkey SET（`k1s0:audit:seen:{event_id}`、TTL 24h）で idempotent を担保する。`hash_chainer` モジュールは `previous_hash` → `event_hash` の SHA-256 chain を生成する。`pg_writer` モジュールは PostgreSQL に WORM append する。`minio_archiver` モジュールは四半期ごとに PG から MinIO Object Lock へ cold archive 転送する。各モジュールは Rust の async trait として interface を持ち、単体テストで個別モック化する。

**確定段階**: リリース時点（kafka + hash + pg / dedup + minio）。**対応要件**: FR-T1-AUDIT-\*、NFR-H-INT-001、NFR-C-NOP-003。

### DS-SW-COMP-055 ハッシュチェーン生成方式

ハッシュチェーンは `sha2::Sha256` で計算する。各イベントの `event_hash` は以下の入力を canonical CBOR（RFC 8949）でシリアライズして SHA-256 を取る: `previous_hash` / `event_id` / `tenant_id` / `actor` / `action` / `resource` / `timestamp` / `payload_hash`（payload 本体は別途 SHA-256 で先に hash 化）。`previous_hash` はテナント単位のチェーンで、前イベントの `event_hash` を参照する。テナント初回は Genesis hash（全 0x00 の 32 byte）を使う。CBOR を選ぶ理由は JSON の空白・順序揺れで同じ入力から異なる hash が出るリスクを排除するためである。

**確定段階**: リリース時点。**対応要件**: FR-T1-AUDIT-001、FR-INFO-AUDITMODEL-001、NFR-H-INT-001。**参照**: 構想設計 [../../../02_構想設計/02_tier1設計/](../../../02_構想設計/02_tier1設計/)。

### DS-SW-COMP-056 WORM append の実装方式

PostgreSQL の `audit_events` テーブルは `INSERT` のみ許可し、`UPDATE` / `DELETE` はトリガ（`tg_audit_readonly`）で pg_raise `E0005` を発生させて拒絶する。テーブルは tenant_id と month で range partitioning し、partition は pg_partman で自動作成する。replica 識別子（audit-0 / audit-1 / audit-2）は tenant_id の stable hash で決定され、各 replica は自担当 tenant のみ書き込む。書込失敗時は Kafka commit をしないことで at-least-once を維持し、dedup モジュールで再試行時の重複を弾く。

**確定段階**: リリース時点。**対応要件**: FR-T1-AUDIT-002、NFR-C-NOP-003、NFR-H-INT-001。**参照**: 構想設計 ADR-DATA-001（CloudNativePG）。

### DS-SW-COMP-057 replica パーティショニング方式

tenant_id は FNV-1a 64bit hash を 3 で剰余して replica index を決める（stable hash）。replica 0 は audit-0 に、1 は audit-1、2 は audit-2 に固定配置される。replica 数変更（3 → 5 など）はテナント hash 再割当を要し、移行中は旧 replica と新 replica の両方に書き込むデュアルライト期間（1 週間）を持つ。デュアルライト中の重複は dedup モジュールで弾く。replica あたりの平均負荷は tenant 数 × 平均 event 数 / 3 で見積もる（リリース時点 の中規模で 30 event/s → replica あたり 10 event/s）。

**確定段階**: リリース時点（初期配分）、採用後の運用拡大時（拡張時）。**対応要件**: NFR-B-WL-\*、NFR-A-FT-001。

### DS-SW-COMP-058 MinIO Object Lock 長期保管

四半期ごとに前四半期分の PG partition を MinIO Object Lock の Compliance モードにアーカイブ転送する。Retention は 7 年（NFR-C-NOP-003）で、Compliance モードは root user も削除不可のため改ざん耐性が最強である。転送は `minio_archiver` モジュールが cron（Kubernetes CronJob）で四半期初日 02:00 JST に起動し、前四半期の partition を `parquet + zstd` でシリアライズして MinIO にアップロードする。アップロード成功確認後に PG partition を `DETACH` + `DROP` する。drop 前に bucket 側の object 整合性を SHA-256 で再検証する。

**確定段階**: リリース時点。**対応要件**: FR-T1-AUDIT-003、NFR-C-NOP-003、NFR-H-INT-001。**参照**: 構想設計 ADR-DATA-003（MinIO Object Lock）。

### DS-SW-COMP-059 監査改ざん検証 API

COMP-T1-AUDIT は `VerifyChain(tenant_id, from_ts, to_ts)` gRPC を公開し、指定期間のハッシュチェーン整合性を検証する。検証は `previous_hash` を順次辿り、途中で不整合を検知した場合は不整合 event_id を返す。大規模期間（例: 3 か月分 30 万件）の検証は spawn_blocking で 100 event ずつ batch 処理し、progress を streaming gRPC で返す。検証は運用者 Runbook から起動され、監査対応時・障害調査時に利用する。

**確定段階**: リリース時点。**対応要件**: FR-T1-AUDIT-001、NFR-H-INT-001、DX-RB-\*。

### DS-SW-COMP-060 Kafka Consumer の at-least-once 設定

kafka_consumer は `rdkafka` crate の StreamConsumer で構成し、`enable.auto.commit=false` で手動 commit する。event を受信 → dedup 確認 → hash 計算 → pg_writer への書込成功 → Kafka offset commit の順で厳密化する。commit 失敗は再試行 3 回後に DLQ topic（`k1s0.audit.events.dlq.v1`）に積み、次回受信時に pg_writer が dedup で弾く。consumer group rebalance は static membership（`group.instance.id` を replica 名に固定）で StatefulSet Pod と整合する。

**確定段階**: リリース時点。**対応要件**: FR-T1-AUDIT-\*、NFR-A-CONT-003、NFR-H-INT-001。

## COMP-T1-DECISION 内部モジュール詳細

COMP-T1-DECISION は ZEN Engine による JDM 決定表評価に特化した Pod であり、p99 1ms を守るため全ロジックを in-process に収める。

### DS-SW-COMP-061 Decision Pod の内部モジュール構成

COMP-T1-DECISION は 4 モジュールで構成する。`rule_loader` モジュールはテナント別 JDM ルールセットを Control API（別エンドポイント `/admin/rules`）から受け取り、内部の LRU cache に格納する。`zen_engine` モジュールは `zen-engine` crate の評価器をラップし、ルールセット ID + 入力データから決定結果を返す。`cache` モジュールは LRU（`moka` crate）で評価結果を tenant + rule_id + input_hash で cache する（TTL 60 秒）。`api_server` モジュールは tonic gRPC で `EvaluateDecision(tenant_id, rule_id, input)` を公開する。in-process のみで完結し、外部ストアは ZEN Engine 用に存在しない。

**確定段階**: リリース時点。**対応要件**: FR-T1-DECISION-\*、NFR-B-PERF-004。**参照**: 構想設計 ADR-RULE-001。

### DS-SW-COMP-062 ZEN Engine 統合方式

`zen-engine` crate（GoRules 製、MIT ライセンス）を `Cargo.toml` で pin 固定する。ZEN の評価は `zen_engine::DecisionEngine::evaluate()` を同期呼び出しし、CPU bound のため `spawn_blocking` で実行する。評価結果は `serde_json::Value` で返り、tonic レスポンスの `google.protobuf.Struct` に変換する。評価エラー（ルール不整合・入力型不一致）は `zen_engine::EngineError` を google.rpc.Status に写像し、gRPC `InvalidArgument` / `FailedPrecondition` で返す。

**確定段階**: リリース時点。**対応要件**: FR-T1-DECISION-001、FR-T1-DECISION-002、NFR-B-PERF-004。

### DS-SW-COMP-063 テナント別ルールセット管理

ルールセットは tenant_id + rule_id + version の 3 属性で識別する。Control API `/admin/rules` で新規版登録・有効化（atomic swap）・取消を行う。cache は `DashMap<(tenant_id, rule_id), Arc<RuleSet>>` で保持し、swap 時は新しい Arc を insert して古い Arc は参照終了後に drop される。ルールセット更新はゼロダウンタイムで、inflight 評価は旧版で完了、新評価から新版が使われる。ルールセット本体は JSON（JDM 形式）で保存し、起動時は Control API 経由で初期ロードする。

**確定段階**: リリース時点。**対応要件**: FR-T1-DECISION-003、DX-FM-\*。

### DS-SW-COMP-064 評価結果 cache の LRU 戦略

`moka` crate で非同期 LRU を構築し、最大 10,000 エントリ / TTL 60 秒 / idle TTL 30 秒で運用する。cache キーは (tenant_id, rule_id, input_hash) で、input_hash は入力 JSON の canonical SHA-256。cache hit 率は Prometheus metric `k1s0_decision_cache_hit_ratio` で観測し、50% を下回ると想定 RPS が設計前提を超えていると判断する。cache は Pod ローカルで、replica 間で共有しない（共有コストが p99 1ms を壊すため）。

**確定段階**: リリース時点。**対応要件**: NFR-B-PERF-004、NFR-D-MON-\*。

### DS-SW-COMP-065 Decision Pod の起動ウォームアップ

Pod 起動時は Control API から高頻度ルールセット（tenant top 10 × rule top 5 = 50 セット）をプリロードし、LRU cache に常駐化する。プリロード完了まで readiness probe を fail させることで、cold cache の Pod がトラフィックを受けないようにする。プリロード所要時間は 3〜5 秒を目標とする。プリロード対象は Backstage の Frequently Used Rules レポートから静的に決定する（採用後の運用拡大時 で動的学習化）。

**確定段階**: リリース時点。**対応要件**: NFR-B-PERF-004、NFR-A-FT-001。

## COMP-T1-PII 内部モジュール詳細

COMP-T1-PII は PII 検出とマスキングに特化した Pod であり、副作用なしの純関数で実装される。外部ストアを持たず、テナント設定は Decision API 経由で取得する。

### DS-SW-COMP-066 PII Pod の内部モジュール構成

COMP-T1-PII は 4 モジュールで構成する。`parser` モジュールはテキストを走査し PII パターンを検出する。`masker` モジュールは検出位置を置換・暗号化する。`rule_fetcher` モジュールはテナント別 PII 検出ルールを COMP-T1-DECISION から取得する。`api_server` モジュールは tonic gRPC で `MaskPII(tenant_id, text, contexts)` を公開する。全モジュール in-memory で、永続ストアは持たない。

**確定段階**: リリース時点。**対応要件**: FR-T1-PII-\*。

### DS-SW-COMP-067 PII 検出パーサ方式

PII 検出は正規表現（`regex` crate）と Aho-Corasick（`aho-corasick` crate）の併用で構成する。正規表現は構造化パターン（電話番号 `\d{2,4}-\d{2,4}-\d{4}` / メール `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}` / マイナンバー `\d{12}`）に使い、Aho-Corasick は辞書ベース検出（社員番号プレフィクス一覧など）に使う。正規表現は Pod 起動時にコンパイルし `OnceLock` でグローバル保持、実行時再コンパイルしない。正規表現 DoS 対策として `regex` crate の linear-time guarantee（NFA ベース）を利用し、catastrophic backtracking を構造的に排除する。

**確定段階**: リリース時点。**対応要件**: FR-T1-PII-001、NFR-G-ENC-\*、NFR-E-DOS-\*。

### DS-SW-COMP-068 マスキング暗号方式

マスキングは 3 モードを用途別に適用する。第一は完全マスク（`***` で置換、例: パスワード）。第二は部分マスク（先頭 3 文字 + `***` + 末尾 2 文字、例: メール）。第三は決定論的暗号（AES-SIV、key は OpenBao Transit engine から取得、検索可能性が必要な場合）。AES-SIV は同じ入力から常に同じ暗号文を生成し、検索インデックスを維持しつつ復号は Transit の許可された主体のみ可能。暗号 key 更新は OpenBao の自動 rotation（年 1 回）に追従し、旧 key は復号のみ可能とする。

**確定段階**: リリース時点（マスク基本 / AES-SIV）。**対応要件**: FR-T1-PII-002、NFR-G-ENC-\*、NFR-H-KEY-001。**参照**: 構想設計 ADR-SEC-002。

### DS-SW-COMP-069 テナント別 PII ルールの取得方式

テナント別 PII 検出ルール（どのパターンを検出するか、どのモードでマスクするか）は COMP-T1-DECISION から ZEN Engine で取得する。PII Pod は起動時に自テナント分ルールを tier1 内部 gRPC で取得し cache（TTL 5 分）する。ルール更新は flagd の push で通知され、次回 cache expire 時に再取得する。テナント新規オンボーディング時は Backstage で PII ルールセットを登録し Decision API に反映する（[../../70_開発者体験方式設計/](../../70_開発者体験方式設計/) 参照）。

**確定段階**: リリース時点。**対応要件**: FR-T1-PII-\*、FR-T1-DECISION-\*、DX-FM-\*。

### DS-SW-COMP-070 PII 呼び出しの低レイテンシ設計

PII 呼び出しは ファサード層の Log Adapter から高頻度で発火するため、p99 3ms 以内を目標とする（NFR-B-PERF-006 の 10ms オーバヘッド予算内の 30%）。実装は in-memory 処理のみで DB / 外部 API を呼ばず、regex コンパイル済みインスタンスのみ使用する。CPU bound のため spawn_blocking は使わず、async context で直接処理する（1 呼び出しあたり < 500μs を目標）。複数テキストの一括処理は batch API `MaskPIIBatch` で 100 件までをまとめて処理する。

**確定段階**: リリース時点。**対応要件**: NFR-B-PERF-006、FR-T1-PII-\*。

## 自作層共通の横断事項

### DS-SW-COMP-071 tracing と OTel 統合

tracing は `tracing` crate + `tracing-subscriber` + `tracing-opentelemetry` で構築する。全 gRPC リクエストに root span を張り、`trace_id` / `span_id` を自動生成して Jaeger/Tempo 互換の OTLP で OpenTelemetry Collector に送出する。facade 層からの span 親子関係は `grpc-metadata` の `traceparent` ヘッダで伝搬し（[../03_内部インタフェース方式設計/03_Go_Rust間言語境界方式.md](../03_内部インタフェース方式設計/03_Go_Rust間言語境界方式.md) 参照）、Go ⇔ Rust の境界をまたいで単一 trace を維持する。sampling は リリース時点で 10% / 1% に削減する。

**確定段階**: リリース時点。**対応要件**: FR-T1-TELEMETRY-\*、NFR-D-TRACE-\*。

### DS-SW-COMP-072 メトリクス収集と公開

Prometheus メトリクスは `prometheus` crate で管理し、`axum` で `/metrics` エンドポイント（ポート 9090）を公開する。メトリクス名は `k1s0_` プレフィクスで統一し、ラベル `component` / `method` / `status` / `tenant_id`（低カーディナリティ化のため tenant は上位 100 に絞る）を付与する。Histogram の bucket は `[0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]` で自作層特有の低レイテンシ（sub-ms 域）を捉える。

**確定段階**: リリース時点。**対応要件**: FR-T1-TELEMETRY-\*、NFR-D-MON-\*。

### DS-SW-COMP-073 エラーハンドリング方式

Rust のエラーは `thiserror` crate でドメイン固有 enum を定義し、`anyhow` は app-level の context 付与にのみ使う（library 層では禁止）。gRPC レスポンスには `google.rpc.Status` を使い、`code` は Protobuf enum、`message` は日本語 i18n 対応、`details` に `k1s0.errors.v1.ErrorDetail`（error_code / trace_id / retryable / docs_url）を詰める。panic は避け、panic 発生時は Pod 全体を crash させずに該当 request のみ 500 で返す（`tokio::task::JoinError` でキャッチ）。

**確定段階**: リリース時点。**対応要件**: NFR-E-ERR-\*、FR-T1-LOG-\*。

### DS-SW-COMP-074 ヘルスチェックエンドポイント

各自作 Pod は `/healthz`（HTTP GET、liveness）と `/readyz`（HTTP GET、readiness）を `axum` で公開する。liveness は tokio runtime 生存と tonic server listening で 200 を返す。readiness は Pod 固有の条件で判定する: AUDIT は Kafka consumer attached + PostgreSQL connection available、DECISION は rule preload 完了、PII は regex コンパイル完了。Kubernetes probe の設定は ファサード層と同じ（initialDelaySeconds 10s / periodSeconds 5s / timeoutSeconds 3s / failureThreshold 3）。

**確定段階**: リリース時点。**対応要件**: NFR-A-FT-001、NFR-D-MON-\*。

### DS-SW-COMP-075 メモリフットプリント目標

Pod 起動後の定常 memory 目標は AUDIT 512 MiB、DECISION 1.5 GiB（ZEN cache 込み）、PII 256 MiB とする。目標超過時は Prometheus alert `k1s0_pod_memory_usage_bytes` で検知する。Rust の `jemalloc` は optional で有効化（`jemallocator` crate）し、Go ファサード層との align を取る。memory leak 検出は リリース時点 で `heaptrack` + `valgrind` の定期実行をテスト環境で行う。

**確定段階**: リリース時点（目標 / leak 検出）。**対応要件**: NFR-B-CAP-\*、NFR-F-ENV-\*。

### DS-SW-COMP-076 unsafe コードの扱い

Rust の `unsafe` ブロックは原則禁止し、使用する場合は SAFETY コメントで不変条件を明記し、必ずレビュー承認を得る。FFI 呼び出し（例: ZEN Engine が将来 C FFI 経由になる場合）は crate 境界で安全 wrapper を提供する。現時点では ZEN Engine が pure Rust のため unsafe 不要。PII の regex 処理も unsafe 不要。crate 依存の unsafe は `cargo-geiger` で定期監査し、unsafe 使用率が増えた場合は ADR 起票する。

**確定段階**: リリース時点。**対応要件**: NFR-E-ENC-001、NFR-H-INT-\*。

### DS-SW-COMP-077 CPU bound 処理の切り分け

CPU bound 処理（hash 計算・regex マッチ・ZEN 評価）は `spawn_blocking` で別スレッドプールに逃し、async task の latency を保護する。spawn_blocking 後の結果は `JoinHandle::await` で回収し、async context に戻してレスポンスを返す。spawn_blocking のプールサイズは Pod の CPU limit × 2 を上限とし、上限到達時は新規タスクが queue に積まれて tail latency が増える。これを検知するため `tokio::runtime::Handle::metrics()` で blocking queue 長を監視する。

**確定段階**: リリース時点。**対応要件**: NFR-B-PERF-\*、NFR-B-WL-\*。

### DS-SW-COMP-078 自作層の CI ビルド方式

Rust ビルドは GitHub Actions で `cargo build --release --workspace` + `cargo test` + `cargo clippy -- -D warnings` + `cargo fmt --check` + `cargo audit`（依存脆弱性）+ `cargo deny check licenses`（ライセンス監査）を必須ステップとする。incremental ビルドは禁止（production container は always clean build）、`sccache` で cache 共有を加速する。container image は `gcr.io/distroless/cc-debian12` ベースで、static linking はせず dynamic linking（musl へのポータビリティは不要）。image サイズ目標は各 Pod 80 MiB 以下。

**確定段階**: リリース時点。**対応要件**: DX-CICD-\*、DX-TEST-\*、NFR-F-ENV-\*。

### DS-SW-COMP-079 graceful shutdown と状態保全

SIGTERM を `tokio::signal::unix::SIGTERM` で受け、以下順序で shutdown する: (1) tonic server に新規受付停止を通知、(2) 進行中リクエストの完了待機（最大 30 秒）、(3) Pod 固有のクリーンアップ（AUDIT は Kafka commit 完了、DECISION は cache を drop、PII は何もしない）、(4) tokio runtime 停止。terminationGracePeriodSeconds は 35 秒に設定する。StatefulSet の AUDIT は PVC detach に時間がかかるため 60 秒に延長する。

**確定段階**: リリース時点。**対応要件**: NFR-A-FT-001、NFR-A-REC-001。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定段階 |
|---|---|---|
| DS-SW-COMP-050 | Rust 自作層の共通基盤 | 採用初期 |
| DS-SW-COMP-051 | tonic gRPC サーバ実装方式 | リリース時点 |
| DS-SW-COMP-052 | tokio runtime チューニング | リリース時点 |
| DS-SW-COMP-053 | Dapr sidecar 不要の根拠 | リリース時点 |
| DS-SW-COMP-054 | AUDIT 5 モジュール構成 | 採用初期 |
| DS-SW-COMP-055 | ハッシュチェーン生成方式（SHA-256 + CBOR） | リリース時点 |
| DS-SW-COMP-056 | WORM append 実装方式（PG トリガ） | リリース時点 |
| DS-SW-COMP-057 | replica パーティショニング方式 | リリース時点/2 |
| DS-SW-COMP-058 | MinIO Object Lock 長期保管 | リリース時点 |
| DS-SW-COMP-059 | 監査改ざん検証 API | リリース時点 |
| DS-SW-COMP-060 | Kafka Consumer at-least-once 設定 | リリース時点 |
| DS-SW-COMP-061 | DECISION 4 モジュール構成 | リリース時点 |
| DS-SW-COMP-062 | ZEN Engine 統合方式 | リリース時点 |
| DS-SW-COMP-063 | テナント別ルールセット管理 | リリース時点 |
| DS-SW-COMP-064 | 評価結果 cache LRU 戦略（moka） | リリース時点 |
| DS-SW-COMP-065 | Decision 起動ウォームアップ | リリース時点 |
| DS-SW-COMP-066 | PII 4 モジュール構成 | リリース時点 |
| DS-SW-COMP-067 | PII 検出パーサ方式（regex + Aho-Corasick） | リリース時点 |
| DS-SW-COMP-068 | マスキング暗号方式（AES-SIV） | 採用初期 |
| DS-SW-COMP-069 | テナント別 PII ルール取得 | リリース時点 |
| DS-SW-COMP-070 | PII 呼び出しの低レイテンシ設計 | リリース時点 |
| DS-SW-COMP-071 | tracing と OTel 統合 | リリース時点 |
| DS-SW-COMP-072 | メトリクス収集と公開 | リリース時点 |
| DS-SW-COMP-073 | エラーハンドリング方式（thiserror） | リリース時点 |
| DS-SW-COMP-074 | ヘルスチェックエンドポイント | リリース時点 |
| DS-SW-COMP-075 | メモリフットプリント目標 | 採用初期 |
| DS-SW-COMP-076 | unsafe コードの扱い | リリース時点 |
| DS-SW-COMP-077 | CPU bound 処理の切り分け（spawn_blocking） | リリース時点 |
| DS-SW-COMP-078 | 自作層 CI ビルド方式 | リリース時点 |
| DS-SW-COMP-079 | graceful shutdown と状態保全 | リリース時点 |

## 対応要件一覧

- FR-T1-AUDIT-001 / FR-T1-AUDIT-002 / FR-T1-AUDIT-003（ハッシュチェーン・WORM・長期保管）
- FR-T1-DECISION-001 / FR-T1-DECISION-002 / FR-T1-DECISION-003 / FR-T1-DECISION-004（JDM 評価・ルール管理）
- FR-T1-PII-001 / FR-T1-PII-002（検出・マスキング）
- FR-T1-LOG-\* / FR-T1-TELEMETRY-\*（観測連携）
- FR-INFO-AUDITMODEL-001（監査イベントスキーマ）
- NFR-A-CONT-003 / NFR-A-FT-001 / NFR-A-REC-001
- NFR-B-PERF-004（Decision p99 1ms）/ NFR-B-PERF-006（計装 10ms）/ NFR-B-WL-\* / NFR-B-CAP-\*
- NFR-C-NOP-001 / NFR-C-NOP-003（7 年保管）
- NFR-D-MON-\* / NFR-D-TRACE-\* / NFR-E-AC-001〜005 / NFR-E-NW-001〜004 / NFR-E-ENC-001
- NFR-F-ENV-\* / NFR-G-ENC-\* / NFR-H-INT-001 / NFR-H-KEY-001
- DX-CICD-\* / DX-TEST-\* / DX-FM-\* / DX-RB-\*

構想設計 ADR-TIER1-001 / ADR-DATA-001 / ADR-DATA-003 / ADR-RULE-001 / ADR-SEC-002 と双方向トレースする。
