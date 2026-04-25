# 02. Dapr ファサード層コンポーネント

本ファイルは IPA 共通フレーム 2013 の **ソフトウェア方式設計プロセス 7.1.2.1（ソフトウェア構造とコンポーネントの方式設計）** に対応する。tier1 Dapr ファサード層を構成する Go 実装 3 コンポーネント（COMP-T1-STATE / COMP-T1-SECRET / COMP-T1-WORKFLOW）の内部モジュール構成、Dapr Building Blocks の使い分け、公開 API から内部モジュールを経由して Dapr SDK とバックエンド OSS に至る呼び出し階層を方式として固定化する。

## 本ファイルの位置付け

[01_tier1全体コンポーネント俯瞰.md](01_tier1全体コンポーネント俯瞰.md) が 6 Pod の外形と配置を確定したのに対し、本ファイルはファサード層 3 Pod の内部を 1 階層掘り下げる。各 Pod の中で「API Router」「Dapr Adapter」「Log Adapter」「Metrics Emitter」「Policy Enforcer」の 5 モジュールがどの責務を担い、どの順序で呼び出されるかを固定することで、詳細設計段階でのモジュール境界の乱れを防ぐ。構想設計 [../../../02_構想設計/02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md](../../../02_構想設計/02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md) が「Dapr を tier1 内部に閉じ込める」方針を確立している前提で、本ファイルはその内部構造を仕様化する。

tier1 公開 API は Dapr SDK の引数をそのまま通すのではなく、5 モジュールの直列パイプラインを通過させることで横断的関心事（認証・テナント・監査・観測・エラー変換）を必ず強制する。このパイプライン設計が緩むと、例えば認証バイパスや OTel span 欠落が発生し、採用検討で約束した SLO/SLA と監査要件が崩れる。本ファイルはパイプラインの構造と通過順を方式として確定させる。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-SW-COMP-020` 〜 `DS-SW-COMP-049` の 30 件に加え、6 段階多層防御統括として `DS-SW-COMP-141` を補欠採番する。通番は [01_tier1全体コンポーネント俯瞰.md](01_tier1全体コンポーネント俯瞰.md) の DS-SW-COMP-019 から連続し、[03_自作Rust領域コンポーネント.md](03_自作Rust領域コンポーネント.md) で DS-SW-COMP-050 に継続する。DS-SW-COMP-141 は企画書 L197-212 の 6 段階多層防御を設計側で統括するための補欠 ID で、050〜140 の連番は既に他ファイル（03 / 04 / 05 / 06 および本ファイルの 02 統括前枠）で全て消費済みのため、最小の未使用番号として 141 を割り当てる。現行の正式採番計画では 02 ファイル末尾の 050 は 03 側が先行使用しており、統括節分は未使用帯に繰り送る方針である。

## ファサード層の共通構造

3 コンポーネントはいずれも「API Router → Policy Enforcer → Dapr Adapter → バックエンド OSS」の直列パイプラインを持つ。横串として「Log Adapter」「Metrics Emitter」が全モジュール境界に挿入され、OTel span / Prometheus メトリック / 構造化ログを生成する。この構造は全 3 Pod で共通で、公開 API が異なっても内部の骨格は同一にすることで、運用者が Pod 間で一貫した観測・デバッグ手順を適用できる。モジュール間の契約は Go interface で定義し、単体テストでは各モジュールをモック化する。

### DS-SW-COMP-020 ファサード層 5 モジュール構成

全ファサード Pod は以下 5 モジュールを持つ。API Router は gRPC サーバ（`grpc-go`）として公開 API のエンドポイントを受け、リクエストを内部 Context に変換する。Policy Enforcer は認証（JWT 検証）・認可（RBAC）・テナント境界（`tenant_id` の強制付与）・レート制限・冪等性キー管理を実施する。Dapr Adapter は Dapr Go SDK を呼び出してバックエンドに到達する唯一の経路となる。Log Adapter は全モジュール境界で `zerolog` の構造化ログを生成する。Metrics Emitter は `prometheus/client_golang` で RED メトリック（Rate / Errors / Duration）を Pod 単位で公開する。これら 5 モジュールの順序と役割は 3 Pod 共通で、モジュール名・interface 名も統一する。

**確定段階**: リリース時点。**対応要件**: FR-T1-INVOKE-\*、NFR-E-AC-\*、NFR-E-MON-\*、NFR-D-MTH-\*。

### DS-SW-COMP-021 呼び出しパイプラインの固定順序

リクエストは必ず「API Router → Policy Enforcer → Dapr Adapter → Dapr Go SDK → daprd サイドカー → OSS バックエンド」の順で流れる。Policy Enforcer をバイパスして Dapr Adapter を直接呼ぶコードパスは禁止する（レビュー・Lint で強制）。Log Adapter と Metrics Emitter は各境界で呼び出され、`trace_id` / `span_id` / `tenant_id` / `request_id` を必ず引き回す。エラーが発生した場合は Dapr Adapter 層で Protobuf の google.rpc.Status に変換され、API Router が gRPC ステータスとして返却する。この固定順序が崩れると、例えば OTel span が欠落したり、認証検証前に Dapr 呼び出しが発火して情報漏洩につながる。

**確定段階**: リリース時点。**対応要件**: NFR-E-AC-\*、NFR-D-MTH-\*、NFR-E-MON-001。

### DS-SW-COMP-022 モジュール間 interface と DI

モジュール間は Go interface で疎結合とし、`wire`（Google Wire）による依存注入でコンパイル時に結合を確定する。runtime reflection ベースの DI（uber/dig など）は起動時間と監査可能性の観点で禁止する。interface 名は `APIRouter` / `PolicyEnforcer` / `DaprAdapter` / `LogAdapter` / `MetricsEmitter` で統一し、各 Pod の `internal/` 配下に実装を置く（[06_パッケージ構成_Rust_Go.md](06_パッケージ構成_Rust_Go.md) 参照）。単体テストでは interface をモックに差し替え、結合テストでは Dapr Adapter のみモック化して Policy Enforcer までの挙動を検証する。

**確定段階**: リリース時点。**対応要件**: DX-TEST-\*、DX-GP-\*。

## COMP-T1-STATE 内部モジュール詳細

COMP-T1-STATE は 5 公開 API（Service Invoke / State / PubSub / Binding / Feature）を集約する Pod であり、ファサード 3 Pod の中で最も複雑な内部構造を持つ。公開 API ごとに API Router の endpoint handler が別れるが、Policy Enforcer から下流は共通モジュールを使い回す。

### DS-SW-COMP-023 API Router の endpoint 分割方式

API Router は 5 公開 API を別々の Proto service 定義として公開し、それぞれ `k1s0.v1.service_invoke.ServiceInvokeService` / `k1s0.v1.state.StateService` / `k1s0.v1.pubsub.PubSubService` / `k1s0.v1.binding.BindingService` / `k1s0.v1.feature.FeatureService` の 5 サービスを単一 gRPC サーバ上で登録する。単一ポート（50051）で全 5 サービスを serving し、gRPC Reflection を 運用蓄積後の Stage 環境で有効化する（Production では無効）。サービス間の依存（例: PubSub 配信前に Feature 判定）は API Router レベルでは混ざらず、必要な場合は Policy Enforcer 経由で共通ロジックを呼ぶ。

**確定段階**: リリース時点。**対応要件**: FR-T1-INVOKE-\*、FR-T1-STATE-\*、FR-T1-PUBSUB-\*、FR-T1-BINDING-\*、FR-T1-FEATURE-\*。

### DS-SW-COMP-024 Dapr Building Blocks の使い分け

COMP-T1-STATE は Dapr の 5 Building Block を使い分ける（採用後の運用拡大時 での完成状態）。Service Invoke API は Dapr `serviceinvocation` BB を通じて tier2 マイクロサービス間の gRPC 相互呼び出しを仲介する。State API は Dapr `state` BB で Valkey Cluster に KV 操作する（TTL / ETag / Transaction 対応）。PubSub API は Dapr `pubsub` BB で Kafka topic に publish / subscribe する（CloudEvents 1.0 準拠）。Binding API は Dapr `bindings` BB で 外部 HTTP / SMTP / S3 等への input / output binding を行う。Feature API は Dapr ではなく `flagd` gRPC を直接呼ぶ（Feature Management 用 BB が Dapr 側で stable 化していないため）。各 BB のバージョン固定は `go.mod` で Dapr Go SDK のバージョンピンで管理する。

**確定段階**: 採用後の運用拡大時（State / PubSub / Service Invoke / Binding / Feature の全 5 BB を一括で本格化）。採用初期 は COMP-T1-STATE 自体が `k1s0.Log` のみの最小ファサードとして稼働し、State 以下 5 API はスタブで Dapr BB との結線を持たない（[企画書.md](../../../01_企画/企画書.md) の 採用初期 / 採用初期 / 採用初期 のいずれにも `k1s0.State` 以下は含まれていないため、Dapr BB の stable 依存も 採用後の運用拡大時 に寄せる）。**対応要件**: FR-T1-STATE-\*、FR-T1-PUBSUB-\*、FR-T1-BINDING-\*、FR-T1-INVOKE-\*、FR-T1-FEATURE-\*。**参照**: 構想設計 ADR-DATA-002（Valkey Cluster）、ADR-MSG-001（Strimzi Kafka）。

### DS-SW-COMP-025 State API の内部モジュール

State API 内部では「etag 検証 → TTL 計算 → Consistency レベル決定 → Dapr state BB 呼び出し」の 4 ステップを経る。etag 検証は Optimistic Concurrency Control で書込衝突を検知し、衝突時は gRPC status `FailedPrecondition` を返す。TTL は tier2 からの指定値を tenant policy でクランプ（最小 1s / 最大 7 日）する。Consistency は リリース時点 では strong 固定、採用後の運用拡大時 で eventual 切替を Feature Flag 経由で許容する。Dapr `state` BB 呼び出し時のタイムアウトは 200ms に設定し、p99 10ms 目標（NFR-B-PERF-003）の内訳に収まるよう監視する。

**確定段階**: リリース時点。**対応要件**: FR-T1-STATE-001、FR-T1-STATE-002、FR-T1-STATE-003、NFR-B-PERF-003。

### DS-SW-COMP-026 PubSub API の内部モジュール

PubSub API 内部では「CloudEvents 変換 → Schema Registry 確認（採用後の運用拡大時）→ Dapr pubsub BB publish」の 3 ステップを経る。CloudEvents 変換は tier2 から渡された JSON を CloudEvents 1.0 の必須ヘッダ（id / source / type / subject / time / datacontenttype / specversion）で包む。採用後の運用拡大時 で Confluent Schema Registry と接続する場合、publish 前に schema 適合を検証する（リリース時点 では検証省略で tier2 側の責任）。Subscribe 側は Dapr pubsub BB の subscription を API Router で受け、ACK / NACK を Dapr SDK 経由で返す。p99 50ms 目標（NFR-B-PERF-005）に対して CloudEvents 変換 5ms / SR 検証 10ms / Kafka publish 30ms を内訳として配分する。

**確定段階**: 採用後の運用拡大時（publish / subscribe 本格化 + Schema Registry 連携を一括で実施）。PubSub は [企画書.md](../../../01_企画/企画書.md) で 採用後の運用拡大時 の「Kafka（Strimzi）」+「Apicurio Registry」導入と同期する。**対応要件**: FR-T1-PUBSUB-\*、NFR-B-PERF-005。

### DS-SW-COMP-027 Feature API の内部モジュール

Feature API は flagd gRPC クライアントを内部に抱え、評価コンテキスト（tenant_id / user_id / environment / segment）を構築して flagd に送る。flagd のレスポンスはローカル cache（5 秒 TTL）に保持し、p99 5ms を達成する。flagd が到達不能な場合は cache から stale 応答を返し、cache も miss の場合は変数ごとにコードに埋めた既定値を返す（graceful degradation）。tenant 別の flag 定義は flagd の sync provider（リリース時点 は file provider、採用後の運用拡大時 で Kubernetes ConfigMap sync provider）で供給する。

**確定段階**: リリース時点。**対応要件**: FR-T1-FEATURE-\*、NFR-A-CONT-002、DX-FM-\*。**参照**: 構想設計 ADR-FM-001（flagd）。

### DS-SW-COMP-028 Service Invoke API の内部モジュール

Service Invoke API は Dapr `serviceinvocation` BB を通じて tier2 / tier3 サービス間の gRPC 呼び出しを仲介する。内部では「caller 認証 → callee 存在確認（Dapr app-id 登録確認）→ mTLS 確立（Istio Ambient）→ Dapr invoke」の順で実行する。Istio Ambient との役割分担は、mTLS は Istio 側、認可は Policy Enforcer 側とし、Dapr の mTLS 機能は リリース時点 では無効化して Istio に委譲する（[../../30_共通機能方式設計/](../../30_共通機能方式設計/) 参照）。

**確定段階**: リリース時点。**対応要件**: FR-T1-INVOKE-\*、NFR-E-AC-001〜005、NFR-E-NW-001〜004。

### DS-SW-COMP-029 Binding API の内部モジュール

Binding API は external resource（外部 HTTP / SMTP / S3 / MQ）への output binding を担う。内部で「binding 定義読込 → Policy Enforcer の tenant 許可確認 → Dapr bindings BB invoke → 結果 wrap」の順で実行する。リリース時点で は HTTP binding のみ / SMTP / S3 を追加、採用後の運用拡大時 で MQTT を追加する。bindings 定義は Dapr Component YAML として ConfigMap で管理し、tenant 別の許可は Policy Enforcer 側の policy.rego で制御する（[../../30_共通機能方式設計/](../../30_共通機能方式設計/) 参照）。

**確定段階**: 採用初期/2。**対応要件**: FR-T1-BINDING-\*、NFR-E-AC-\*。

## COMP-T1-SECRET 内部モジュール詳細

COMP-T1-SECRET は Secrets API のみを担う単一責務 Pod であり、Leader Election が内部構造に追加される点が COMP-T1-STATE と異なる。

### DS-SW-COMP-030 Leader Election 内部モジュール

Leader Election は Kubernetes `coordination.k8s.io/v1` Lease を用い、`client-go` の `leaderelection` パッケージで実装する。active Pod が Lease を保持する間だけ OpenBao に接続しリース発行 API を受ける。standby Pod は API Router で受けたリクエストを active Pod の内部 gRPC（`internal/transfer` service）に転送する。Leader 切替は Lease TTL 15 秒 / renew 5 秒 / retry 2 秒の設定で行い、active 死亡時 standby が 10 秒以内に昇格する（詳細は [../../40_制御方式設計/](../../40_制御方式設計/) 参照）。

**確定段階**: リリース時点。**対応要件**: FR-T1-SECRETS-002、NFR-A-CONT-002、NFR-A-FT-001。

### DS-SW-COMP-031 Secrets API の内部モジュール

Secrets API 内部では「Dapr secrets BB 呼び出し → OpenBao 動的 Secret 発行 → リース情報を Valkey cache → tier2 に返却」の順で実行する。動的 Secret のリース ID とライフタイムは Valkey に保存し、別 Pod（COMP-T1-WORKFLOW 等）からのリース延長要求を active Pod に転送する。リース失効時は OpenBao の Watch 機構で検知して Valkey cache を更新する。PKI Secret と Transit Secret は別 endpoint（`/v1/pki/`、`/v1/transit/`）で公開し、用途別にレート制限を分ける。

**確定段階**: リリース時点。**対応要件**: FR-T1-SECRETS-\*、NFR-E-ENC-001、NFR-H-KEY-001。**参照**: 構想設計 ADR-SEC-002（OpenBao）。

### DS-SW-COMP-032 OpenBao 接続プールと health check

OpenBao 接続は active Pod のみが保持し、HTTP/2 + keep-alive で 1 本の持続接続にまとめる。接続障害時は 3 秒・6 秒・12 秒の指数バックオフで再接続を試み、30 秒継続失敗で Pod 自体を crashed と見なし Kubernetes の liveness probe で再起動する。OpenBao 側の sealed 状態検出（`/v1/sys/health` の 503）は degrade モードに遷移し、既発行リースのみ Valkey cache から返す。新規リース発行は 503 を返却する（NFR-A-CONT-002 の degrade 仕様）。

**確定段階**: リリース時点。**対応要件**: NFR-A-CONT-002、NFR-A-FT-001。

## COMP-T1-WORKFLOW 内部モジュール詳細

COMP-T1-WORKFLOW は Workflow API と Saga API を担い、U-WORKFLOW-001 の決着まで Dapr Workflow と Temporal の 2 実装を Pluggable 構造で並存させる。

### DS-SW-COMP-033 Pluggable Workflow Backend

Workflow Backend は interface `WorkflowBackend` を介して実装差異を吸収する。実装 1 は `DaprWorkflowBackend`（Dapr Workflow SDK）、実装 2 は `TemporalWorkflowBackend`（Temporal Go SDK）を提供する。起動時の環境変数 `K1S0_WORKFLOW_BACKEND=dapr|temporal` で切替し、同一 Pod 内で両方を並行稼働しない（state store が分離されるため）。U-WORKFLOW-001 決着後は片方を削除するが、それまではテスト環境で両方を比較検証する（[../../40_制御方式設計/](../../40_制御方式設計/) 参照）。

**確定段階**: リリース時点 中に確定。**対応要件**: FR-T1-WORKFLOW-\*、U-WORKFLOW-001。**参照**: 構想設計 ADR-RULE-002。

### DS-SW-COMP-034 task queue sticky 方式

Workflow タスクは task queue に投入され、3 replica の各 Pod が sticky に worker として pull する。sticky 割当は `tenant_id` と `workflow_id` のハッシュで replica を決定し、同一 workflow_id の再入は必ず同一 replica に戻す。Pod 死亡時は Temporal / Dapr Workflow 側の lease 失効で sticky 解除され、別 replica が引き継ぐ（RTO 30 秒以内）。HPA でスケールアウトすると新 replica への sticky 再割当が発火し補償遅延が発生するため、replica 数は固定 3 とする。

**確定段階**: リリース時点。**対応要件**: FR-T1-WORKFLOW-001、FR-T1-WORKFLOW-002、NFR-A-REC-001。

### DS-SW-COMP-035 Saga 補償トランザクション

Saga API は tier2 側の workflow 定義を受け付け、ステップ単位の forward / compensate 関数を内部で管理する。補償発火は forward 失敗時または明示的 rollback 要求時に実行され、順序は逆順で、個別ステップ失敗時は「DLQ に積む → 手動リカバリ Runbook」のフォールバックを持つ。詳細な Saga 方式は [../../40_制御方式設計/](../../40_制御方式設計/) に委ね、本ファイルは API と Workflow Backend 間の境界のみ定義する。

**確定段階**: リリース時点。**対応要件**: FR-T1-WORKFLOW-003、FR-T1-WORKFLOW-004。

## 横断モジュールの仕様

以下 3 モジュールは 3 Pod 共通で同一の interface と実装を共有する。`k1s0-common` package から import して使い、Pod 固有の拡張は禁止する。

### DS-SW-COMP-036 Policy Enforcer の内部構造

Policy Enforcer は「JWT 検証 → Tenant 境界確認 → RBAC 評価 → Rate Limit → 冪等性キー確認」の 5 ステップを持つ。JWT 検証は Keycloak 発行トークンを JWKS キャッシュで検証し、tenant_id をクレームから抽出する。RBAC 評価は OpenPolicyAgent（OPA）の SDK モード（Rego を Pod 内評価）で policy.rego を評価する。Rate Limit は tenant 別に redis-cell（Valkey）で token bucket を管理する。冪等性キーは tier2 から渡された `X-Idempotency-Key` を Valkey に 24 時間保持し、重複検出で同一レスポンスを返す。

**確定段階**: リリース時点（JWT/Tenant / RBAC/RateLimit / 冪等性）。**対応要件**: NFR-E-AC-001〜005、DX-GP-\*。

### DS-SW-COMP-037 Log Adapter の構造化ログ方式

Log Adapter は `zerolog` を基盤とし、全ログエントリに `trace_id` / `span_id` / `tenant_id` / `component` / `request_id` / `level` / `msg` を必須フィールドとして含める。出力先は stdout（JSON Lines）のみで、リリース時点 で導入する OTel Collector Agent DaemonSet が拾い上げて Prometheus（メトリック派生分）に転送し、リリース時点 で Loki が導入された後はログ本体を Loki に集約する（[企画書.md](../../../01_企画/企画書.md) の 採用初期で OTel Collector / Loki 導入）。リリース時点 では stdout のみで kubectl logs による一次確認に留まる。機密情報（Secret 値・PII）は zerolog hook で自動マスキングし、COMP-T1-PII への gRPC 呼び出しで安全性を確保する（COMP-T1-PII 本体の本格実装は 採用後の運用拡大時 の tier1 自作 Rust 領域投入と同期、詳細は [03_自作Rust領域コンポーネント.md](03_自作Rust領域コンポーネント.md) 参照）。

**確定段階**: リリース時点（stdout JSON Lines 出力 / OTel Collector 連携 / Loki 集約）、採用後の運用拡大時（PII マスキング同期呼び出し）。**対応要件**: FR-T1-LOG-\*、NFR-D-MON-\*、NFR-G-ENC-\*。

### DS-SW-COMP-038 Metrics Emitter の RED メトリック方式

Metrics Emitter は RED（Rate / Errors / Duration）モデルで Pod あたり以下を公開する。`k1s0_request_total{component, api, status}`（Counter）、`k1s0_request_errors_total{component, api, error_type}`（Counter）、`k1s0_request_duration_seconds{component, api}`（Histogram、bucket [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1, 5]）。Prometheus `ServiceMonitor` で scrape し Mimir に保存する。追加メトリクスとして Pod 固有（例: STATE は `k1s0_valkey_conn_pool_size`、SECRET は `k1s0_openbao_lease_count`、WORKFLOW は `k1s0_sticky_reassign_total`）を定義する。

**確定段階**: リリース時点。**対応要件**: FR-T1-TELEMETRY-\*、NFR-D-MON-\*、NFR-B-PERF-\*。

## Dapr daprd サイドカー構成

ファサード 3 Pod は daprd サイドカーを 1 個ずつ持ち、アプリコンテナと UNIX domain socket または localhost:3500 で通信する。

### DS-SW-COMP-039 daprd サイドカー注入方式

daprd は Dapr Control Plane の `dapr-sidecar-injector` MutatingWebhook で自動注入する。対象 Pod は annotation `dapr.io/enabled=true` と `dapr.io/app-id=<component-id>`（例: `t1-state`）で指定する。`dapr.io/app-port=50051` でアプリ側 gRPC ポートを指定し、daprd は localhost:50001（gRPC）と localhost:3500（HTTP）で inbound を受ける。リソース制限は CPU request 100m / limit 500m、Memory request 128Mi / limit 512Mi で固定する（[../../50_非機能方式設計/](../../50_非機能方式設計/) 参照）。

**確定段階**: リリース時点。**対応要件**: NFR-B-CAP-\*、NFR-F-ENV-\*。

### DS-SW-COMP-040 Dapr Component YAML 管理方式

Dapr Component YAML は GitOps 管理とし、`infra/dapr-components/` リポジトリで Argo CD が同期する。Component は tenant を跨ぐ共通（Valkey / Kafka / OpenBao）と tenant 別（SMTP / 外部 HTTP binding）に分け、tenant 別は namespace scope で分離する。秘密情報は Component の `auth.secretStore` で OpenBao を参照し、YAML 自体には含めない（詳細は [../../30_共通機能方式設計/](../../30_共通機能方式設計/) 参照）。

**確定段階**: リリース時点。**対応要件**: NFR-C-NOP-002、NFR-H-KEY-001。

## API → モジュール → Dapr SDK → OSS の呼び出し階層

具体的なコールスタックは 4 階層で整理する。第 1 階層は tier2 からの gRPC 呼び出し（例: `k1s0.v1.state.StateService.SaveState`）。第 2 階層は API Router → Policy Enforcer → Dapr Adapter の内部パイプライン。第 3 階層は Dapr Go SDK の `client.SaveState()` 呼び出し。第 4 階層は daprd サイドカー → Valkey Cluster の実通信。この 4 階層のうち第 2 階層を tier1 が独自実装し、第 3・4 階層は Dapr の責務とする。第 1 階層の契約は [../02_外部インタフェース方式設計/06_API別詳細方式/](../02_外部インタフェース方式設計/06_API別詳細方式/) で、第 2 階層は本ファイル、第 3・4 階層は Dapr 公式ドキュメントに委ねる。

### DS-SW-COMP-041 4 階層コールスタックの責務境界

責務境界は以下の 4 階層で整理される。第 1 階層（公開 API）は gRPC エンドポイント受信の責務を持ち、所管は k1s0 外部契約で関連 ID は DS-SW-EIF-\*。第 2 階層（内部パイプライン）は Policy / Log / Metrics の強制を担い、所管は k1s0 本ファイルで関連 ID は DS-SW-COMP-020〜038。第 3 階層（Dapr SDK）は Building Block 呼び出しを担い、所管は Dapr 固定版管理で関連 ID は DS-SW-COMP-024。第 4 階層（daprd → OSS）は実通信とプロトコル変換を担い、所管は Dapr と OSS で関連 ID は構想設計 ADR。

各階層の所管が明確であることで、障害時の切り分けが「L2 以下の問題は Dapr ログ、L2 以上は k1s0 ログ」の 2 択に収束する。これが 採用初期 の 採用側の小規模運用を成立させる前提である。

**確定段階**: リリース時点（責務境界 / 実装）。**対応要件**: NFR-C-NOP-001、DX-RB-\*。

## ファサード層特有の運用考慮

### DS-SW-COMP-042 Dapr バージョンアップ方式

Dapr Control Plane のバージョンアップは四半期に 1 回、マイナーバージョンまでを対象とする（例: 1.14.x → 1.15.x）。メジャーバージョン（例: 1.x → 2.x）は年次計画で 6 か月のバッファを取る。バージョンアップは Canary 手法で dev → staging → production の順で 2 週間かけて展開し、異常時は Argo Rollouts で自動ロールバックする（[../../55_運用ライフサイクル方式設計/](../../55_運用ライフサイクル方式設計/) 参照）。Dapr Go SDK は Control Plane のバージョンに追従し、`go.mod` で厳密 pin する。

**確定段階**: リリース時点。**対応要件**: NFR-C-NOP-\*、NFR-H-KEY-001。

### DS-SW-COMP-043 Feature Flag による段階的公開

新 API の段階的公開は COMP-T1-STATE 内の Feature API を tier1 自身で利用する。`k1s0.internal.feature.tier1.new-api-v2` のような flag を flagd に登録し、tenant 単位で有効化する。Canary 比率は 1% → 5% → 25% → 100% の 4 段階で、各段階 48 時間以上の観測窓を持つ。flag 名の命名規約は `k1s0.internal.<component>.<feature>` に統一する（[../../70_開発者体験方式設計/](../../70_開発者体験方式設計/) 参照）。

**確定段階**: リリース時点。**対応要件**: DX-FM-\*、DX-GP-\*。

### DS-SW-COMP-044 PII マスキングの呼び出し境界

Log Adapter は出力前に必ず COMP-T1-PII への内部 gRPC 呼び出しで機密情報検出・マスキングを実施する。この呼び出しは同期だが、失敗時は「ログ出力を抑制」ではなく「PII 検出器未通過」の明示ラベルを付けてログを出力し、監視側でアラート発火する（ログ欠落は監査の致命傷のため、マスク前提を優先）。PII 呼び出しの頻度を下げるため、tenant 単位の LRU cache（キー: ログテンプレート、TTL 10 秒）で結果を再利用する。

**確定段階**: リリース時点。**対応要件**: FR-T1-PII-\*、FR-T1-LOG-\*、NFR-G-ENC-\*。

### DS-SW-COMP-045 Audit への非同期書込

全 API 呼び出しの監査イベントは Policy Enforcer 層で生成し、内部 Kafka topic `k1s0.audit.events.v1` に publish する。COMP-T1-AUDIT が同 topic を subscribe してハッシュチェーン永続化する。同期書込ではなく非同期化する理由は、Audit 書込を同期にすると p99 500ms（NFR-B-PERF-001）の予算を超過するためである。非同期化に伴う at-least-once 配信は COMP-T1-AUDIT 側で dedup する（[03_自作Rust領域コンポーネント.md](03_自作Rust領域コンポーネント.md) 参照）。

**確定段階**: リリース時点。**対応要件**: FR-T1-AUDIT-\*、NFR-B-PERF-001、NFR-H-INT-001。

### DS-SW-COMP-046 Decision への同期呼び出し

Policy Enforcer 内の RBAC 評価で、tier1 内部の判定ロジック（例: tier2 ごとの許容操作セット）は COMP-T1-DECISION への内部 gRPC 呼び出しで ZEN Engine に委譲する。同期呼び出しとする理由は、認可判定は結果がレスポンスに直結するためである。p99 1ms 目標（NFR-B-PERF-004）を守るため in-process 並みの呼び出し遅延（< 1ms）が求められ、内部 gRPC ＋ Istio Ambient のオーバヘッドを 500μs 以内に抑える設計とする。

**確定段階**: リリース時点。**対応要件**: FR-T1-DECISION-\*、NFR-E-AC-001〜005、NFR-B-PERF-004。

### DS-SW-COMP-047 Connection Pool と graceful shutdown

各 Pod は backend 接続プールを Pod 起動時に warm up する。Valkey は 10 接続、Kafka Producer は 3 接続、OpenBao は 1 接続（active のみ）、flagd は 2 接続を初期確立する。graceful shutdown は SIGTERM 受信で API Router が新規リクエスト拒否に切替、インフライトリクエスト完了まで最大 30 秒待機、その後 Dapr Adapter → 接続プール順にクローズする。Kubernetes の terminationGracePeriodSeconds は 35 秒に設定する。

**確定段階**: リリース時点。**対応要件**: NFR-A-FT-001、NFR-A-REC-001。

### DS-SW-COMP-048 タイムアウト階層

タイムアウトは「L1 public 30s > L2 pipeline 25s > L3 Dapr 20s > L4 backend 15s」の階層で定める。各階層で 5 秒のバッファを持つことで、外側が先にタイムアウトすることを防ぐ。API ごとの個別設定は Policy Enforcer で override 可能（例: Decision は L1 を 5s、L4 を 1s に短縮、Workflow は L1 を 60s に延長）。context.WithTimeout で階層を Go context に載せ、内部 gRPC 呼び出し時に残時間を `grpc-timeout` メタデータで伝搬する（[../03_内部インタフェース方式設計/01_内部gRPC契約方式.md](../03_内部インタフェース方式設計/01_内部gRPC契約方式.md) 参照）。

**確定段階**: リリース時点。**対応要件**: NFR-B-PERF-001、NFR-A-CONT-\*。

### DS-SW-COMP-049 ヘルスチェックエンドポイント

各 Pod は `/healthz`（liveness、HTTP GET）と `/readyz`（readiness、HTTP GET）を公開する。liveness は API Router プロセス生存で 200 を返す。readiness は 5 項目（Dapr sidecar 到達 / backend 接続確立 / Leader Election 参加 / Dapr Component 読込完了 / Policy 読込完了）が全て満たされたときに 200 を返す。Kubernetes probe は initialDelaySeconds 10s / periodSeconds 5s / timeoutSeconds 3s / failureThreshold 3 で設定する。詳細は [../../50_非機能方式設計/](../../50_非機能方式設計/) に委ねる。

**確定段階**: リリース時点。**対応要件**: NFR-A-FT-001、NFR-D-MON-\*。

## Dapr 隠蔽の 6 段階多層防御統括

ここまで本ファイルはファサード 3 Pod の内部モジュール・パイプライン・Building Block 使い分けを仕様化してきた。しかし Dapr SDK の直接 import を tier2/tier3 に漏らさないための**強制機構**は、本ファイル単独では完結せず、複数の設計ドキュメントに散在している。企画書 L197-212 は「バラつきを防ぐ多層防御」として 6 段階を宣言しているが、概要設計側ではこのうち ①雛形 CLI / ②Opinionated API / ③CI Guard までしか統括されておらず、④リファレンス実装 / ⑤PR チェックリスト / ⑥内製 analyzer の在処が設計書間で散らばっていた。本節は 6 段階を単一ファイルで束ねて、どの段階がどの設計 ID でどの段階で具体化されるかを一望可能にする。

散在を許すと、例えば「tier2 の TypeScript コードがリフレクション経由で `@dapr/dapr` を動的 require する」という典型的逸脱を、CI Guard が静的検出できず、PR チェックリストにも項目がなく、内製 analyzer も未着手のまま リリース時点 をロールアウトしてしまう、という状態が発生する。この状態は採用検討で約束した「Dapr 隠蔽による差し替え可能性」を実質的に崩すため、6 段階の設計 ID を 1 か所で突き合わせて「各段階のカバレッジ」を検証可能な形にする必要がある。

### DS-SW-COMP-141 6 段階多層防御の統括と相互補完

6 段階は機械検出と人的レビューを組み合わせた段階的捕捉構造である。静的言語（Go / Rust / C#）の禁止 import は ①〜④ で機械的に検出でき、動的言語（TypeScript / Python）の動的ロード・リフレクション経由の Dapr SDK 呼び出しや、Opinionated API の型は合うが設計思想を外した使い方は CI 単独では検出しきれず、⑤ の PR レビューで補足し、頻出パターンを ⑥ の内製 analyzer に吸い上げる。完全自動化ではなく段階的捕捉率向上が狙いであり、どの段階も「他段階で検出漏れしたものを捕まえる」補完関係にある。

散文で 6 段階を列挙する。

1. **雛形 CLI**（ゼロから書かせない段階）— tier2/tier3 の新規サービス作成時に雛形 CLI が Opinionated な初期ファイル一式を生成し、開発者が Dapr SDK を直接 import するコードを書き始められないようにする。具体化は本ファイルの [DS-SW-COMP-020〜022](02_Daprファサード層コンポーネント.md) の 5 モジュール構成を生成テンプレート化したもので、詳細は [../../../02_構想設計/02_tier1設計/02_API契約/03_API設計原則.md](../../../02_構想設計/02_tier1設計/02_API契約/03_API設計原則.md)・[06_パッケージ構成_Rust_Go.md](06_パッケージ構成_Rust_Go.md) の DS-SW-COMP-124〜131（internal/ 配下 package 分割）、および [../../70_開発者体験方式設計/04_Backstageポータル詳細方式.md](../../70_開発者体験方式設計/04_Backstageポータル詳細方式.md) の Software Template に分解される。リリース時点で 初版提供 / 全 6 Pod 雛形をカバーする。

2. **Opinionated API**（やり方を 1 通りに絞る段階）— tier1 公開 11 API は Dapr Building Block を 1 対 1 でラップせず、横断的関心事（認証・テナント・監査・観測・エラー変換）を Policy Enforcer で強制するパイプラインを経由する独自 gRPC サービスとして公開する。具体化は本ファイルの [DS-SW-COMP-023〜029](02_Daprファサード層コンポーネント.md)（COMP-T1-STATE 内部）と [../02_外部インタフェース方式設計/](../02_外部インタフェース方式設計/) DS-SW-EIF-001 以降の 11 API 契約である。リリース時点で State/PubSub / Service Invoke/Binding/Workflow / Decision/Feature を完成させる。

3. **CI Guard**（静的言語の禁止 import を機械検出する段階）— Go / Rust / C# の静的言語は GitHub Actions ワークフローで `dapr.io/go-sdk` / `dapr-client` / `Dapr.Client` などの禁止 import パターンを `golangci-lint` / `clippy` / `Roslyn analyzer` で検出し、検出時 PR をブロックする。具体化は [../../70_開発者体験方式設計/01_CI_CD方式.md](../../70_開発者体験方式設計/01_CI_CD方式.md) DS-DEVX-CICD-* の禁止パターン lint 設計である。リリース時点で Go の Dapr SDK 検出から始め / Rust/C# を追加する。

4. **リファレンス実装**（模範サービスを 1 本提供する段階）— tier1 公開 API の正しい使い方を示すサンプル `golden-path-service` を 1 本メンテナンスし、tier2 開発者が模倣対象として参照する。具体化は [../../20_ソフトウェア方式設計/05_利用者文書_暫定版/](../../20_ソフトウェア方式設計/05_利用者文書_暫定版/) 配下の GoldenPath サンプルと [../../70_開発者体験方式設計/](../../70_開発者体験方式設計/) DX-GP-* のゴールデンパス仕様である。リリース時点で Service Invoke + State を使う最小サンプル / PubSub + Workflow を追加したフル構成まで段階的に整備する。

5. **PR チェックリスト**（動的言語・設計思想逸脱を人のレビューで補完する段階）— TypeScript / Python など動的言語の `require('@dapr/dapr')` 動的ロードや `importlib` 経由ロード、および型は合うが設計思想を外した Opinionated API の誤用（例: Policy Enforcer バイパス、Idempotency Key なし State Set 多用）は静的検出困難なため、PR チェックリストを Backstage の Software Template に内蔵し、レビュワーが必ず確認する項目とする。具体化は [../../70_開発者体験方式設計/04_Backstageポータル詳細方式.md](../../70_開発者体験方式設計/04_Backstageポータル詳細方式.md) DS-DEVX-BS-* の Software Template 内蔵 PR チェックリストである。リリース時点で 運用開始 / 動的言語検出項目を拡張する。

6. **内製 analyzer**（採用後の運用拡大時 で初版、以降頻出逸脱を吸い上げて成長する段階）— ⑤ の PR レビューで頻出する逸脱パターン（tier2/tier3 コードに Dapr SDK 直接 import を検出、動的言語のリフレクション経由ロード検出など）を内製の AST 解析ツールに吸い上げ、機械検出の捕捉率を段階的に向上させる。具体化は [../../70_開発者体験方式設計/01_CI_CD方式.md](../../70_開発者体験方式設計/01_CI_CD方式.md) DS-DEVX-CICD-* または [../../70_開発者体験方式設計/05_テスト戦略方式.md](../../70_開発者体験方式設計/05_テスト戦略方式.md) DX-TEST-* 系に位置づけ、採用後の運用拡大時 で初版（0.5 人月想定）、採用側のマルチクラスタ移行時で頻出パターン追加の運用を回す。

6 段階を単一ファイルで統括する理由は、散在すると強制力が検証不能になるためである。段階ごとに所管ファイルが分かれていると「リリース時点 で ①②③ は整ったが ④⑤ が Backstage 未実装のため実質無防備」という状態を見落とし、採用検討で約束した Dapr 隠蔽の保証が実質崩れる。本統括節は四半期ごとの Product Council レビューで 6 段階のカバレッジ表（各段階が該当段階 で機能しているか）を読み合わせる対象とし、いずれかの段階でカバレッジ欠落がある場合は該当段階 のロールアウトをブロックする運用を導入する。

**確定段階**: リリース時点（①②④ / ③⑤）、採用後の運用拡大時（⑥）。**対応要件**: FR-T1-*（tier1 公開 API 全般）、DX-GP-\*、DX-TEST-\*、NFR-C-NOP-\*。**参照**: 企画書 L197-212、構想設計 [../../../02_構想設計/02_tier1設計/02_API契約/03_API設計原則.md](../../../02_構想設計/02_tier1設計/02_API契約/03_API設計原則.md)。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定段階 |
|---|---|---|
| DS-SW-COMP-020 | ファサード層 5 モジュール構成 | リリース時点 |
| DS-SW-COMP-021 | 呼び出しパイプライン固定順序 | リリース時点 |
| DS-SW-COMP-022 | モジュール間 interface と DI | リリース時点 |
| DS-SW-COMP-023 | STATE API Router の endpoint 分割 | リリース時点 |
| DS-SW-COMP-024 | Dapr Building Blocks の使い分け | 採用初期/1c |
| DS-SW-COMP-025 | STATE State API 内部モジュール | リリース時点 |
| DS-SW-COMP-026 | STATE PubSub API 内部モジュール | 採用初期/2 |
| DS-SW-COMP-027 | STATE Feature API 内部モジュール | リリース時点 |
| DS-SW-COMP-028 | STATE Service Invoke API 内部モジュール | リリース時点 |
| DS-SW-COMP-029 | STATE Binding API 内部モジュール | 採用初期/2 |
| DS-SW-COMP-030 | SECRET Leader Election 内部モジュール | リリース時点 |
| DS-SW-COMP-031 | SECRET Secrets API 内部モジュール | リリース時点 |
| DS-SW-COMP-032 | SECRET OpenBao 接続プールと health check | リリース時点 |
| DS-SW-COMP-033 | WORKFLOW Pluggable Backend | リリース時点 |
| DS-SW-COMP-034 | WORKFLOW task queue sticky 方式 | リリース時点 |
| DS-SW-COMP-035 | WORKFLOW Saga 補償トランザクション | リリース時点 |
| DS-SW-COMP-036 | Policy Enforcer の内部構造 | 採用初期/1c |
| DS-SW-COMP-037 | Log Adapter の構造化ログ方式 | リリース時点 |
| DS-SW-COMP-038 | Metrics Emitter の RED メトリック方式 | リリース時点 |
| DS-SW-COMP-039 | daprd サイドカー注入方式 | リリース時点 |
| DS-SW-COMP-040 | Dapr Component YAML 管理方式 | リリース時点 |
| DS-SW-COMP-041 | 4 階層コールスタックの責務境界 | 採用初期 |
| DS-SW-COMP-042 | Dapr バージョンアップ方式 | リリース時点 |
| DS-SW-COMP-043 | Feature Flag による段階的公開 | リリース時点 |
| DS-SW-COMP-044 | PII マスキングの呼び出し境界 | リリース時点 |
| DS-SW-COMP-045 | Audit への非同期書込 | リリース時点 |
| DS-SW-COMP-046 | Decision への同期呼び出し | リリース時点 |
| DS-SW-COMP-047 | Connection Pool と graceful shutdown | リリース時点 |
| DS-SW-COMP-048 | タイムアウト階層 | リリース時点 |
| DS-SW-COMP-049 | ヘルスチェックエンドポイント | リリース時点 |
| DS-SW-COMP-141 | Dapr 隠蔽 6 段階多層防御統括 | 採用初期/2 |

## 対応要件一覧

- FR-T1-INVOKE-\* / FR-T1-STATE-\* / FR-T1-PUBSUB-\* / FR-T1-BINDING-\* / FR-T1-SECRETS-\* / FR-T1-WORKFLOW-\* / FR-T1-FEATURE-\* / FR-T1-LOG-\* / FR-T1-TELEMETRY-\* / FR-T1-DECISION-\* / FR-T1-AUDIT-\* / FR-T1-PII-\*
- NFR-A-CONT-001 / NFR-A-CONT-002 / NFR-A-CONT-003 / NFR-A-FT-001 / NFR-A-REC-001
- NFR-B-PERF-001（tier1 API p99 500ms）/ NFR-B-PERF-002（スループット 150 RPS）/ NFR-B-PERF-003（State Get 10ms）/ NFR-B-PERF-004（Decision 1ms）/ NFR-B-PERF-005（PubSub Publish 50ms）/ NFR-B-PERF-006（Log・Telemetry 計装 10ms）/ NFR-B-PERF-007（Feature Flag 10ms）/ NFR-B-WL-001 / NFR-B-CAP-\*
- NFR-C-NOP-001 / NFR-C-NOP-002
- NFR-E-MON-\* / NFR-D-MTH-\*
- NFR-E-AC-\* / NFR-E-ENC-001 / NFR-E-NW-\* / NFR-E-MON-001
- NFR-F-ENV-\* / NFR-G-ENC-\* / NFR-H-INT-001 / NFR-H-KEY-001
- DX-TEST-\* / DX-GP-\* / DX-FM-\* / DX-MET-\* / DX-RB-\*
- U-WORKFLOW-001（COMP-T1-WORKFLOW の未決事項）

構想設計 ADR-TIER1-001 / ADR-TIER1-002 / ADR-DATA-002 / ADR-MSG-001 / ADR-SEC-002 / ADR-FM-001 / ADR-RULE-002 と双方向トレースする。
