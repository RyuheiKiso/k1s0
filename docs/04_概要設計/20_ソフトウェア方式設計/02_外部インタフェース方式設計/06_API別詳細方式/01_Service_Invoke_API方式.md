# 01. Service Invoke API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応し、tier1 11 API のうち同期サービス呼び出しを担う Service Invoke API の詳細方式を定める。共通契約は [00_API共通規約方式.md](00_API共通規約方式.md)（DS-SW-EIF-200〜211）を参照し、本ファイルは Service Invoke 固有の Protobuf 定義・再試行ポリシー・.NET Framework 互換仕様・SLO 内訳の 4 点に集中する。

## 本ファイルの位置付け

Service Invoke API は tier2 / tier3 のビジネスアプリが他アプリを同期呼び出しするための唯一の正規経路である。親ファイル [01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001 で宣言したとおり、Dapr service-invocation を Go ファサード（`facade-svcinv` Pod）でラップし、Istio Ambient の mTLS を透過的に効かせる構造を採る。業務ロジックから見ると「相手 app_id を指定して呼び出すだけで認証・暗号化・トレース・再試行が付随する」抽象として提供する。

この抽象が崩れると、tier2 / tier3 が認証ヘッダの手動付与、サーキットブレーカの自作、JWT refresh の個別実装といった低レイヤ労苦を背負うことになり、ビジネス価値の産出速度が目に見えて落ちる。本ファイルは抽象の境界を過不足なく定義し、実装担当（Go facade-svcinv チーム）とクライアント担当（tier2 / tier3）の双方が同じ契約に従える状態を作る。

.NET Framework 4.8 クライアント対応は JTC の既存資産の大部分を占め、Service Invoke の HTTP/1.1 互換プロキシ機能は稟議通過の条件の 1 つである。この対応が欠けると既存資産の漸進的移行が不能となり、短期 ROI が崩れる。本ファイルは gRPC 主・HTTP/1.1 補助の二層を方式として明確化する。

## Protobuf Service 定義

親ファイル DS-SW-EIF-008 で gRPC を一次公開形式と宣言した。Service Invoke API の gRPC service 定義は Dapr の `daprd` が提供する `ServiceInvocation` Proto を直接公開せず、tier1 独自の抽象 proto `k1s0.public.invoke.v1.ServiceInvoke` で覆う。Dapr を直接公開すると将来の Dapr 非互換変更や代替バックエンド（自作 mesh 等）への移行時に SDK 互換性が崩れる。

**設計項目 DS-SW-EIF-220 Protobuf Service 定義**

`protos/k1s0/public/invoke/v1/service_invoke.proto` に以下の service を定義する。`Invoke` は gRPC ネイティブ呼び出し用、`InvokeHttp` は HTTP/1.1 相手への透過プロキシ用として RPC を 2 つに分離する。HTTP と gRPC を 1 つの RPC に混載すると、verb / query / body の取扱いが不明瞭になりクライアント側で誤用が多発する。

```protobuf
service ServiceInvoke {
  // gRPC ターゲット呼び出し。data は Any でラップされる Protobuf message
  rpc Invoke(InvokeRequest) returns (InvokeResponse);
  // HTTP/1.1 ターゲット呼び出し。.NET Framework 系 REST サーバ互換
  rpc InvokeHttp(InvokeHttpRequest) returns (InvokeHttpResponse);
}

message InvokeRequest {
  // Dapr app-id（DS-SW-EIF-226 の命名規約準拠）
  string target_app_id = 1;
  // メソッド名（Protobuf full method "pkg.Service/Method" 形式）
  string method = 2;
  // リクエストペイロード（Any でラップ）
  google.protobuf.Any data = 3;
  // 追加 metadata（allow-list 経由で伝搬）
  map<string, string> metadata = 4;
}

message InvokeHttpRequest {
  string target_app_id = 1;
  // HTTP verb（GET / POST / PUT / DELETE / PATCH）
  HttpVerb http_verb = 2;
  // パス（例 "/api/v1/orders"）、query 文字列を含む完全 URI ではなく path のみ
  string path = 3;
  // query パラメータ
  map<string, string> http_query = 4;
  // body（Content-Type で解釈）
  bytes data = 5;
  // Content-Type（例 "application/json"）
  string content_type = 6;
  // 追加ヘッダ（allow-list 経由で伝搬）
  map<string, string> metadata = 7;
}
```

レスポンスは対称的に `InvokeResponse` / `InvokeHttpResponse` を定義する。HTTP 応答は `http_status` と `body`、`content_type`、`headers` を返す。エラーは共通規約 DS-SW-EIF-202 の K1s0Error に集約する。

## .NET Framework 向け HTTP/1.1 互換プロキシ仕様

親ファイル DS-SW-EIF-012 で .NET Framework 4.8 は Phase 1c まで `Grpc.Core` を使用し、Phase 2 以降は HTTP/JSON 一本化と宣言した。Service Invoke は .NET Framework クライアントが tier2 / tier3 の既存 REST サーバを呼び出す正規経路でもあり、HTTP/1.1 互換プロキシを `InvokeHttp` RPC として提供する構成を取る。

**設計項目 DS-SW-EIF-221 HTTP/1.1 互換プロキシの実装境界**

`facade-svcinv` Pod は受信した `InvokeHttpRequest` を Dapr の HTTP service-invocation（`/v1.0/invoke/<app-id>/method/<method>`）に変換して転送する。Dapr は相手 sidecar の HTTP/1.1 endpoint を Istio Ambient L7 経由で呼び出し、応答を `InvokeHttpResponse` に再梱包して返す。クライアント側は gRPC `Invoke` と同じ interceptor チェーン（認証 / トレース / エラー統一）を再利用でき、HTTP であることを意識しない。

HTTP/1.1 互換は tier2 / tier3 の REST サーバが Dapr sidecar を持つことが前提で、sidecar を持たない外部サーバへの呼び出しは Binding API（`../11_Feature_API方式.md` ではなく `05_Binding_API方式.md`）の領分として分離する。この責務境界が曖昧になると Service Invoke が無節操に拡大し、Dapr の service-mesh 抽象が壊れる。

## タイムアウト・リトライ・サーキットブレーカ

親ファイル DS-SW-EIF-205（共通規約）で deadline 伝搬契約を定めたが、Service Invoke は特に相手アプリの一時障害（GC 一時停止、デプロイ中の接続拒否）が頻発するため、再試行とサーキットブレーカの設計を具体化する必要がある。無節操な再試行は相手アプリに雪崩を起こすため、指数バックオフ + jitter + サーキットブレーカの三点セットで抑制する。

**設計項目 DS-SW-EIF-222 再試行ポリシー**

再試行は以下の条件に限定する。500 系の `INVALID_ARGUMENT` や `UNAUTHENTICATED` は再試行しない（再送しても成功しない）。`DEADLINE_EXCEEDED` は deadline 残量を確認し、残量が再試行 1 回分未満なら再試行を打ち切る。

| 判定条件 | 再試行する/しない | 根拠 |
|---------|--------------|------|
| gRPC `UNAVAILABLE` | する | 一時障害、Google API Guide 準拠 |
| gRPC `DEADLINE_EXCEEDED` | 残量あれば 1 回のみ | deadline 超過は再試行でも失敗しやすい |
| gRPC `RESOURCE_EXHAUSTED`（429） | しない（Retry-After 尊重） | 相手側で拒否されており再送は逆効果 |
| gRPC `UNAUTHENTICATED` / `PERMISSION_DENIED` | しない | 認証情報の自動再発行は AuthInterceptor 側で実施済 |
| gRPC `INVALID_ARGUMENT` / `NOT_FOUND` | しない | クライアント起因、再送で成功しない |
| gRPC `INTERNAL` | しない | サーバ起因バグ、再送よりエスカレ |

再試行間隔は指数バックオフ 100ms / 200ms / 400ms、各値に ±50% の jitter を加える。最大再試行回数は 3 回、総遅延は最悪 1050ms（+ ネットワーク RTT）に抑制する。根拠: 700ms の再試行総遅延でも Service Invoke p99 300ms SLO を 1 回の失敗で即座に超えるため、再試行はあくまで「短期的なブリップ」への対処と割り切る。長時間障害はサーキットブレーカで即座に fail-fast させる。

**設計項目 DS-SW-EIF-223 サーキットブレーカ**

`facade-svcinv` は相手 app_id ごとに sliding window サーキットブレーカを持つ。5 連続失敗（`UNAVAILABLE` / `DEADLINE_EXCEEDED`）で open 状態に遷移し、30 秒間は即時 `BACKEND_CIRCUIT_OPEN`（HTTP 503）を返す。30 秒後 half-open に遷移し、1 リクエストのみ試行して成功すれば close、失敗すれば再度 30 秒 open に戻る。

根拠: 5 連続失敗の閾値は「一時的ブリップ（通常 1〜2 連続失敗で自己回復）」と「持続的障害」を分離する経験値。30 秒の open 時間は Kubernetes の PodDisruptionBudget 再配置や Istio エンドポイント再収束の所要時間 15〜25 秒に対して十分なマージンを確保する。サーキットブレーカの状態は Prometheus メトリクス `k1s0_svcinv_circuit_state{target_app_id=...}` として export し、運用で可視化する。

## mTLS 自動付与（Istio Ambient）

Service Invoke の通信は全区間で暗号化されている必要がある（NFR-E-ENC-005）。Istio Ambient Mesh を採用し、Pod 間通信（`facade-svcinv` → 相手 Pod）は透過的に mTLS 化する。

**設計項目 DS-SW-EIF-224 Istio Ambient による mTLS 自動付与**

tier1 / tier2 / tier3 の全 Namespace に `istio.io/dataplane-mode: ambient` ラベルを付与し、ztunnel が Pod 間通信を自動的に mTLS でラップする。アプリ側・`facade-svcinv` 側ともに証明書管理・TLS handshake・証明書ローテーションの実装は不要で、すべて Istio CA（Citadel）が自動処理する。

Ambient は sidecar 不要なため起動時間・メモリ消費が sidecar mesh より軽く、tier1 の 11 API × 複数レプリカ構成でも運用負荷が線形増加しない（sidecar mesh 比 40% メモリ削減、Istio 公称値）。L7 機能（retry / 再送 / header 操作）が必要な場合のみ waypoint proxy を追加投入し、Service Invoke のように tier1 が L7 制御を自前で行う API では waypoint は不要とする。

## ヘッダ伝搬範囲

Service Invoke は相手アプリに metadata / HTTP ヘッダを伝搬する際、無制限に伝搬するとセキュリティホール（例: 認証トークンが第三者アプリに漏れる）になる。allow-list 方式で厳密に制御する。

**設計項目 DS-SW-EIF-225 ヘッダ伝搬 allow-list**

以下のヘッダのみ相手アプリに自動伝搬する。allow-list 外のヘッダはクライアントが明示的に `metadata` フィールドで指定した場合のみ伝搬する。

| ヘッダ | 伝搬 | 根拠 |
|--------|-----|------|
| `authorization` | 自動伝搬 | 相手アプリ側でも JWT 検証必須、同一テナント内の遷移 |
| `traceparent` / `tracestate` | 自動伝搬 | 分散トレース連結 |
| `k1s0-tenant-id` | 自動伝搬 | テナント境界維持 |
| `k1s0-correlation-id` | 自動伝搬 | 業務相関 |
| `idempotency-key` | 自動伝搬 | 冪等連鎖 |
| `Cookie` / `Set-Cookie` | 伝搬禁止 | セッション混同リスク、REST Cookie ベース認証は Service Invoke では非サポート |
| `Host` / `X-Forwarded-*` | 伝搬禁止 | Envoy / ztunnel が自己管理 |
| カスタムヘッダ | 明示指定時のみ伝搬 | 意図しない情報漏洩防止 |

allow-list の定義は `.proto` のコメントではなく `facade-svcinv` の Go 実装で hard-code し、設定変更には PR レビューを必須とする。ランタイムで変更可能な config にすると攻撃面が広がるため、コンパイル時固定とする。

## Dapr app-id 命名規約

Service Invoke の `target_app_id` は Dapr が Pod 発見に使う一意識別子であり、命名が衝突すると誤配信が発生する。tier1 側で命名空間を管理し、テナント・ドメイン・アプリ種別の 3 軸で一意性を保証する。

**設計項目 DS-SW-EIF-226 Dapr app-id 命名規約**

app-id は `<tenant_id>-<domain>-<service>-<env>` 形式とする。例: `t0001-order-api-prod`、`t0002-inventory-worker-pre`。構成要素と根拠は以下のとおり。

| セグメント | 形式 | 根拠 |
|-----------|------|------|
| `tenant_id` | `t` + 4 桁数字 | 親ファイル DS-SW-EIF-004 と整合 |
| `domain` | 小文字英数字 3〜15 文字 | ビジネスドメインの識別（order / inventory / billing 等） |
| `service` | 小文字英数字ハイフン 3〜30 文字 | ドメイン内の具体サービス |
| `env` | `prod` / `pre` / `dev` のいずれか | 環境分離、本番と Pre を誤呼び出し防止 |

命名規約違反の app-id は `facade-svcinv` が起動時に validation し、違反 Pod は起動失敗させる。Dapr 側では規約違反が検出できないため tier1 側でバリデーションゲートを持つ。

## SLO p99 300ms の積算内訳

親ファイル DS-SW-EIF-013 で Service Invoke p99 300ms と宣言したが、どの区間に何 ms 配分するかが未定義だった。配分が曖昧だと SLO 逸脱時に責任区間の切り分けが遅れる。

**設計項目 DS-SW-EIF-227 p99 300ms の区間別内訳**

p99 300ms は以下の区間で構成される。各区間の閾値超過を Prometheus の区間別メトリクス（`k1s0_svcinv_latency_seconds{stage="..."}`）で個別監視する。

| 区間 | 配分 p99 | 根拠 |
|------|---------|------|
| クライアント SDK 内（interceptor 3 段） | 1 ms | DS-SW-EIF-211 の共通 interceptor 加算 0.75ms + 余裕 |
| クライアント → Envoy Gateway（NW） | 5 ms | AZ 内通信 |
| Envoy Gateway（認証 / ルーティング） | 4 ms | JWT 検証・Rate Limit・Tenant 判定 |
| Envoy → `facade-svcinv` Pod（NW） | 5 ms | 同一 AZ の Pod 間 |
| `facade-svcinv` Pod 内（Dapr SDK 呼び出し） | 10 ms | Go + Dapr SDK の処理時間 |
| `facade-svcinv` → Dapr sidecar（UDS） | 5 ms | Unix Domain Socket、localhost |
| Dapr sidecar → 相手 sidecar（mTLS） | 50 ms | Istio Ambient ztunnel mTLS 往復 |
| 相手アプリ処理 | 200 ms | ビジネスロジックの実質予算 |
| 応答経路（逆方向） | 20 ms | 要求経路の逆 |
| **合計** | **300 ms** | |

相手アプリの 200ms が tier2 / tier3 のビジネスロジックに割り当てられる実質予算であり、tier1 インフラ部分は 100ms 以内に収める設計制約となる。本表は SRE ダッシュボード（`../../../03_要件定義/30_非機能要件/B_性能拡張/`）の区間別可視化パネルと 1:1 対応させる。

## 固有エラーコード

共通規約 DS-SW-EIF-203 で 0〜9999 の番号空間を共通系統に割り当て、10000〜 を各 API の固有エラー空間とした。Service Invoke は 10100〜10199 を使用する。

**設計項目 DS-SW-EIF-228 Service Invoke 固有エラーコード**

| enum 値 | 番号 | gRPC status | HTTP | 発生条件 |
|--------|------|------------|------|---------|
| `INVOKE_TARGET_NOT_FOUND` | 10100 | `NOT_FOUND` | 404 | 指定 app_id が Dapr placement に未登録 |
| `INVOKE_TARGET_UNAVAILABLE` | 10101 | `UNAVAILABLE` | 503 | ターゲット Pod 全滅、Istio エンドポイント無し |
| `INVOKE_METHOD_NOT_FOUND` | 10102 | `UNIMPLEMENTED` | 501 | 相手が指定 method を実装していない |
| `INVOKE_TIMEOUT` | 10103 | `DEADLINE_EXCEEDED` | 504 | deadline 超過（DS-SW-EIF-205 伝搬） |
| `INVOKE_CIRCUIT_OPEN` | 10104 | `UNAVAILABLE` | 503 | DS-SW-EIF-223 サーキットブレーカ open |
| `INVOKE_PAYLOAD_TOO_LARGE` | 10105 | `INVALID_ARGUMENT` | 413 | DS-SW-EIF-207 の 4MB / 10MB 超過 |
| `INVOKE_HTTP_VERB_UNSUPPORTED` | 10106 | `INVALID_ARGUMENT` | 400 | 未対応 verb（CONNECT / TRACE 等） |

エラー発生時は `K1s0Error.details[]` に `google.rpc.ErrorInfo` を同梱し、`reason` に enum 名、`metadata` に `target_app_id` と `method` を付与する。SRE はこの詳細情報で原因 app を即座に特定できる。

## Phase 別公開範囲

親ファイル DS-SW-EIF-016 で Service Invoke は Phase 1a（MVP-0）から公開と宣言した。Phase 内での機能段階を明確化する。

**設計項目 DS-SW-EIF-229 Service Invoke の Phase 別機能**

| Phase | 公開機能 | 根拠 |
|-------|---------|------|
| Phase 1a（MVP-0） | `Invoke` RPC のみ、gRPC only、再試行・サーキットブレーカ有効 | 最小構成で稟議通過後の価値実証 |
| Phase 1b（MVP-1a） | `InvokeHttp` RPC 追加、HTTP/JSON 変換有効、.NET Framework `Grpc.Core` SDK 提供 | .NET Framework 資産の移行開始 |
| Phase 1c（MVP-1b） | ストリーミング RPC は非対応（Service Invoke は unary に限定） | 親ファイル DS-SW-EIF-008 と整合 |
| Phase 2 | `.NET Framework HTTP/JSON SDK` 一本化、`Grpc.Core` 廃止 | 公式非推奨ライブラリからの脱却 |

Phase 1a で HTTP/JSON を提供しない理由は、HTTP/JSON 変換の検証工数を MVP-0 のリソースでは確保できないためである。Phase 1b での追加により段階的に公開範囲を広げ、各 Phase で契約の後方互換性を守る。

## 対応要件一覧

本ファイルは Service Invoke API の詳細方式設計であり、以下の要件 ID に対応する。

- FR-T1-INVOKE-001〜FR-T1-INVOKE-005（Service Invoke 機能要件一式）
- FR-T1-INVOKE-001（同期 gRPC 呼び出し）/ FR-T1-INVOKE-002（HTTP/1.1 互換プロキシ）/ FR-T1-INVOKE-003（タイムアウト・リトライ制御）/ FR-T1-INVOKE-004（サーキットブレーカ）/ FR-T1-INVOKE-005（認証トークン自動伝搬）
- FR-EXT-DOTNET-001（.NET Framework 互換）
- NFR-E-ENC-005（全区間暗号化、Istio Ambient mTLS）
- NFR-B-PERF-001（Service Invoke p99 300ms）
- ADR 参照: ADR-TIER1-001（Go+Rust）/ ADR-TIER1-002（Protobuf gRPC）/ ADR-0001（Istio Ambient 採用）
- 共通規約参照: [00_API共通規約方式.md](00_API共通規約方式.md) DS-SW-EIF-200〜211
- 親参照: [01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) DS-SW-EIF-001 / 013 / 016
- 本ファイルで採番: DS-SW-EIF-220 〜 DS-SW-EIF-229
