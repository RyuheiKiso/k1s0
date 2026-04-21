# 01. tier1 11 API 方式概要

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応する。tier1 が tier2 / tier3 へ公開する 11 の公開 API を束ねて俯瞰し、各 API の責務・実装 Pod・バックエンド・SLO・共通契約を概要設計視点で固定化する。個別 API の詳細仕様は [06_API別詳細方式/](06_API別詳細方式/) の 11 ファイル（`01_Service_Invoke_API方式.md` ～ `11_Feature_API方式.md`）で個別に採番するため、本ファイルは横断視点に絞る。

## 本ファイルの位置付け

tier1 は 11 個の API を通じて tier2 / tier3 に価値を提供する。この 11 個は構想設計 [ADR-TIER1-001](../../../02_構想設計/02_tier1設計/) でハイブリッド内部言語（Dapr ファサード=Go、自作領域=Rust）として確定し、要件定義 [FR-T1-* 49 件](../../../03_要件定義/20_機能要件/) で機能契約が確立している。本ファイルはその全体像を 1 枚に束ねることで、API 間の位置関係・共通契約の抜け漏れ・SLO の分配の妥当性をアーキテクトが一瞥できる状態を作る。

11 API を個別ファイルだけで記述すると、共通契約（認証 / トレース / 冪等 / エラー）が API ごとに重複する一方で、個別差分の根拠が埋もれる。本ファイルは共通契約を一元化し、個別ファイルが共通契約を参照する構成で記述の重複と不整合を同時に排除する。tier2 / tier3 の開発者視点で「どの API をどのタイミングで使えばよいか」の意思決定を支える俯瞰資料としても機能させる。

## 11 API の俯瞰と責務分担

11 API は「業務ロジックが呼び出す価値抽象」「プラットフォームが与える観測性抽象」「プラットフォームが与える統制抽象」の 3 系統で読み解くと整理しやすい。業務ロジックが呼び出す価値抽象は Service Invoke / State / PubSub / Secrets / Binding / Workflow / Decision / Feature の 8 個、観測性抽象は Log / Telemetry の 2 個、統制抽象は Audit-Pii の 1 個である。価値抽象はアプリから明示的に呼び出され業務が完結する単位、観測性抽象はコンポーネントの挙動を可視化する単位、統制抽象は J-SOX・個人情報保護・改ざん防止の監査証跡を責任持って記録する単位、と読むと各 API の存在理由が線で繋がる。

**設計項目 DS-SW-EIF-001 11 API の責務と実装 Pod・バックエンドの写像**

11 API の責務と実装 Pod・バックエンドは以下表のとおり固定化する。根拠セルは単なる一覧ではなく「なぜこの Pod で、なぜこのバックエンドなのか」を併記している。

| # | API | 責務（何を解決するか） | 実装 Pod | バックエンド | 根拠 |
|---|-----|-----------|-----------|-----------|------|
| 1 | Service Invoke | 同期 gRPC サービス呼び出しと HTTP/1.1 互換プロキシ（.NET Framework 向け）、タイムアウト・リトライ・サーキットブレーカ、認証トークン自動伝搬 | Go（facade-svcinv） | Dapr service-invocation + Istio Ambient mTLS | Dapr SDK stable、mTLS は Ambient が透過提供 |
| 2 | State | Get / Set / Delete / TTL / Bulk（最大 100 キー）/ ETag 楽観ロック / Transaction（最大 10 操作） | Go（facade-state） | Valkey Cluster（Dapr State コンポーネント） | p99 10ms 達成には L1 キャッシュ内完結が必須、Kafka 等では不足 |
| 3 | PubSub | Publish / Subscribe、at-least-once、Consumer Group、DLQ（失敗 3 回で転送）、トピック命名強制 | Go（facade-pubsub） | Kafka（Strimzi、KRaft） | at-least-once と DLQ は Kafka の consumer group と DLT 機能で自然実装 |
| 4 | Secrets | 動的シークレット発行、リース管理、自動ローテーション | Go（facade-secrets） | OpenBao | HashiCorp Vault 互換で BSL 非該当の唯一選択 |
| 5 | Binding | Input / Output binding（MQTT、HTTP、Cron、Kafka） | Go（facade-binding） | Dapr Binding Components | IoT / バッチ / cron の透過化 |
| 6 | Workflow | 長期実行（Temporal）/ 短期（Dapr Workflow）、Saga 補償 | Go（facade-workflow） | Dapr Workflow + Temporal | 短期は Dapr 埋め込み、長期は Temporal に ADR-WF-001 で分離 |
| 7 | Log | 構造化ログ（JSON、ECS 準拠）、PII 自動マスキング | Rust（custom-log） | Grafana Loki（Phase 1b）/ OTLP Collector | PII マスキングは Rust 側で確定的に実装し GDPR 対応 |
| 8 | Telemetry | OpenTelemetry トレース / メトリクス / プロファイル | Go（facade-telemetry） | OTel Collector → Tempo / Prometheus / Pyroscope | OTLP 業界標準、計装負荷は Collector 側集約 |
| 9 | Decision | ZEN Engine 決定評価、p99 1ms | Rust（custom-decision） | ZEN Engine（プロセス内ライブラリ） | JDM 準拠の低レイテンシ、Rust 実装で NUMA 最適 |
| 10 | Audit-Pii | ハッシュチェーン監査（WORM）、PII 自動マスキング | Rust（custom-audit） | PostgreSQL（CloudNativePG）+ MinIO 冷データ | 改ざん防止は crypto 処理の Rust 側で確定実装 |
| 11 | Feature | flagd Feature Flag、テナント別 / パーセント / 段階展開 | Go（facade-feature） | flagd + Postgres 設定ストア | OpenFeature 準拠で将来差し替え可 |

価値抽象（1〜6、9、11 の 8 個）は「何をどのデータで処理するか」が API 単位で決まる。観測性抽象（7、8 の 2 個）は全 API から横串で呼ばれ、計装の漏れを tier1 側で吸収する。統制抽象（10 の 1 個）は tier1 内部が価値 API を呼び出した際に自動で記録する仕組みと、tier2 / tier3 が明示的に記録する仕組みの 2 経路を提供する。

## 共通契約（全 API 横断）

11 API は個別の機能差はあるが、横断的に守るべき契約を共通化することで tier2 / tier3 開発者の学習コストを抑える。共通契約は「認証」「トレース」「テナント伝搬」「冪等」「エラー」「レスポンスヘッダ」の 6 点に集約する。

**設計項目 DS-SW-EIF-002 認証契約の全 API 必須化**

全 API で JWT Bearer Token を必須とする。gRPC metadata の `authorization` ヘッダ、HTTP/JSON の `Authorization: Bearer <jwt>` ヘッダで送付する。トークン未提示・失効・署名検証失敗はすべて gRPC `UNAUTHENTICATED` / HTTP 401 を返し、`WWW-Authenticate: Bearer error="invalid_token"` ヘッダを付与する。匿名アクセスは例外なく禁止し、Phase 1a 段階でも Dev Realm の JWT 発行を前提に開発環境を構成する。Phase 1b 末時点で `spiffe://k1s0.internal/*` の SVID JWT にも対応し、サービス間呼び出しの認証を人間ユーザと分離する。認証詳細は [03_認証認可インタフェース方式.md](03_認証認可インタフェース方式.md) で展開する。

**設計項目 DS-SW-EIF-003 分散トレース契約の全 API 必須化**

全 API で W3C Trace Context の `traceparent` / `tracestate` ヘッダを必須とする。未提示時は tier1 側で新規生成し、生成した trace_id を `X-K1s0-Request-Id` として応答ヘッダに返す。サンプリングは tier1 内部で 100%、tier1 → OTel Collector での head sampling を 10%（本番）/ 100%（Pre / Dev）に設定する。trace_id は Log API / Audit-Pii API に自動伝搬され、1 リクエストに紐付く全イベントを後追い可能にする。

**設計項目 DS-SW-EIF-004 テナント ID 伝搬契約**

全 API で `X-K1s0-Tenant-Id` ヘッダまたは gRPC metadata `k1s0-tenant-id` を必須とする。JWT の `tenant_id` claim と一致しない場合は 403 Forbidden を返す。JWT claim が正とし、ヘッダは明示性確保のためのミラーリングである。tier1 内部で Valkey / Postgres へのアクセス時には必ず WHERE 句または key prefix に tenant_id を含めて RLS / key-scoping を強制する。

**設計項目 DS-SW-EIF-005 冪等キー契約（副作用伴う API）**

副作用を伴う API（Service Invoke の POST 系、State の Set / Delete / Transaction、PubSub の Publish、Binding の Output、Workflow の Start）では `Idempotency-Key` ヘッダまたは gRPC metadata `idempotency-key` を任意で受け付ける。受領した場合、tier1 は Valkey に `<tenant_id>:<api>:<key>` を TTL 24 時間で記録し、同一キーの再送時は初回結果を返す。冪等キー未指定時は全リクエストを独立して処理する。詳細は [../../40_制御方式設計/](../../40_制御方式設計/) の冪等性方式で展開する。

**設計項目 DS-SW-EIF-006 エラー契約（K1s0Error 統一）**

全 API は `k1s0.common.error.v1.K1s0Error` を単一エラー型として返す。フィールドは `code`（列挙）、`message`（人間可読）、`trace_id`、`details[]`（Any 型）の 4 つ。gRPC では `status.Details` として埋め込み、HTTP/JSON では JSON ボディとして返す。エラーコードは [../../30_共通機能方式設計/07_エラーハンドリングとメッセージ方式.md](../../30_共通機能方式設計/07_エラーハンドリングとメッセージ方式.md) で 50 系コード体系（`AUTH_*` / `VALIDATION_*` / `RATE_LIMIT_*` / `BACKEND_*` / `INTERNAL_*`）として採番する。

**設計項目 DS-SW-EIF-007 レスポンスヘッダ契約**

全 API のレスポンスに `X-K1s0-Request-Id`（= trace_id）と `X-K1s0-Api-Version`（= SemVer major.minor、例 `1.3`）を必須で付与する。Request-Id は問い合わせ時の一意識別子、Api-Version は クライアント側でのバージョン検証とメトリクス分類の両方に用いる。加えて 5xx 系では `Retry-After` ヘッダを、429 では `X-K1s0-RateLimit-Reset`（Unix 秒）を付与する。

## 公開方式（gRPC 主、HTTP/JSON 補助）

公開方式は gRPC を一次とし、HTTP/JSON を補助とする。この二層は「性能重視の正規経路」と「既存資産との互換性」を同時に成立させるためである。.NET Framework やレガシー REST クライアントは HTTP/JSON を、Go / Rust / TypeScript / C# (.NET 8) の SDK は gRPC を利用する。

**設計項目 DS-SW-EIF-008 gRPC を一次公開形式とする**

gRPC は HTTP/2 多重化・ストリーミング・型安全性の 3 点で REST に対し優位性を持つ。tier1 の p99 500ms SLO は HTTP/1.1 の head-of-line blocking では達成困難であり、gRPC 採用は SLO 達成の前提条件である。リクエスト / レスポンスは Protobuf binary、メタデータは ASCII で送受信する。サーバサイドストリーミングは Telemetry / Log API でのみ有効化し、他は unary に限定する。

**設計項目 DS-SW-EIF-009 HTTP/JSON を補助公開形式とする（Envoy で変換）**

HTTP/JSON は Envoy Gateway の JSON-gRPC Transcoder で自動変換する。変換マッピングは `.proto` の `google.api.http` オプションで宣言し、SDK 生成時に同一定義から派生させる。HTTP 経由時はエラーを `{"code":"...","message":"...","trace_id":"..."}` として 4xx/5xx とともに返す。.NET Framework では gRPC 公式サポートがないため、HTTP/JSON は正規経路として継続提供する。Telemetry と Log のストリーミングは HTTP/JSON 互換性を持たないため HTTP では未提供とし、ドキュメントで明示する。

**設計項目 DS-SW-EIF-010 エンドポイント命名と経路**

公開ドメインは `api.k1s0.internal.example.jp`、パスは `/grpc/k1s0.public.<api>.v1.<Service>/<Method>`（gRPC over HTTP/2）、および `/v1/<api>/<resource>`（HTTP/JSON）の 2 系統。Envoy Gateway でホスト判定 → Authorization → Tenant 判定 → API ルーティングの順で処理する。クライアント SDK は両方の URL を自動判別する機構を持つ。

## SDK 自動生成

tier2 / tier3 開発者の負担を抑えるため、SDK は `.proto` から自動生成する。手書き SDK は原則禁止する。

**設計項目 DS-SW-EIF-011 buf + protoc-gen で 4 言語 SDK を生成**

対象言語は Go / Rust / TypeScript / C#（.NET 8）の 4 言語。buf CLI を単一ビルドパイプラインとして、`protoc-gen-go` / `protoc-gen-go-grpc` / `tonic-build` / `protoc-gen-ts` / `Grpc.Tools` を通して各言語の SDK を生成する。生成物は `k1s0-sdk-<lang>-v<major>` として Buf Schema Registry（ADR-TIER1-002）もしくは GitHub Packages / NuGet / npm / crates.io 社内ミラーに配信する。バージョンは `.proto` の major バージョン（`k1s0.public.state.v1.*`）と同期する。

**設計項目 DS-SW-EIF-012 .NET Framework 向け SDK の特別対応**

.NET Framework 4.8（既存 JTC 資産の主力）は gRPC 公式サポートがないため、`Grpc.Core`（非推奨だが 4.8 稼働可）の C# レガシー SDK を Phase 1c まで提供する。Phase 2 以降は HTTP/JSON SDK（`k1s0-sdk-netfx-http`）に一本化し、段階的に gRPC 依存を廃止する。この方針は [05_レガシー連携インタフェース方式.md](05_レガシー連携インタフェース方式.md) と連動する。

## SLA / SLO の分配（p99 ms 目標）

構想設計で約束した「tier1 API p99 500ms」は 11 API 全体の上限値であり、各 API はこの上限から役割に応じて内訳値を割り当てる。業務応答系（Service Invoke / State / PubSub）は低レイテンシ重視、バックグラウンド系（Workflow / Binding）は throughput 重視、統制系（Audit-Pii）は永続化完了を優先、観測性系（Log / Telemetry）は非同期化で本線に影響しない、という役割分担で数値を決める。

**設計項目 DS-SW-EIF-013 API 別 p99 レイテンシ目標**

| API | p99 目標 | 根拠 |
|-----|---------|------|
| Service Invoke | 300ms | 業務ロジック 200ms + Dapr 80ms + NW 20ms の積算 |
| State Get | 10ms | Valkey Cluster 内 1 ホップ + シリアライズ |
| State Set / Delete | 20ms | Valkey Cluster 内レプリカ同期 + WAL |
| PubSub Publish | 50ms | Kafka acks=all + ISR 同期 |
| Secrets | 100ms | OpenBao AppRole + リース発行 |
| Binding Output | 200ms | 外部エンドポイント次第、Output は best-effort 計測 |
| Workflow Start | 100ms | Temporal StartWorkflow or Dapr Workflow Start |
| Log | 5ms（非同期エンキュー） | 本線は非ブロック、OTel Collector 側で永続化 |
| Telemetry | 5ms（非同期エンキュー） | 同上 |
| Decision | 1ms | ZEN Engine プロセス内評価（JDM 準拠）|
| Audit-Pii | 30ms | Postgres WAL commit + hash chain 計算 |
| Feature | 5ms | flagd インメモリ評価 |

各数値は構想設計で個別 ADR 化しており、概要設計では ADR 値を転記する方針とする。数値超過が実測で発生した場合は ADR 改訂を要する。

**設計項目 DS-SW-EIF-014 スループット目標とキャパシティ計画**

Phase 1b の目標スループットは tier1 全体で 2,000 RPS（Service Invoke + State + PubSub の合算ピーク）とする。Valkey Cluster は 10,000 ops/sec、Kafka は 5,000 msg/sec、Postgres は 1,000 tx/sec を個別に担保する。負荷分配とスケール単位は [../../50_非機能方式設計/06_キャパシティ計画.md](../../50_非機能方式設計/06_キャパシティ計画.md) で展開する。

## 可観測性とリクエスト相関

全 API は 1 リクエスト単位で trace_id / tenant_id / user_id / api_name / method / status を必ずログ出力し、Log API と Audit-Pii API に自動ブリッジする。この相関は「運用 2 名原則」の前提であり、障害時に tier2 開発者に質問することなく tier1 SRE が原因箇所を特定できることが合格基準となる。

**設計項目 DS-SW-EIF-015 共通計装フィールドの標準化**

全 API の計装ログは `@timestamp` / `trace_id` / `tenant_id` / `user_id` / `api_name` / `method` / `status` / `latency_ms` / `error_code`（エラー時） / `request_id` の 10 フィールドを ECS 準拠で必須出力する。フィールド追加は許容するが削除は破壊的変更として禁止する。詳細は [../../30_共通機能方式設計/](../../30_共通機能方式設計/) のログ方式で展開する。

## フェーズ別公開順序

tier2 / tier3 開発者への公開は段階的に行う。一括公開するとテスト網羅性が破綻し、バグがあった場合に全面ロールバックとなるリスクを避ける。

**設計項目 DS-SW-EIF-016 API 公開のフェーズ順**

Phase 1a（MVP-0）は Service Invoke / State / PubSub / Feature / Log の 5 API を限定公開する。Phase 1b（MVP-1a）は Secrets / Binding / Workflow（Dapr Workflow のみ）/ Telemetry / Decision / Audit-Pii を追加し 11 API 完備状態に到達する。Phase 1c（MVP-1b）は Workflow の Temporal バックエンドを有効化し、長期実行対応を完了する。Phase 2 以降は既公開 API の機能拡張（Bulk 上限引き上げ、ストリーミング拡張）のみで、新規 API 追加は設計方針 [../../00_設計方針/02_設計原則と制約.md](../../00_設計方針/02_設計原則と制約.md) の原則 7 に従い ADR 起票を要する。

## 対応要件一覧

本ファイルは tier1 11 API 公開インタフェースの俯瞰設計であり、以下の要件 ID に対応する。

- FR-T1-001〜FR-T1-049（tier1 11 API 全機能要件、49 件）
- FR-EXT-DOTNET-001（.NET Framework クライアント互換）
- FR-EXT-IDP-001（Keycloak JWT 認証統合）
- FR-EXT-MON-001（監視基盤連携）
- NFR-E-SEC-001〜003（全 API 認証必須）
- NFR-B-PERF-001〜005（p99 レイテンシ目標）
- ADR 参照: ADR-TIER1-001（Go+Rust 分担）/ ADR-TIER1-002（Protobuf gRPC 必須）
- 本ファイルで採番: DS-SW-EIF-001 〜 DS-SW-EIF-016
