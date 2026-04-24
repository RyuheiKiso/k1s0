# 00. tier1 API 共通規約

本書は tier1 が公開する 11 API（Service Invoke / State / PubSub / Secrets / Binding / Workflow / Log / Telemetry / Decision / Audit-Pii / Feature）に横断で適用される共通契約を一元化する。各 API ファイル（01〜11）の個別要件は本書の遵守を前提に記述されており、11 ファイルで同じ説明を繰り返さない。

## 本書の位置付け

tier1 公開 API は形式が揃っていなければ tier2/tier3 から「どの API は idempotency-key を送ればよいか」「どの API は 500 を返すか」を都度確認する必要が生じ、SDK ラッパの品質が発散する。共通規約を明示することで、個別 API の要件記述を「その API 固有の振る舞い」だけに絞り、レビュー負荷と実装ブレを同時に抑える。

本書の対象は「tier2/tier3 から観測できる契約面」に限定する。バックエンド OSS（Dapr / OpenBao / ZEN Engine 等）の内部仕様は構想設計（[../../02_構想設計/02_tier1設計/](../../02_構想設計/02_tier1設計/)）で扱う。

## Dapr 互換性マトリクス

tier1 公開 API は Go ファサード層で Dapr SDK を内部的に使用する API と、k1s0 独自実装の API が混在する。Dapr バージョンアップ時の互換性判断、および tier2/tier3 から「どの挙動が Dapr 由来か」を監査可能にするため、各 API の依存関係と差分を以下に一元化する。Dapr 側で破壊的変更が入った際は本表の「Dapr 依存レベル」列から影響範囲を逆引きする。

| tier1 公開 API | Dapr Building Block | Dapr 依存レベル | 差分（k1s0 独自拡張） | バックエンド | 破壊的変更の検知責務 |
|---|---|---|---|---|---|
| Service Invoke | Service Invocation | **強依存** | 認証トークン自動伝搬、HTTP/1.1 プロキシ（Dapr 非対応）、Dapr Resiliency Policy の既定値固定 | Istio Ambient mTLS + k1s0 Router | 契約テスト（CI）で Dapr minor アップデート時に回帰検知 |
| State | State Management | **強依存** | ETag 必須化（Dapr は任意）、tenant_id プレフィックス自動付与、Valkey/PostgreSQL 切替の透過化 | Valkey、PostgreSQL (CloudNativePG) | ETag 挙動差異を integration test で検知 |
| PubSub | Pub/Sub | **強依存** | Topic 名テナント分離、Kafka ACL 連携、at-least-once 保証の明示化 | Apache Kafka (Strimzi) | Kafka プロトコル変更 + Dapr 版数差異を dual check |
| Secrets | Secrets Management | **弱依存** | OpenBao Dynamic Secret 専用（Dapr Local File store は非対応）、キャッシュ TTL 管理を k1s0 側で実装 | OpenBao | OpenBao API 変更が主、Dapr は薄ラッパ |
| Binding | Bindings | **弱依存** | 宛先 URL allowlist 強制、Egress Gateway 経由必須、SSRF 防御（Dapr にない） | 外部 HTTP / SMTP / MinIO | Dapr Binding 実装差異は年次で棚卸し |
| Workflow | Workflow（短期: Dapr Workflow / 長期: Temporal） | **弱依存** | 短期実行は Dapr Workflow、長期実行 Saga は Temporal を利用し、tier2 には単一 API を提示する | Dapr Workflow（Valkey） / Temporal（Temporal PostgreSQL） | ADR-RULE-002 と契約テストで両実行系の振る舞い差分を継続検証 |
| Log | — | **非依存（k1s0 独自）** | Dapr には Logging Building Block がない。k1s0 が OpenTelemetry Collector 経由で Loki へ直接配送 | OTel Collector → Loki | 依存なし。OTel 仕様変更のみ追跡 |
| Telemetry | — | **非依存（k1s0 独自）** | Dapr Observability の上位互換。OTel Collector で span/metrics を Tempo/Mimir に配送 | OTel Collector → Tempo/Mimir | 依存なし |
| Decision | — | **非依存（k1s0 独自）** | ZEN Engine（Rust in-process）を Go から呼出し、gRPC over Unix socket で IPC | ZEN Engine | ZEN Engine バージョン追従のみ |
| Audit-Pii | — | **非依存（k1s0 独自）** | hash_chain 改ざん検知、PII 不可逆マスキング、WORM 永続化は Dapr 範囲外 | Kafka → MinIO Object Lock | 依存なし |
| Feature | — | **非依存（k1s0 独自）** | flagd バックエンドを k1s0 が独自ラップ。OpenFeature SDK 準拠 | flagd | OpenFeature / flagd 仕様変更のみ追跡 |

**Dapr 依存レベルの定義**:

- **強依存**: Dapr SDK の API を直接呼出し、挙動が Dapr に大きく左右される。Dapr minor アップデートで回帰テスト必須。
- **弱依存**: Dapr は薄ラッパで、挙動の大部分は k1s0 独自実装。Dapr アップデートの影響は限定的。
- **非依存**: Dapr を使わず k1s0 が独自実装。バックエンド OSS の仕様変更のみ追跡。

Phase 1a 着手時に本表の「破壊的変更の検知責務」列に基づき、Dapr の minor バージョンアップデートを CI の契約テスト対象として組み込む。本表は新規 API 追加時・Dapr メジャーアップデート時に L2 Platform Team が必ず更新する。

## 公開 API と内部コンポーネントの収容マトリクス

tier1 の内部実装は tier2/tier3 から不可視のため本書では契約面のみを扱うが、11 公開 API が tier1 内部のどの Pod に収容されるかを以下のマトリクスで示す。内部 Pod の分割根拠（なぜ 6 Pod なのか、なぜその言語・運用形態なのか）は構想設計（[../../../02_構想設計/02_tier1設計/01_設計の核/03_内部コンポーネント分割.md](../../../02_構想設計/02_tier1設計/01_設計の核/03_内部コンポーネント分割.md)）で扱う。本マトリクスは tier2 開発者・監査担当・稟議決裁者が「どの Pod が自分の API を実装しているか」「なぜその Pod に集約されるのか」を逆引きする入口として機能し、全体構成図（[../../../01_企画/全体構成図.md](../../../01_企画/全体構成図.md) Row 3 内部 Pod インベントリ）に列挙される 6 Pod が要件 ID から機械的に辿れる状態を保証する。

| 公開 API | 収容 Pod (COMP-T1-\*) | 言語 / 運用形態 | 主要バックエンド |
|---|---|---|---|
| Service Invoke (FR-T1-INVOKE-001〜005) | COMP-T1-STATE | Go / Deployment+HPA 3–10 / Stateless | Istio Ambient mTLS 経由の tier2 間 gRPC |
| State (FR-T1-STATE-001〜005) | COMP-T1-STATE | 同上 | Valkey Cluster (Dapr state.valkey) |
| PubSub (FR-T1-PUBSUB-001〜005) | COMP-T1-STATE | 同上 | Apache Kafka (Dapr pubsub.kafka) |
| Secrets (FR-T1-SECRETS-001〜004) | COMP-T1-SECRET | Go / Leader-Elected (active 1 / standby 2) | OpenBao (動的 Secret / PKI / Transit) |
| Binding (FR-T1-BINDING-001〜004) | COMP-T1-STATE | Go / Deployment+HPA 3–10 | 外部 HTTP / SMTP / MinIO |
| Workflow (FR-T1-WORKFLOW-001〜005) | COMP-T1-WORKFLOW | Go / Deployment 固定 3 replica / sticky | Dapr Workflow（Valkey） + Temporal（Temporal PostgreSQL） |
| Log (FR-T1-LOG-001〜004) | Pod 化しない（SDK + OTel Collector） | — | OpenTelemetry Collector → Loki |
| Telemetry (FR-T1-TELEMETRY-001〜004) | Pod 化しない（SDK + OTel Collector） | — | OpenTelemetry Collector → Tempo / Mimir |
| Decision (FR-T1-DECISION-001〜004) | COMP-T1-DECISION | Rust / Deployment+HPA 2–8 / in-mem | ZEN Engine in-process（外部ストア無し） |
| Audit (FR-T1-AUDIT-001〜003) | COMP-T1-AUDIT | Rust / StatefulSet / PVC per replica | PostgreSQL WORM append-only + MinIO Object Lock |
| Pii (FR-T1-PII-001〜002) | COMP-T1-PII | Rust / Deployment+HPA 2–6 / sidecar-free | なし（純関数、設定は Decision 経由） |
| Feature (FR-T1-FEATURE-001〜004) | COMP-T1-STATE（現行） | Go / Deployment+HPA 3–10 | flagd |

Log と Telemetry の 2 API は専用 Pod を持たない。tier1 SDK 内で構造化ログ生成と OTel span 生成を行い、配送はインフラ層の OpenTelemetry Collector DaemonSet に委譲する。NFR-B-PERF-006（計装オーバヘッド < 10ms）を守るための帰結であり、tier1 アプリ層に中継 Pod を挟むと計装自体のレイテンシが広がる。Feature API は現行 COMP-T1-STATE 同居としているが、flagd 以外の OSS に切替わる場合は COMP-T1-FEATURE 独立 Pod への分離判定が走る（構想設計ファイル「決定を見直す条件」参照）。

本マトリクスは公開 API 側の契約を変えないため、構想設計の 6 Pod 構成が改訂された場合でも 11 API の要件 ID は維持される。双方向トレース（要件 → Pod、Pod → 要件）の担保は [../../80_トレーサビリティ/03_構想設計マトリクス.md](../../80_トレーサビリティ/03_構想設計マトリクス.md) の「tier1 内部コンポーネント（COMP-T1-\*）」節で行う。

## HTTP/JSON 互換インタフェース共通仕様

gRPC を使えない tier2/tier3（.NET Framework サイドカー等のレガシー共存経路、FR-EXT-DOTNET-001）のため HTTP/JSON 互換エンドポイントを提供する。以下は全 API 共通の HTTP レイヤ仕様であり、個別 API ファイルではこれを前提に「どの RPC が HTTP 互換対象か」のみを記述する。

- **URL パス**: `POST /k1s0/<api>/<rpc>`（例: `POST /k1s0/state/get`、`POST /k1s0/pubsub/publish`）
- **Content-Type**: リクエスト・レスポンスとも `application/json; charset=utf-8`
- **JSON スキーマ**: Protobuf message を `protojson`（Google 公式 JSON マッピング）で直列化。`oneof` は最初にセットされたフィールドのみ、`enum` は文字列表現、`Timestamp` は RFC 3339、`bytes` は base64 エンコード
- **認証**: `Authorization: Bearer <jwt>` ヘッダ必須、gRPC と同じ Keycloak 検証
- **trace 伝播**: `traceparent` / `tracestate` ヘッダ必須（W3C Trace Context）
- **idempotency**: 書込み系は `X-K1s0-Idempotency-Key` ヘッダで指定（UUID v7 推奨）
- **HTTP Status ↔ K1s0Error マッピング**:

| HTTP Status | K1s0Error カテゴリ | 再試行 |
|---|---|---|
| 200 | （成功）| — |
| 400 | InvalidArgument | 不可 |
| 401 | Unauthenticated | 不可（トークン再取得後に再試行）|
| 403 | PermissionDenied | 不可 |
| 404 | NotFound | 不可 |
| 409 | AlreadyExists / Conflict | 条件付き（ETag 更新後）|
| 429 | ResourceExhausted | 可（`Retry-After` ヘッダで指定）|
| 503 | Unavailable | 可（`Retry-After` ヘッダで指定、指数バックオフ）|
| 504 | DeadlineExceeded | 可 |
| 500 | Internal | 不可（ポストモーテム対象）|

レスポンス Body には gRPC の `ErrorDetail` と同内容の JSON を格納する（`code`、`message`、`retry_after_ms`、`trace_id`）。

HTTP/JSON は **gRPC の代替であり、新規 API 設計の優先路ではない**。新規 tier2/tier3 は gRPC を使用し、HTTP/JSON は .NET Framework サイドカー経由の旧システム連携に限定する。

## 通信プロトコルと可観測性

tier1 公開 API は Protobuf 定義の gRPC をカノニカル通信路とする（ADR-TIER1-002）。HTTP/JSON は .NET Framework サイドカー等のレガシー共存経路に限定したフォールバックであり、新規 tier2/tier3 は gRPC を優先する。

すべての呼び出しは以下を強制する。

- W3C Trace Context（`traceparent` / `tracestate`）をヘッダで継承・生成。tier1 ファサードで必ず 1 span 発行
- Prometheus メトリクス `k1s0_tier1_<api>_requests_total{tenant_id,method,code}` と `_duration_seconds{tenant_id,method}` を自動発行
- Log API への構造化ログは `trace_id` / `span_id` / `tenant_id` / `user_id` を必須フィールドとして自動注入

可観測性の SLI 計算はこの自動計装を前提とする（NFR-I-SLI-001 および [../../30_非機能要件/I_SLI_SLO_エラーバジェット.md](../../30_非機能要件/I_SLI_SLO_エラーバジェット.md)）。

## 認証と認可

tier1 API への匿名アクセスは禁止する（NFR-E-AC-001）。tier2/tier3 の呼び出しは以下 2 系統のいずれかで認証される。

- **エンドユーザー文脈**: Keycloak 発行の JWT を gRPC メタデータ `authorization: Bearer <jwt>` に付与。`sub`、`tenant_id`、`roles` クレームを tier1 が検証
- **ワークロード文脈**: SPIFFE ID（`spiffe://<trust-domain>/ns/<ns>/sa/<sa>`）を Istio Ambient の mTLS で検証（NFR-E-AC-003、ADR-SEC-003）

`TenantContext`（[40_tier1_API契約IDL/00_共通型定義.md](../40_tier1_API契約IDL/00_共通型定義.md)）の `tenant_id` は呼び出し側が自己宣言するのではなく、tier1 が JWT / SPIFFE ID から導出して上書きする。クライアントから渡された値と不一致な場合は `K1s0Error.PermissionDenied` で即拒否する。

認可は RBAC（Keycloak Role）を基本とし、細粒度の業務ルールは Decision API（ZEN Engine）に委譲する。tier1 は呼び出し時に「認可されているか」を判定し、Audit API に認可成功・失敗を記録する（NFR-E-MON-001）。

## エラー型 `K1s0Error`

tier2/tier3 から観測されるエラーは統一型 `K1s0Error` のみとする。Dapr / OpenBao / Kafka 等のバックエンド固有エラー文字列を表層に漏らすことは禁止する。契約は [40_tier1_API契約IDL/00_共通型定義.md](../40_tier1_API契約IDL/00_共通型定義.md) 内の `ErrorDetail` に集約する。

エラーカテゴリは以下 8 種で充足することを必達とし、未分類エラーは `Internal` に集約する。

- `InvalidArgument` — 呼出側の入力誤り。リトライ不可（`retry_after_ms` は無視される）
- `Unauthenticated` — JWT 不在・署名不正・期限切れ
- `PermissionDenied` — テナント越境、RBAC 拒否、allowlist 外
- `NotFound` — キー / バージョン / リソース未存在
- `AlreadyExists` / `Conflict` — ETag 不一致、冪等性キー衝突
- `ResourceExhausted` — レート制限、クォータ超過（NFR-E-NW-004）。`retry_after_ms` 必須
- `Unavailable` — 一時的なバックエンド不能。`retry_after_ms` 必須、指数バックオフ推奨
- `Internal` — tier1 バグ、未分類。Audit に Severity 2 で記録

`retry_after_ms` は再試行可能エラー（`ResourceExhausted` / `Unavailable`）でのみ意味を持つ。`InvalidArgument` / `PermissionDenied` では呼出側が再試行しても同じ結果となるため、SDK は自動再試行を禁止する。

## 冪等性と再試行

状態変更を伴う API（State.Set / PubSub.Publish / Workflow.Start / Secrets.Rotate / Binding.Send / Audit.Write）は冪等性キー（`idempotency_key`）を受け付ける。同一キーでの再試行は副作用を重複させず、初回と同じレスポンスを返すことを MUST とする。キー保管は Valkey で最短 24 時間（契約上は 1 時間で十分とし、余裕を持たせる）。

冪等性キー未指定の書込は一度きりの実行を意味し、再試行の責任は呼出側にある。SDK は既定で書込系には UUIDv7 を自動生成し付与する。

読取系（State.Get / Secrets.Get / Decision.Evaluate / Feature.Evaluate）は副作用がないため冪等性キーを受け付けない。

自動再試行の方針は以下を既定とする。

- クライアント SDK: `Unavailable` / `ResourceExhausted` に対して指数バックオフ（初回 100ms、最大 3 回、jitter ±20%）
- 上記以外のエラーは即座に呼出側へ返却
- `DeadlineExceeded` はサーバ到達前のキャンセルを示すため、副作用は発生していないとみなして再試行可能

## タイムアウトとデッドライン伝播

gRPC デッドラインは呼出側が指定し、tier1 は受信したデッドラインをバックエンド呼出に伝播する。tier1 内部でのデフォルトは 3 秒、Workflow.Start は 30 秒、Binding.Send（外部 HTTP）は 10 秒とする。デッドライン超過時は `Unavailable` ではなく `DeadlineExceeded` を返し、Audit には成否不明として記録する（NFR-E-MON-001 の双方向記録要件）。

## マルチテナント分離

全 API は `tenant_id` を水平分離キーとして扱う。tier1 は以下の 3 層でテナント越境を防ぐ（NFR-E-AC-003）。

- **L1（入口）**: JWT / SPIFFE ID から `tenant_id` を導出し、リクエストの `TenantContext` を上書き
- **L2（ルーティング）**: バックエンドのキー / トピック / バケット / パーティションに `<tenant_id>/` を自動付与
- **L3（監査）**: Audit API は `tenant_id` をクエリ索引に持ち、他テナントのレコード閲覧を拒否

L2 のキー空間分離は tier2/tier3 から不可視。tier2 が `SetState("foo", ...)` と呼んだ場合、物理キーは `<tenant_id>/foo` になる。

## バージョニングと後方互換

IDL の `package` は `k1s0.tier1.<api>.v<N>` で採番する。以下を MUST 契約とする。

- Phase 1〜2 の期間は v1 系で後方互換破壊を発生させない
- 新フィールドは optional で追加。既存フィールドの削除は DEPRECATED 期間を最短 2 四半期経過させた後 v2 で削除
- `enum` 値の追加は可、既存値の削除は不可
- 破壊的変更は v2 を並列提供し、v1 は 1 年間並走させる（OR-EOL-003）

tier2/tier3 は SDK の semver メジャー番号と IDL の v 番号を対応付けて管理する。Phase 1〜2 で v1 → v2 への強制移行は想定しない。

## レート制限とクォータ

全 API はテナント単位の RPS 上限と同時接続数上限を受ける（NFR-E-NW-004）。既定値は以下を採用し、契約プランに応じて [60_事業契約/06_課金メータリング.md](../../60_事業契約/06_課金メータリング.md) で調整する。

- Free / Trial: 50 RPS、同時接続 10
- Standard: 500 RPS、同時接続 100
- Enterprise: 5,000 RPS、同時接続 1,000（超過時は個別交渉）

超過時は `ResourceExhausted` を `retry_after_ms` 付きで返す。バーストは 2 倍まで 10 秒間許容する（token bucket）。

## 監査と痕跡

特権操作（Secret 取得・ローテーション、Decision 評価、Feature Flag 変更、Binding 外部送信、Audit 書込、Workflow 状態変更）は自動的に Audit API にイベントを発行する。呼出側が明示的に Audit を呼ぶ必要はない。Audit イベントには以下を必須とする。

- `trace_id` / `span_id`（トレース連携）
- `tenant_id` / `actor`（JWT `sub` または SPIFFE ID）
- `action` / `resource` / `result`（成功/失敗/拒否）
- `previous_hash` / `event_hash`（改ざん検知のハッシュチェーン）

tier2/tier3 から Audit API を直接呼ぶ場合は、上記必須フィールドを SDK ヘルパが自動付与する。

## 参照

- [40_tier1_API契約IDL/](../40_tier1_API契約IDL/) — 本規約を Protobuf に実装
- [50_tier1_APIシーケンス.md](../50_tier1_APIシーケンス.md) — 共通契約の呼出シーケンス
- [../../30_非機能要件/E_セキュリティ.md](../../30_非機能要件/E_セキュリティ.md) — NFR-E-AC-001〜005 / NFR-E-MON-001〜004 / NFR-E-NW-004
- [../../30_非機能要件/I_SLI_SLO_エラーバジェット.md](../../30_非機能要件/I_SLI_SLO_エラーバジェット.md) — SLI 計測と SLO 目標
- ADR-TIER1-001 / ADR-TIER1-002 / ADR-TIER1-003
