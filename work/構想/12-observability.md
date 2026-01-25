## 12. 観測性（標準出力・疎通チェック）（MUST）

OTel Collector/Trace UI の起動を「用意した」で終わらせず、各サービスが必ず満たすべき出力と、dev-check の合格条件を固定する。

### 標準出力（固定）

ログ

- 形式: JSON（1 行 1 イベント）
- 必須フィールド: `timestamp`, `level`, `service_name`, `env`, `trace_id`, `span_id`, `message`
- 追加推奨: `request_id`, `grpc.method` / `http.route`, `error.kind`

メトリクス

- エクスポート: OTel Metrics（Collector 経由）
- 命名規約: `k1s0.<service>.<component>.<metric>`
- 必須メトリクス（最低限）
	- リクエスト数/失敗数（HTTP/gRPC）
	- レイテンシ（p50/p95/p99 を算出可能な histogram）
	- 外部依存（DB/Redis/他サービス）呼び出しの失敗数/レイテンシ
	- 設定取得（config-service/DB）の成功/失敗数

トレース

- 伝播: W3C Trace Context
- 必須属性（最低限）: `service.name`, `deployment.environment`, `rpc.system`/`rpc.method` または `http.method`/`url.path`
- 外部依存呼び出し（DB/Redis/他サービス）は span を切る

### 疎通チェック（`scripts/dev-check.ps1`）の合格条件（固定）

- プロセス/ヘルス
	- HTTP を提供する場合: `GET /healthz` が 200
	- gRPC のみの場合: `grpc.health.v1.Health/Check` が `SERVING`
- 依存疎通
	- DB: マイグレーション適用済みで `SELECT 1` 等の read が成功
	- Redis: ping 相当が成功（利用するサービスのみ）
	- config-service 利用時: 設定取得の最小 API が成功（キャッシュ運用ならその状態も出力）
- 観測
	- 起動後に少なくとも 1 件のトレース/メトリクスが Collector に到達していること

### エラー規約（REST / gRPC / ログ / メトリクス）（MUST）

目的: チーム間でエラー表現がバラつくことを防ぎ、運用（調査/監視/アラート/顧客対応）を標準化する。

#### エラーの層別責務（固定）

- domain
	- domain のエラーは「業務上の失敗」を表す（HTTP/gRPC に依存しない）
	- 外部I/O（DB/HTTP/Redis 等）の具体エラーを直接露出しない
- application
	- domain エラーと外部I/Oエラーを受け取り、ユーザーに返す失敗（分類/再試行可否/影響範囲）を決める
- presentation
	- application の失敗を REST/gRPC の表現へ変換する（共通ルールは framework が提供）

#### エラーコード（固定）

- 外部へ返すエラーには安定した `error_code` を必ず付与する（文字列）
- `error_code` は変更しない（名称変更が必要な場合は段階移行し、互換期間を設ける）
- 命名は次を推奨とし、テンプレで固定する
	- `{service_name}.{category}.{reason}`（小文字+数字+アンダースコア、ドット区切り）
	- 例: `auth.invalid_credentials`, `user.not_found`, `db.conflict`

#### REST のエラーレスポンス（固定）

- 形式は `application/problem+json`（RFC7807 互換）を基本とする
- 最低限、次の情報を必ず返す
	- `status`（HTTP status code）
	- `title`（短い要約。固定文言でよい）
	- `detail`（人間向け詳細。機密情報は含めない）
	- `error_code`（上記規約）
	- `trace_id`（観測性と相関するため必須）
	- `request_id`（採用する場合。未採用なら省略可）
- バリデーションエラーは `errors`（配列）でフィールド単位の情報を返してよい
	- 例: `[{"field":"email","reason":"required"}]`

#### gRPC のエラー表現（固定）

- gRPC は Canonical Status Code を使用し、アプリ独自コードは作らない
- 返す Status は次を基本とする（代表例）
	- 入力不正/バリデーション: `INVALID_ARGUMENT`
	- 認証失敗: `UNAUTHENTICATED`
	- 認可失敗: `PERMISSION_DENIED`
	- リソースなし: `NOT_FOUND`
	- 競合: `ALREADY_EXISTS` / `FAILED_PRECONDITION`
	- 外部依存の一時障害: `UNAVAILABLE`
	- タイムアウト: `DEADLINE_EXCEEDED`
	- 想定外: `INTERNAL`
- `error_code` をメタ情報として付与し、クライアントが機械判定できるようにする（方法は framework で統一）

#### framework による内外変換の固定（MUST）

目的: サービスごとに「HTTP/gRPCのエラー変換」を書かせず、同じ規約が“勝手に効く”状態にする。

- application 層は「内部分類 + `error_code` + 安全なメッセージ」を持つ失敗を返す
- presentation 層は framework が提供する変換（middleware / interceptor）を必ず通す
	- REST: problem details（`error_code`, `trace_id` 等を含む）へ自動変換
	- gRPC: canonical status code + `error_code`（metadata）へ自動変換
- 変換ルールの例外は ADR 必須とし、例外は lint/CI で検知できる入口（allowlist）を用意する

#### ログ/メトリクス出力（固定）

- エラー発生時のログは次を必須フィールドとして追加する
	- `error.kind`（分類: validation/authz/conflict/dependency/internal 等）
	- `error.code`（= `error_code`）
	- `error.message`（要約。機密情報は含めない）
	- `http.status_code` または `grpc.status_code`
- メトリクスは失敗を必ず集計できるようにする
	- 例: `k1s0.<service>.request.failures`（labels: protocol, route/method, status_code, error_code）
	- 例: 外部依存失敗: `k1s0.<service>.dependency.failures`（labels: dependency, error_kind）

---


