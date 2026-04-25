# 00. API 共通規約方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応し、[親ファイル 01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001〜016 で宣言した横断契約を「どう実装するか」「どう検証するか」まで深掘りする。11 API の個別方式ファイル（`01_Service_Invoke_API方式.md` ～ `11_Feature_API方式.md`）はすべて本ファイルを参照する前提で記述し、共通契約の重複記述を排除する。

## 本ファイルの位置付け

親ファイルは共通契約の宣言に留まり、実装詳細（Protobuf メッセージ形状、HTTP ヘッダ名の厳密マッピング、エラーコードの enum 値、gRPC / HTTP ステータスコードの写像、SDK 側 interceptor の責務分担、contract test のゲート構成）までは踏み込まない。この深掘りが欠けると個別 API ファイルで同じ契約を微妙に異なる表現で重複定義し、SDK 実装者と tier2 / tier3 利用者の双方に混乱を生む。本ファイルは共通契約の単一情報源（Single Source of Truth）として位置付け、個別 API ファイルは API 固有の差分と SLO 配分だけに集中させる構成とする。

共通契約の単一情報源化は「API ごとの学習コストを 1 時間以内に収める」という DevEx 要件（NFR-DEVEX-001）の前提でもある。利用者は本ファイルを 1 度読めば認証・トレース・テナント伝搬・冪等・エラー・ヘッダ・タイムアウト・Rate Limit・サイズ上限の 9 項目を全 API で再利用でき、個別 API ファイルでは差分のみ追加学習すればよい構造を作る。

## gRPC metadata の実装形式

gRPC は metadata を ASCII key-value で運ぶ。親ファイル DS-SW-EIF-002〜005 で宣言した共通ヘッダ群は gRPC metadata としてどのキー名・どのエンコーディングで送るかを厳密に固定しないと、SDK 生成時と受信側パースで乖離する。tier1 は小文字スネーク形式を厳守し、Protobuf アノテーション（`grpc.gateway.protoc_gen_openapiv2`）と整合させる。

**設計項目 DS-SW-EIF-200 gRPC metadata キー名と値形式**

gRPC metadata のキー名は以下のとおり固定する。根拠セルは「なぜそのキー名か / なぜその値形式か」を併記する。

| metadata キー | 必須/任意 | 値形式 | 根拠 |
|--------------|----------|--------|------|
| `authorization` | 必須 | `Bearer <jwt>` | HTTP 標準 RFC 6750 と完全一致、Envoy JWT Filter が同一キーを参照 |
| `traceparent` | 必須（未送信時は tier1 で生成） | W3C Trace Context v1 準拠 `00-<trace_id>-<span_id>-<flags>` | OTel 業界標準、Envoy AccessLog / Tempo が同一キーを前提 |
| `tracestate` | 任意 | W3C 準拠 `k1s0=<vendor_state>` | 親ファイル DS-SW-EIF-003 と整合 |
| `k1s0-tenant-id` | 必須 | `t` プレフィックス + 4 桁通番（例 `t0001`） | 親ファイル DS-SW-EIF-004、JWT claim と厳密照合 |
| `k1s0-correlation-id` | 任意（業務相関用） | ULID 26 文字 | trace_id とは別に業務起点で連結する用途、ULID は時系列ソート可能 |
| `idempotency-key` | 副作用 API で任意 | UUID v4 または ULID、最大 128 byte | 親ファイル DS-SW-EIF-005、長さ上限は Valkey key サイズ制約 |
| `k1s0-api-version` | 任意 | SemVer `<major>.<minor>`（例 `1.3`） | SDK 側バージョン判定、レスポンスヘッダと対称 |

gRPC metadata のキー名はすべて小文字スネーク形式に正規化する。gRPC HTTP/2 仕様ではヘッダ名は大文字小文字不区別だが、実装間の相互運用性を最大化するため小文字固定で送信する。バイナリ値（`-bin` サフィックス）は本 tier では使用しない。

## HTTP/JSON ヘッダマッピング

親ファイル DS-SW-EIF-009 で HTTP/JSON は Envoy JSON-gRPC Transcoder で自動変換すると宣言した。ヘッダマッピングは Transcoder の既定変換に委ねるだけでは不十分で、tier1 固有ヘッダ（`k1s0-tenant-id` 等）の写像を明示的に固定する必要がある。未固定だと Transcoder が独自命名を生成し、HTTP クライアントと gRPC クライアントで契約が乖離する。

**設計項目 DS-SW-EIF-201 HTTP/JSON ヘッダと gRPC metadata の双方向写像**

Envoy Gateway の `grpc_json_transcoder` フィルタで以下の双方向写像を設定する。写像テーブルは `.proto` の `google.api.http` オプションと一緒に `buf` で管理し、SDK 生成時に同一定義から派生する。

| HTTP/JSON ヘッダ | gRPC metadata | 変換方向 | 備考 |
|-----------------|---------------|---------|------|
| `Authorization: Bearer <jwt>` | `authorization: Bearer <jwt>` | 双方向 | 完全一致、大文字小文字のみ差異 |
| `traceparent` | `traceparent` | 双方向 | 同名 |
| `tracestate` | `tracestate` | 双方向 | 同名 |
| `X-K1s0-Tenant-Id` | `k1s0-tenant-id` | 双方向 | X-K1s0 プレフィックスは HTTP 慣用、gRPC は小文字スネーク |
| `X-K1s0-Correlation-Id` | `k1s0-correlation-id` | 双方向 | 同上 |
| `Idempotency-Key` | `idempotency-key` | 双方向 | IETF draft-ietf-httpapi-idempotency-key との整合 |
| `X-K1s0-Api-Version` | `k1s0-api-version` | 双方向 | レスポンスヘッダ命名と対称 |

リクエスト時は HTTP → gRPC 方向で Envoy が変換し、レスポンス時は gRPC → HTTP 方向で変換する。写像漏れのヘッダは転送されないため、新規ヘッダ追加時は本表の更新と Envoy 設定変更を同時に ADR 化することを運用規約とする。

## K1s0Error Protobuf 型定義

親ファイル DS-SW-EIF-006 で K1s0Error を統一エラー型として宣言したが、フィールド詳細・enum 値・Any 型 details[] の使い方までは未定義だった。個別 API で都度定義すると enum 値が衝突し SDK 間で互換性が崩れる。

**設計項目 DS-SW-EIF-202 K1s0Error Protobuf 型の厳密定義**

`k1s0.common.error.v1.K1s0Error` メッセージを以下のとおり確定する。Protobuf 定義は `protos/k1s0/common/error/v1/error.proto` に配置し、全 API の `.proto` がインポートする。

```protobuf
// K1s0 共通エラー型。gRPC status.Details および HTTP/JSON ボディで返す
message K1s0Error {
  // エラーコード列挙（DS-SW-EIF-203 参照）
  ErrorCode code = 1;
  // 人間可読のメッセージ。英語基底、日本語は details[] に LocalizedMessage で同梱
  string message = 2;
  // W3C Trace Context の trace_id（16 byte hex 32 文字）
  string trace_id = 3;
  // 詳細情報（LocalizedMessage / FieldViolation / RetryInfo 等を Any で同梱）
  repeated google.protobuf.Any details = 4;
  // 発生時刻（RFC 3339、UTC）
  google.protobuf.Timestamp occurred_at = 5;
  // リクエスト ID（correlation_id、存在する場合）
  string correlation_id = 6;
}
```

`details[]` には Google API Design Guide 準拠の `google.rpc.LocalizedMessage` / `google.rpc.BadRequest.FieldViolation` / `google.rpc.RetryInfo` / `google.rpc.QuotaFailure` / `google.rpc.ErrorInfo` を格納する。SDK 側は Any を型判定で展開し、言語固有の例外クラスに変換する。

## 共通エラーコード体系

親ファイル DS-SW-EIF-006 で 5 系統（`AUTH_*` / `VALIDATION_*` / `RATE_LIMIT_*` / `BACKEND_*` / `INTERNAL_*`）を宣言したが、enum 値の具体列挙は未定義だった。API ごとに独自追加されると enum 空間が衝突し、Buf Schema Registry の breaking change 検出で弾かれる。

**設計項目 DS-SW-EIF-203 ErrorCode enum の予約語空間**

ErrorCode enum は以下の予約語空間で運用する。親ファイル DS-SW-EIF-006 で宣言した 5 系統に対し、各系統 100 番ずつの番号空間を割り当て、合計 500 値まで拡張可能な設計とする。

| 系統 | 番号範囲 | 例（enum 値） | 根拠 |
|------|---------|-------------|------|
| 未分類 | 0 | `ERROR_CODE_UNSPECIFIED` | Protobuf 既定値必須 |
| 認証/認可 | 1000〜1099 | `AUTH_TOKEN_MISSING` / `AUTH_TOKEN_INVALID` / `AUTH_TOKEN_EXPIRED` / `AUTH_TENANT_MISMATCH` / `AUTH_PERMISSION_DENIED` | JWT 検証失敗 5 分類で UX 改善 |
| 入力検証 | 2000〜2099 | `VALIDATION_FIELD_REQUIRED` / `VALIDATION_FIELD_FORMAT` / `VALIDATION_SIZE_EXCEEDED` / `VALIDATION_ENUM_UNKNOWN` | protoc-gen-validate と対称 |
| レート制限 | 3000〜3099 | `RATE_LIMIT_PER_TENANT` / `RATE_LIMIT_PER_USER` / `RATE_LIMIT_PER_API` / `RATE_LIMIT_QUOTA_EXCEEDED` | 粒度別に原因特定可 |
| バックエンド | 4000〜4099 | `BACKEND_UNAVAILABLE` / `BACKEND_TIMEOUT` / `BACKEND_CIRCUIT_OPEN` / `BACKEND_DEPENDENCY_FAILED` | 依存先分離で MTTR 短縮 |
| 内部 | 5000〜5099 | `INTERNAL_UNKNOWN` / `INTERNAL_CONFIG_MISSING` / `INTERNAL_SERIALIZATION_FAILED` | バグ起因の分類 |

各 API 固有エラー（例 `STATE_ETAG_MISMATCH`、`PUBSUB_TOPIC_NOT_FOUND`）は 10000〜 の API 別番号空間で個別 API ファイルに採番する。共通系統の番号空間は個別 API では使用禁止とし、Buf Schema Registry で強制する。

## gRPC / HTTP status code マッピング

K1s0Error の `code` フィールドから gRPC status code と HTTP status code への写像が未定義だと、SDK 側の再試行判定・サーキットブレーカ判定が API ごとにブレる。Google API Design Guide の写像ルールを基底に tier1 固有のカスタマイズを加えた写像表を固定する。

**設計項目 DS-SW-EIF-204 ErrorCode → gRPC status / HTTP status 写像**

以下の写像を SDK の interceptor で統一実装し、個別 API での再定義を禁止する。

| ErrorCode 系統 | gRPC status code | HTTP status code | 根拠 |
|---------------|-----------------|-----------------|------|
| AUTH_TOKEN_MISSING / INVALID / EXPIRED | `UNAUTHENTICATED` (16) | 401 | RFC 6750 / Google API Guide 一致 |
| AUTH_TENANT_MISMATCH / PERMISSION_DENIED | `PERMISSION_DENIED` (7) | 403 | 認証は済で認可が拒否 |
| VALIDATION_* | `INVALID_ARGUMENT` (3) | 400 | クライアント起因 |
| RATE_LIMIT_* | `RESOURCE_EXHAUSTED` (8) | 429 | RFC 6585 / Google API Guide 一致 |
| BACKEND_UNAVAILABLE | `UNAVAILABLE` (14) | 503 | 再試行可性シグナル |
| BACKEND_TIMEOUT | `DEADLINE_EXCEEDED` (4) | 504 | ゲートウェイタイムアウト |
| BACKEND_CIRCUIT_OPEN | `UNAVAILABLE` (14) | 503 | クライアント側で backoff を期待 |
| BACKEND_DEPENDENCY_FAILED | `FAILED_PRECONDITION` (9) | 424 | 依存先ダウンは前提条件違反 |
| INTERNAL_* | `INTERNAL` (13) | 500 | サーバ起因、再試行の判定はクライアント裁量 |
| 個別 API 固有の NOT_FOUND | `NOT_FOUND` (5) | 404 | 各 API ファイルで採番 |
| 個別 API 固有の ALREADY_EXISTS | `ALREADY_EXISTS` (6) | 409 | 冪等キー併用時の検出 |
| 個別 API 固有の ABORTED / ETAG_MISMATCH | `ABORTED` (10) | 409 | 楽観ロック競合、State API で使用 |

HTTP 側では `Retry-After` ヘッダを `UNAVAILABLE` / `DEADLINE_EXCEEDED` / `RESOURCE_EXHAUSTED` の 3 種に必ず付与する。値は秒単位で、サーバ側の現在状態（サーキットブレーカ open 残時間 / Rate Limit window 残時間）を正直に反映する。

## タイムアウト契約

gRPC は deadline、HTTP は timeout で概念が分離しているが、tier1 としては両者を同一の「締切時刻」として扱い、サーバ側で統一的に cancel 伝搬する。deadline の伝搬が未定義だとバックエンド（Valkey / Kafka / Postgres）へのリクエストがゾンビ化し、リソース枯渇を招く。

**設計項目 DS-SW-EIF-205 deadline 伝搬契約**

クライアントは gRPC では `ctx.WithDeadline()`、HTTP では `Request-Timeout`（秒、IETF draft-ietf-httpapi-deadline）ヘッダで締切を指定する。tier1 サーバは受信 deadline を Dapr / バックエンドに透過伝搬し、deadline 超過時は gRPC `DEADLINE_EXCEEDED` / HTTP 504 を返して下流処理を即時 cancel する。クライアントから deadline 未指定の場合は API 種別ごとのデフォルト値（Service Invoke 30s、State 5s、PubSub 10s、その他は個別 API ファイルで定義）を適用する。

deadline 到達時、tier1 は `context.Canceled` を下流に伝搬し、Valkey クライアント（`go-redis`）の `WithContext` / Kafka クライアント（`kgo`）の `ctx` 引数で確実に cancel を波及させる。cancel の確認はリリース前 E2E テストで、deadline 超過後 100ms 以内にバックエンド接続が閉じることを観測する。

## Rate Limit レスポンス仕様

親ファイル DS-SW-EIF-007 で `X-K1s0-RateLimit-Reset` ヘッダ付与を宣言したが、window 境界の定義と残量表示の有無が未定義だった。クライアント側の自動 backoff を高精度化するため、IETF draft-ietf-httpapi-ratelimit-headers に準拠しつつ tier1 固有拡張を加える。

**設計項目 DS-SW-EIF-206 Rate Limit レスポンスヘッダ一式**

429 応答時に以下のヘッダを必ず付与する。根拠セルに「なぜ付けるか」を併記する。

| ヘッダ | 値 | 根拠 |
|--------|----|------|
| `Retry-After` | 次ウィンドウ開始までの秒数（整数） | RFC 6585、既存クライアントの自動再試行と互換 |
| `X-K1s0-RateLimit-Limit` | 現ウィンドウの上限値 | クライアント側の残量表示 UI で使用 |
| `X-K1s0-RateLimit-Remaining` | 現ウィンドウの残量（0 になった瞬間に 429） | 429 到達前の予測的 backoff を可能化 |
| `X-K1s0-RateLimit-Reset` | ウィンドウ終了時刻の Unix 秒 | 絶対時刻での再試行計算、時計ずれに堅牢 |
| `X-K1s0-RateLimit-Policy` | `1000;w=60` 形式（1000 req/60s） | IETF draft 準拠の機械可読表現 |

Rate Limit の window は sliding window を 10 秒粒度で近似する固定 window とし、Envoy Gateway の local rate limit フィルタで実装する。テナントごと / ユーザごと / API ごとの 3 軸で独立カウントし、どの軸に抵触したかは `K1s0Error.details[]` 内 `google.rpc.QuotaFailure.Violation.subject` で通知する。

## サイズ上限

gRPC / HTTP のメッセージサイズ上限が未固定だと、tier2 / tier3 の開発者が想定外の 413 / RESOURCE_EXHAUSTED に遭遇する。また、無制限な大容量リクエストは攻撃面を広げるため、セキュリティ観点でも上限を明示固定する。

**設計項目 DS-SW-EIF-207 メッセージサイズ上限とエラー挙動**

tier1 全 API の上限値は以下のとおり固定する。個別 API が更に厳しい上限を必要とする場合（State の value 1MB 等）は個別 API ファイルで上書き定義する。

| チャネル | 上限値 | 根拠 |
|---------|--------|------|
| gRPC 単一メッセージ | 4 MB | `grpc-go` の既定値、HTTP/2 フレーム制約 16MB より保守的に設定 |
| gRPC ストリーム 1 メッセージ | 4 MB | 上同、ストリーム全体は無制限だが 1 メッセージ単位で制約 |
| HTTP/JSON リクエストボディ | 10 MB | .NET Framework クライアント経由のバッチ投入を考慮した上限、超過は 413 |
| HTTP/JSON レスポンスボディ | 10 MB | 上同、ページネーションで超過回避を前提 |
| metadata / ヘッダ合計 | 32 KB | Envoy Gateway 既定値、Authorization の JWT で 4KB 消費想定 |

上限超過時は `VALIDATION_SIZE_EXCEEDED`（gRPC `INVALID_ARGUMENT` / HTTP 413）を返し、`K1s0Error.details[]` に超過量（byte）と上限値を `ErrorInfo.metadata` として付与する。

## SDK 共通ミドルウェア（interceptor）

4 言語 SDK（Go / Rust / TypeScript / C#）の利用者が共通契約を手書きで実装するのは事故の温床である。SDK 自動生成時に認証 / トレース / エラー統一の 3 つの interceptor を強制同梱し、利用者が interceptor を取り外せない構成にする。

**設計項目 DS-SW-EIF-208 SDK 共通 interceptor の責務分担**

SDK は以下 3 つの interceptor を必須同梱する。各 interceptor は単一責任原則で責務を分離し、利用者側の custom interceptor が挟まる場合も副作用が決定的に合成できる順序で固定する。

| 順序 | interceptor | 責務 | 言語別実装 |
|-----|------------|------|-----------|
| 1 (最外) | AuthInterceptor | JWT 取得・自動更新・Authorization ヘッダ付与 | Go: `grpc.UnaryClientInterceptor` / Rust: `tower::Layer` / TS: `@grpc/grpc-js` Interceptor / C#: `CallCredentials` |
| 2 | TraceInterceptor | W3C traceparent 生成・継承・OTel span 作成 | 各言語の OTel 公式 gRPC instrumentation |
| 3 (最内) | ErrorInterceptor | K1s0Error → 言語固有例外クラス変換、retry 判定 | 各言語で手書き、ErrorCode → Exception 写像テーブル同梱 |

AuthInterceptor は JWT の expiry を 30 秒前倒しで更新し、Keycloak への refresh_token リクエストを直列化する（同時更新による token 抹消を防止）。TraceInterceptor は親 span が存在しない場合にサンプリング判定（本番 10%、Pre / Dev 100%）を実施する。ErrorInterceptor は `UNAVAILABLE` / `DEADLINE_EXCEEDED` を再試行対象として判定し、指数バックオフ（100/200/400ms、jitter 50%）で最大 3 回再試行する。

## Dapr 隠蔽契約と SDK 直接利用の禁止

k1s0 の競合差別化（[../../../../01_企画/企画書.md](../../../../01_企画/企画書.md) p174-193、構想設計 ADR-TIER1-001 / ADR-TIER1-007）の中核は「tier2 / tier3 が Dapr に縛られない」ことであり、この約束は本ファイルの規約より上位に位置する設計原則である。tier1 公開 11 API は Dapr Go SDK の薄いラッパーではなく、Dapr バージョン更新・実装差し替え（Sidecar 廃止 / Service Mesh 統合 / 商用 IDP 移行）を tier2 / tier3 のコード変更ゼロで吸収する**抽象境界**として設計される。この境界が漏れると、Dapr のメジャー更新（過去実績で API シグネチャ変更が発生）が tier2 / tier3 全コードベースの改修を強制し、採用検討で約束した「Dapr ロックインの不在」が崩壊する。

本節の規約は親ファイル DS-SW-EIF-001〜016 で宣言した横断契約と整合し、SDK 共通ミドルウェア（DS-SW-EIF-208）が遵守を強制する。Dapr 隠蔽は単なる「SDK ラッパーがある」ことではなく、「直接利用が構造的に不可能である」ことが本質であり、後者を CI ガードと PR レビューで担保する。

### 設計項目 DS-SW-EIF-212 tier2 / tier3 における Dapr SDK 直接利用の 4 禁止事項

tier2 マイクロサービスおよび tier3 アプリは以下 4 種の Dapr 直接利用を一切行ってはならない。違反は CI ゲート（DS-SW-EIF-213）と PR レビュー（[../../../00_設計方針/04_設計レビュー方針.md](../../../00_設計方針/04_設計レビュー方針.md)）の双方で阻止する。

| 禁止事項 | 具体例 | 違反検出手段 | 違反時の挙動 |
|---|---|---|---|
| 1. import 直接禁止 | `import "github.com/dapr/go-sdk/client"`、`using Dapr.Client;`、`from dapr.clients import DaprClient` | CI で AST 解析（Go: `go list -deps -json` / .NET: Roslyn / TS: ts-morph / Python: ast モジュール）、`dapr` プレフィックスの直接 import を検出 | PR ブロック（人手 override 不可、tier1 チームの ADR 起票を経た例外承認のみ） |
| 2. 動的ロード禁止 | `plugin.Open("dapr-client.so")`、`Assembly.LoadFrom("Dapr.Client.dll")`、`require("@dapr/dapr")` の遅延読込 | CI で文字列スキャン + シンボル解析、動的 link 候補を検出 | PR ブロック |
| 3. リフレクション禁止 | Go の `reflect.ValueOf(daprClient).Method(...)`、.NET の `Type.GetType("Dapr.Client.DaprClient").GetMethod(...)` 経由の Dapr SDK アクセス | CI で reflect / Reflection の使用箇所を検出し、Dapr 関連シンボル文字列との同居を警告 | PR レビューで個別判断（false positive 多いため） |
| 4. Dapr Sidecar 直接 HTTP / gRPC 呼び出し禁止 | `http://localhost:3500/v1.0/state/...` への直接アクセス、`grpc://localhost:50001` への直接接続 | CI で `localhost:3500` / `localhost:50001` のハードコード文字列を検出、tier1 SDK 経由でのみ接続可能とする内部 DNS（`tier1.k1s0.svc.cluster.local`）以外への接続を禁止 | PR ブロック |

本表の禁止対象は Dapr に閉じる。tier2 / tier3 が Kubernetes API・Istio API・Prometheus exposition formatに依存することは禁止しない（これらは k1s0 の差別化対象外であり、業界標準）。Dapr のみが「将来差し替え可能性のある実装詳細」として隠蔽対象となる根拠は、構想設計 ADR-TIER1-001 で明示している。

### 設計項目 DS-SW-EIF-213 tier1 Go ファサード層の Dapr SDK 隔離責務

tier1 Go ファサード層は Dapr Go SDK を import できる**唯一の層**として位置付け、3 種の責務を負う。各責務は独立したパッケージとして実装し、ファサード層内部でも Dapr 依存を最小化する。

| 責務 | 実装パッケージ | 担保するもの |
|---|---|---|
| 1. Dapr SDK バージョン同期吸収 | `tier1/internal/dapr/client` | Dapr Go SDK の API シグネチャ変更を本パッケージ内で吸収。tier1 の他パッケージは本パッケージ提供の安定インタフェース（`tier1/internal/dapr/iface`）のみに依存。Dapr 1.x → 2.x 移行時の影響範囲を本パッケージに局所化 |
| 2. エラー正規化 | `tier1/internal/dapr/errnorm` | Dapr SDK ネイティブエラー（`status.Error(codes.Unavailable, ...)` 等）を K1s0Error（DS-SW-EIF-202）に変換。tier2 / tier3 は K1s0Error のみを観測 |
| 3. トレース・メトリクス統一 | `tier1/internal/dapr/otel` | Dapr Sidecar 経由の呼び出しに OTel span を作成し、tier1 の trace_id と連結。Dapr 内部 span（`dapr_runtime_*`）は tier2 / tier3 に露出させず、tier1 内部 span のみを公開トレースに昇格 |

`tier1/internal/dapr/` パッケージ群への直接 import は tier1 の公開 API パッケージ（`tier1/api/v1/...`）からは禁止する。Go の internal 規約で物理的に隔離し、不正 import はビルドエラーとなる構造を作る。本構造は CI で `go list ./...` の依存グラフ抽出 + `internal` パッケージへの不正参照検査として強制する。

### 設計項目 DS-SW-EIF-214 Dapr バージョン更新の影響境界と SLO 維持契約

Dapr のバージョン更新（patch / minor / major）が tier2 / tier3 に与える影響境界を 3 段階で固定する。本契約により、Dapr 更新作業を tier1 内部で完結させ、採用検討で約束した「Dapr ロックインの不在」を SLO 観点でも担保する。

| Dapr バージョン更新種別 | tier2 / tier3 への影響 | tier1 ファサードの責務 | SLO 維持要件 |
|---|---|---|---|
| Patch（1.13.0 → 1.13.1） | ゼロ。SDK 互換性 100% 維持 | DS-SW-EIF-213 の 3 責務でブラックボックス更新 | tier1 公開 API の p99 / 可用性 SLO に劣化を生じさせない（劣化検出時はロールバック）|
| Minor（1.13 → 1.14） | ゼロ。新機能は tier1 公開 API として明示露出した場合のみ tier2 / tier3 が認知 | DS-SW-EIF-213 の 1（バージョン同期）でシグネチャ差分を吸収。新機能の公開は別 PR + ADR | 同上 + 新機能露出時は SemVer minor bump（DS-SW-EIF-210 と整合）|
| Major（1.x → 2.x） | ゼロ。tier1 API シグネチャは v1 系を維持し、必要に応じて v2 を追加（12 か月並列運用、DS-SW-EIF-209 と整合） | DS-SW-EIF-213 の全責務 + Dapr 2.x 専用の `tier1/internal/dapr/v2/` を新設し、v1 / v2 双方の Sidecar に対応 | 12 か月並列運用中は v1 / v2 双方で SLO を維持。期限経過後 v1 廃止は major bump として再周知 |

本契約の最重要点は、Dapr major 更新（過去実績で年 1 回程度発生）の作業負荷を tier1 チーム内で完結させ、tier2 / tier3 開発チームの工数を一切要求しないことである。これは [../../../../01_企画/企画書.md](../../../../01_企画/企画書.md) で約束した「OSS 隠れコスト」の主要項目（Dapr SDK 同期工数 年 1,242 万円相当）が、tier1 内部に閉じることの設計担保である。

## Contract Test

`.proto` 変更が破壊的になっていないか、生成 SDK が期待どおり動作するか、HTTP/JSON 変換が正しいかを自動検証する仕組みが未整備だと、リリース時に tier2 / tier3 側で初めて破壊的変更に気付く事故が起きる。contract test を CI パイプライン必須ゲートとして組み込む。

**設計項目 DS-SW-EIF-209 Contract Test の 3 段構成**

Contract test は以下 3 段で構成し、PR マージ前に全段合格を必須とする。

| 段階 | ツール | 検証内容 | 不合格時の扱い |
|------|--------|---------|--------------|
| 1. Schema breaking check | `buf breaking` | `.proto` の field 削除・tag 変更・enum 値削除を検出 | PR ブロック（人手 override 不可） |
| 2. Validation rule check | `protoc-gen-validate` + `buf lint` | 必須フィールド未設定・enum 範囲外・サイズ上限違反を検出 | PR ブロック（override 可だが ADR 必須） |
| 3. Interoperability test | `grpcurl` + `curl` で 4 言語 SDK を試験実行 | gRPC と HTTP/JSON で同一結果を返すことを E2E 確認 | PR ブロック、リリース候補ビルドで再実行 |

Schema breaking check は `buf.build/breaking-config` で FILE レベル（フィールド削除、tag 変更、enum 値削除）を検出する。意図的な破壊的変更は major バージョン（`k1s0.public.state.v2.*`）への切り替えを要求し、v1 と v2 は最低 12 か月並列運用する（非推奨方針は `../../../40_運用ライフサイクル/04_非推奨とEOL.md` に同期）。

## セマンティック非後方互換の検出

Schema breaking check は syntactic な互換性のみ検出し、意味的変更（enum 値の意味が変わる、レスポンスの単位が ms → us に変わる、等）は検出できない。セマンティック互換性は別の検出系を設ける必要がある。

**設計項目 DS-SW-EIF-210 セマンティック非後方互換の検出ゲート**

セマンティック変更の検出は以下の 3 手段を重ねる。単一ゲートでは漏れるため多層防御とする。

1. `.proto` フィールドのコメント末尾に `@semver=<major>.<minor>.<patch>` タグを付与し、変更時に tag 更新を PR レビューで強制する。tag 更新なしの意味変更はレビューで reject する。
2. `k1s0-api-version` レスポンスヘッダの minor 値を変更時に bump し、SDK 側の version check で警告ログを出力する。クライアントが古い minor に依存した挙動（例: レスポンスフィールドの単位）を検出できる。
3. リリースノート起票を CI で強制する（`docs/CHANGELOG.md` の差分がない PR は block）。リリースノートには Breaking / Non-breaking / Deprecated の 3 区分を必須記載する。

セマンティック変更の検出は完全自動化は不可能であり、レビュー体制と運用ルールの合意形成に帰着する。本規約はレビュアーの判定負荷を最小化するためにツール側の強制と明示的タグを組み合わせる。

## 共通規約の SLO インパクト

共通規約（特に interceptor 3 段）が API 単位の SLO に与える加算レイテンシを固定する。加算値を個別 API で都度計測すると SLO 未達の原因切り分けが困難になる。

**設計項目 DS-SW-EIF-211 共通 interceptor の加算レイテンシ**

共通 interceptor 3 段の加算レイテンシは以下のとおり固定する。これを超えた場合は interceptor の実装を見直し、SLO 計算の前提条件として全 API ファイルで引用する。

| interceptor | 加算 p50 | 加算 p99 | 根拠 |
|------------|---------|---------|------|
| AuthInterceptor | 0.1 ms | 0.5 ms | JWT キャッシュヒット時。refresh 時のみ 30ms だがキャッシュヒット率 99.5% で p99 に影響しない |
| TraceInterceptor | 0.05 ms | 0.2 ms | OTel SDK 公称値、BatchSpanProcessor 使用 |
| ErrorInterceptor | 0.01 ms | 0.05 ms | 正常系は pass-through、例外時のみ写像処理 |
| 合計加算 | 0.16 ms | 0.75 ms | State API p99 10ms に対して 7.5% 枠、許容範囲 |

加算レイテンシの実測は負荷試験（`../../../03_要件定義/30_非機能要件/B_性能拡張/`）で検証し、逸脱時は ADR 改訂を要する。

## 対応要件一覧

本ファイルは tier1 公開 API 共通規約の実装・検証設計であり、要件 ID → 設計 ID の 1:1 対応を以下の表で固定する。表形式併記は DR-COV-001 への緩和策として、CI スクリプトでの機械検証の一次入力となる。共通規約は全 11 API に横串で効くため、本表に載る要件は横断系のみであり、API 固有要件は各 API 方式ファイルの対応要件一覧に別途表形式で記載されている。

| 要件 ID | 要件タイトル | 対応設計 ID | カバー状況 |
|---|---|---|---|
| FR-T1-001〜FR-T1-049（横串契約） | 全 11 API の gRPC / HTTP 契約 横断要件の共通部分 | DS-SW-EIF-200〜211 | 完全 |
| NFR-E-AC-001 | JWT 必須 | DS-SW-EIF-200, DS-CF-AUTH-002 | 完全 |
| NFR-E-AC-002 | トークン検証の強制 | DS-SW-EIF-201, DS-CF-AUTH-003 | 完全 |
| NFR-E-AC-003 | tenant 強制 | DS-SW-EIF-202, DS-CF-AUTHZ-001 | 完全 |
| NFR-E-AC-010 | Rate Limit 多軸 | DS-SW-EIF-203, DS-NFR-SEC-022 | 完全 |
| NFR-B-PERF-001 | p99 レイテンシ目標（interceptor 加算 0.75ms 以内） | DS-SW-EIF-204, DS-NFR-PERF-001 | 完全 |
| NFR-C-OPS-015 | 構造化エラー（問い合わせ対応 MTTR 短縮） | DS-SW-EIF-205, DS-NFR-OPS-015 | 完全 |
| DX-LD-003 | API 学習コスト 1 時間以内（従来 NFR-DEVEX-001 表記） | DS-SW-EIF-206, DS-DEVX-LD-003 | 完全 |

表に載せた要件数は横串 FR 1 行 + NFR 6 件 + DX 1 件 = 計 8 行（横串 FR 行は 49 件をまとめた 1 行として計上）。NFR-DEVEX-001 と称していた要件は DX-LD-003（開発者体験・ローカル開発）に正規化されたため、表記を改めた。

補助参照は以下のとおり。

- ADR 参照: ADR-TIER1-002（Protobuf gRPC 必須）/ ADR-TIER1-003（buf CI ゲート）
- 親参照: [01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) DS-SW-EIF-001〜016
- 本ファイルで採番: DS-SW-EIF-200 〜 DS-SW-EIF-211
