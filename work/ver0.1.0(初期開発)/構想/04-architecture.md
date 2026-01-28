## 4. アーキテクチャ

- マイクロサービス（API First）
- クリーンアーキテクチャ

マイクロサービスの粒度は機能単位とし、複数機能をまとめない。

例: ログイン機能、データベース機能、ログ機能

### 通信方式（MUST）

- サービス間通信は gRPC を基本とする（内部 API）。
- 外部システムへ情報提供する場合のみ REST API（HTTP + OpenAPI）を提供する。
	- 外部公開の要件が無いサービスは、原則 gRPC のみでよい。

### サービス間通信の実運用設計（MUST）

gRPC を「方針」で終わらせず、サービス間通信の運用品質（可用性 / 安全性 / 観測性）をテンプレと framework で固定する。

#### 責務分担（固定）

開発基盤チーム（framework / テンプレの責務）

- 名前解決（Service Discovery）
	- Kubernetes を前提とし、K8s DNS + Service 名で解決する（例: `{service_name}.{namespace}.svc.cluster.local`）。
	- アプリケーションが独自に Consul 等へ依存して解決しない（必要なら ADR で例外化）。
- mTLS（サービス間）
	- Service Mesh で mTLS を強制し、証明書配布/ローテーションは Mesh 側に寄せる（アプリは証明書管理をしない）。
	- Mesh の選定候補: Istio / Linkerd（採用は別途 ADR で固定）。
- LB / レジリエンス
	- 負荷分散・アウトライヤー検知・接続プール等は Mesh/Proxy 側を第一選択とする。
	- リトライ/タイムアウトの標準値とルールをテンプレに同梱し、逸脱を検知できる形にする。
- 観測性（必須）
	- gRPC の分散トレース（TraceContext）を標準で伝播し、メトリクス/ログと相関できるようにする（OTel）。
	- 通信失敗（timeout / retry / circuit breaker）をメトリクスとして標準出力する。

個別機能チーム（各サービス実装の責務）

- gRPC 呼び出しは必ず deadline（timeout）を指定する（無制限禁止）。
- リトライは冪等（idempotent）な RPC のみ許可し、RPC 単位で明示する（禁止が既定）。
- 失敗時の扱い（フォールバック / エラー返却）は application 層のユースケースで定義する。
- 依存先サービスの契約（proto）と互換性ルールを守る（`buf lint` / `buf breaking` を通す）。

#### リトライ/タイムアウト/サーキットブレーカのルール（固定）

- Timeout（必須）
	- すべての gRPC 呼び出しで deadline を設定する。
	- 既定値はテンプレで固定し、設定（YAML/DB）で上書き可能にする。
- Retry（原則禁止・例外のみ）
	- 既定は 0 回（retry なし）。
	- 例外として許可するのは読み取り系など冪等な RPC のみ。
	- 許可する場合も、指数バックオフ + ジッター、最大回数/最大待ち時間を必ず設定する。
- Circuit Breaker（Mesh 優先）
	- 可能な限り Mesh/Proxy 側のアウトライヤー検知（連続失敗時の一時遮断）を利用する。
	- アプリ側で実装が必要な場合は framework crate が提供する共通実装を利用し、サービスごとの独自実装は禁止（必要なら ADR）。

#### 実装の固定（テンプレ / framework が提供）

- framework crate に共通 gRPC クライアント初期化を用意する（コネクション / OTel / timeout の標準設定）。
- 共通初期化は「gRPC client の作成」と「呼び出し作法」を API 形状で固定し、サービスが生の Channel/Client を直接組まない（必要なら ADR で例外化）。
- 依存先アドレス指定は「サービス名 + namespace」等の論理名を基本とし、環境差は config で吸収する。
- Mesh を使う場合、アプリの gRPC は原則「平文（h2c）+ sidecar で mTLS」等、運用方式をテンプレに明記して統一する（方式は ADR で確定）。

##### gRPC クライアント標準化（固定）

目的: サービス間通信の品質（無限待ち/観測欠落/再試行増幅）を「実装者の作法」ではなく framework の既定で防ぐ。

MUST（必須）

- deadline（timeout）必須
	- すべての RPC は deadline を指定する。未指定は framework が禁止（もしくは既定 timeout を必ず付与し、未指定であることをログ/メトリクスに出す）。
	- 既定 timeout はテンプレで固定し、設定（YAML/DB）で上書き可能にする。
	- timeout の下限/上限を固定し、逸脱は設定バリデーション/起動時失敗で早期に落とす。

- retry の既定は 0（原則禁止）
	- 既定ではリトライしない（0回）。
	- 例外を許可する場合は「冪等 RPC のみ」「回数/最大待ち/指数バックオフ+ジッター」を必須とする。

- OTel 伝播と計測の既定有効
	- W3C Trace Context を必ず伝播し、依存呼び出し（他サービス）を span として計測する。
	- 成功/失敗/timeout/retry のメトリクスを標準で出す。

- 共通 interceptors を提供
	- trace context 伝播、`error_code` の受け渡し、`request_id` 相関（採用する場合）、必要なメタデータ（例: tenant）付与を interceptor で自動化する。
	- 個別サービスが “付け忘れ/形式違い” を起こせない形を既定にする。

##### リトライの例外管理（opt-in + ADR + lint）（固定）

目的: 例外を「その場の判断」で増やさず、可視化して減らせる形で管理する。

- 例外は opt-in のみ許可し、コード上で明示する（例: 依存クライアントのビルダーに `RetryPolicy` を渡す、もしくは RPC ごとの allow 指定を行う）。
- opt-in した箇所は lint/CI が検知できるようにし、次を満たさない場合は失敗させる。
	- 冪等性の根拠（読み取り系等）が説明できる
	- 回数/バックオフ/ジッター/最大待ちが設定されている
	- ADR 参照（例: `docs/adr/ADR-xxxx-*.md`）が紐づいている

##### Service discovery の統一（固定）

目的: アプリが独自の discovery を持たず、環境差は config で吸収する。

- Kubernetes を前提に、K8s DNS + Service 名で解決する（例: `{service_name}.{namespace}.svc.cluster.local`）。
- アプリが Consul 等へ依存して解決しない（必要なら ADR）。
- 依存先指定は “論理名” を正本とし、環境差（namespace/cluster domain 等）は `config/{env}.yaml`（非機密）で吸収する。

#### Service Mesh とローカル開発の境界（固定）

- ローカル開発（Docker Compose）
	- Mesh なしを既定とする。
	- gRPC/HTTP は TLS なし（h2c / http）を許容する。
	- 観測性（OTel）はアプリ側で必ず有効にし、Collector に送る。
	- レジリエンス（timeout / retry / circuit breaker）はアプリ側ルールを適用する（Mesh 依存を前提にしない）。
	- 同じコードで Kubernetes（Mesh あり/なし）へ持ち込めるよう、クライアント初期化は framework の共通入口に寄せる（ローカル専用の分岐を増やさない）。

- Kubernetes（dev/stg/prod）
	- Mesh 導入時は mTLS を Mesh 側で強制し、アプリは証明書管理をしない。
	- LB/アウトライヤー検知/接続プール等は Mesh/Proxy 側を第一選択とする。
	- アプリ側では deadline（timeout）とトレース伝播（OTel）を必須とする。

- 例外（ADR 必須）
	- ローカルで TLS/mTLS が必須な要件がある場合
	- アプリ側で独自の Service Discovery/リトライ制御等を持たせる必要がある場合

---


