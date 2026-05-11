---
id: arch.tier1.libraries
axis: tier1
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.tier1.tier1_index
  - arch.tier1.server_systems
  - arch.tier1.feature_categories
  - detail.tier1.migration_pair_conformance
  - detail.tier1.oss_lifecycle_conformance
  - detail.tier1.tier1_enforcement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# tier1 Library 設計方針

## 一文方針
- tier1 Library は機能カテゴリ単位の抽象 API を tier2 / tier3 に提供し、現行採用 OSS は実装の詳細として表面 API に漏らさない。tier2 / tier3 は本 Library のみを通じて infra にアクセスし、OSS のクライアントライブラリを直接 import する経路は CI / lint / 公開 API snapshot / 内部 registry の 4 層で物理 enforce で塞ぐ。

## パッケージ構成（frontend / backend / core 分離）
- Library は core / frontend / backend の 3 パッケージで構成し、対象言語ごとにこの 3 パッケージを提供
- 両者で重複する概念は core に集約し、frontend / backend は core に依存
- core に置く対象例:
  - 認証コンテキスト / テナント識別子の型と伝播ヘルパ
  - 構造化ログ / トレース / メトリクスの共通スキーマ
  - 共通エラー型、リトライ / タイムアウト等のポリシー定義
- frontend パッケージにバックエンド専用の依存（DB ドライバ、Kafka クライアント、Temporal クライアント等）が混入することを禁止する
- 「対象」が both のカテゴリでも、具象 API は frontend / backend で異なって構わない（例: Authentication は frontend で OIDC PKCE クライアント、backend でトークン検証 / 認可ポリシー評価）。core が共有するのは型と概念であり、API 表面の完全一致は要求しない

## 3 抽象化レベル

### L3（OSS 中立）
- 業界標準の wire / protocol（OIDC / OTLP / RESP / S3 / gRPC 等）が存在し、複数 OSS 実装が同じ表面を提供する領域に限定
- OSS 固有の型 / 概念を API 表面に一切出さない
- 受け入れ基準: OSS 差し替え時、業務コード（tier2 / tier3）は無変更
- 想定対象: Logging / Tracing / Metrics / Authentication / Secret / RPC / KeyValue / Object Storage

### L2\*（軽セマンティクス + 真に互換性の高い OSS 族）
- 族の semantics 差が API 表面に出ても破壊的でない領域に限定。高セマンティクス領域（トランザクション境界・ロック挙動・採番・順序保証等が業務ロジックに染み出す領域）には適用しない
- OSS 固有の語彙は Library 独自語彙に翻訳。族内で意味を保てる範囲のチューニング概念（距離関数 / 互換性モード / evaluation context / プロファイル種別等）は API 表面に残す
- カテゴリごとに「同族」を明示列挙し、その範囲内での差し替えを保証対象とする
- 受け入れ基準: 族内差し替え時、業務コードは型置換 + 一部チューニング（パラメータ調整・索引再設計など）で済み、業務ロジックの再設計は不要
- 想定対象: Profiling、Feature Flag、Schema Registry
- 同族の具体定義は [提供機能カテゴリ](04_提供機能カテゴリ.md) を参照

### L1+（単一 OSS 深耕 + 移行コミットメント）
- 高セマンティクス領域で、OSS の semantics 差が業務ロジックに必然的に染み出す領域。複数 OSS 同時サポートを諦める代わりに、単一 OSS の全機能（拡張・運用機能を含む）を Library API として深く表現
- Library の責務:
  - 単一 OSS の全機能を Library API として漏れなく表現する。L3 / L2\* で要求される「OSS 概念の隠蔽」は意図的に放棄
  - 横断要素（認証コンテキスト伝播 / 自動計装 / リトライ・タイムアウト / マルチテナント識別子 / 共通エラー型 / ベストプラクティステンプレート）を強制
  - OSS バージョン跨ぎ（マイナー / メジャー）の互換維持と移行支援
  - OSS ライフサイクルイベント時の移行 toolchain を成果物として継続保持
- 受け入れ基準: OSS クライアントを直接 import させない経路を Library として一本化し、ライフサイクルイベント時の影響範囲が「Library + 該当カテゴリの業務コード + tier1 が提供する移行 toolchain」の三点に閉じること
- 露出される OSS 概念は [提供機能カテゴリ](04_提供機能カテゴリ.md) の露出概念一覧に明示
- 想定対象: Relational Store / Single-leader、Messaging / EventBus、Workflow / Long-running Saga、Rule Engine、Vector Search

### Lv 共通
- L1+ / L2\* で API 表面に露出する OSS 概念は、カテゴリごとに「露出概念一覧」として Library 側ドキュメントに明示
- 抽象化レベルが低いことを理由に Library から外さない。代わりに L1+ / L2\* を選び、tier2 / tier3 が OSS を直接 import する経路を塞ぐことを優先

## 横断的に Library が担う共通要素
- 認証コンテキスト（誰が呼んでいるか）の伝播
- トレース / ログ / メトリクスへの自動計装
- リトライ / タイムアウト / サーキットブレーカ等のレジリエンス
- マルチテナント識別子の伝播と分離

## 強制機構（OSS 直接 import 禁止の機械的担保）
3 層 defense-in-depth、いずれの層も CI 必須チェックとして組み込む（後付け運用 / 例外申請に依存しない）。詳細は [tier1 強制機構](../../04_詳細設計/02_強制機構/01_tier1強制機構.md) を参照。

### 層 1: 依存導入レベル（tier2 / tier3 リポジトリ側）
- Rust: `cargo-deny` の `[bans]` で Kafka / PostgreSQL / Temporal / Valkey / Object Storage 等のクライアント crate を deny
- C# (.NET 8+): BannedApiAnalyzers の `BannedSymbols.txt` で OSS の名前空間を禁止し、Central Package Management で未承認パッケージの参照自体を弾く
- Go: golangci-lint の `depguard` で OSS クライアントの import パスを deny
- TypeScript: `eslint-plugin-import` の `no-restricted-imports` と `eslint-plugin-boundaries` で禁止と層越境を検出
- 上記設定は Backstage の Software Template に同梱し、tier2 / tier3 リポジトリ生成時点で組み込む

### 層 2: Library 公開 API 表面レベル（tier1 側）
- Library が OSS 型を再エクスポートすると業務コードが間接的に OSS 概念へ依存するため、Library 公開 API を snapshot 化し差分を CI で検査
  - Rust: `cargo public-api`
  - C#: `Microsoft.CodeAnalysis.PublicApiAnalyzers`（`PublicAPI.Shipped.txt` 運用）
  - Go: `go-apidiff` と自製 `go/analysis` を併用
  - TypeScript: `api-extractor` の `.d.ts` rollup
- L3 / L2\* カテゴリは公開シグネチャに OSS 型が現れたら CI fail。L1+ カテゴリは「[提供機能カテゴリ](04_提供機能カテゴリ.md)」の露出概念一覧と allowlist 一致を検査

### 層 3: 供給経路レベル（内部パッケージリポジトリの一元化）
- tier2 / tier3 のビルドが OSS クライアントを物理的に取得できないよう、内部パッケージリポジトリを唯一の取得元に固定
  - Rust: 内部 cargo registry を `[source.crates-io]` で置換
  - C#: 内部 NuGet feed のみ `nuget.config` に登録
  - Go: Athens 等の内部 module proxy を `GOPROXY` に固定
  - TypeScript: Verdaccio 等の内部 npm registry に固定
- 内部リポジトリには Library と承認済み補助パッケージのみを公開し、Kafka / Temporal / PostgreSQL 系等の OSS クライアントは置かない。Library リポジトリの CI に限り、外部 feed を許可

## L2\* 同族保証の機械的担保

### 二バックエンド conformance（最重要）
- 各 L2\* カテゴリで、本命 OSS と同族内代替 OSS の最低 2 実装を Testcontainers で同時に立ち上げ、共通の統合テストを両方で green にすることを merge 条件
- 想定の組合せ:
  - Profiling: Parca + Pyroscope
  - Feature Flag: flagd + GO Feature Flag
  - Schema Registry: Apicurio + Karapace
- 片側でしか通らないテストが発生した時点で「その API は L2\* として適格でない」とみなし、API を族内ポータブル範囲まで縮める、または当該カテゴリを L1+ へ昇格

### capability negotiation
- Library の初期化 API で、業務コード側が要求する能力（互換性モード / 距離関数 / evaluation context 種別 / プロファイル種別など）を宣言
- backend が要求能力を満たさない場合、起動時に fail-fast。本番投入後の差し替えで silent な意味論変化を防ぐ

### 露出概念 / 禁止概念の機械可読宣言
- カテゴリごとに YAML で許可概念 / 禁止概念を明示
- 公開 API snapshot と CI で照合し、禁止概念に該当する語が API 表面に出現したら fail
- これは「強制機構」層 2 と同じ枠で動かす

## 出荷時のバックエンド方針
- L3 カテゴリ（wire 互換）: 複数 OSS が wire 標準で互換のため、Library としてはどの実装でも利用可能。Backstage Software Template には初期オプションとして以下を含める
  - KeyValue / Cache: Valkey、Dragonfly
  - Object Storage: Ceph RGW、MinIO
- L2\* カテゴリ（同族）: CI conformance で同族の 2 実装の green を merge 条件。日常運用で並走させるかどうかは別判断
- L1+ カテゴリ: 本命 OSS のみを Backstage template の標準として提供。並走は行わない。移行は OSS ライフサイクルイベント発生時に tier1 が toolchain で支援。並走しないことを「責任放棄」にしないため、移行 toolchain の年次 dry-run を実施

## L1+ 移行コミットメント

### 位置づけ
- L1+ カテゴリは単一 OSS への深耕を選ぶ代わりに、OSS ライフサイクルイベントで発生する移行作業を Library / tier1 が toolchain として成果物化。day-1 で複数 backend を並走させる方式は採らない

### 移行 toolchain の成果物（カテゴリごとに整備）
- schema / state 差分検出（DB なら schema diff、Workflow なら Workflow / Activity 定義の差分、Rule Engine なら決定表 DSL の差分、Messaging なら topic / consumer group / Schema Registry エントリの差分）
- 双方向 ETL / replication（Kafka なら MirrorMaker 2 系、PG なら logical replication / pg_dump、Temporal なら state export / replay）
- dual-write 移行支援（移行期間中、新旧 backend に二重書き込みするための Library 抽象）
- observability 連続性（移行前後で span / metric の意味論を維持する Collector プロセッサ設定とゴールデン値テスト）
- 業務コード移行ガイド（露出概念一覧を起点に、tier2 / tier3 側の修正点を機械的に列挙したチェックリスト）

### 維持責務
- 上記 toolchain は「ライフサイクルイベント時のみ作る」のではなく、Library 本体と同じリポジトリで継続的に維持。Library のバージョンアップ時に toolchain も同時に CI で検証
- 移行先候補（カテゴリごとに 1 つ以上）を文書化し、Testcontainers での dry-run 移行テストを年次で実施。dry-run の green を Library リリース条件に含める

### 移行先候補（参考、確定ではない）
- Relational Store / Single-leader: 別 PG operator（CNPG ↔ StackGres ↔ Crunchy PGO ↔ EDB Postgres）、Aurora PG、Neon
- Messaging / EventBus: RedPanda、AWS MSK、Confluent Cloud
- Workflow / Long-running Saga: Cadence、Temporal Cloud
- Rule Engine: 自製 DSL、別 OSS
- Vector Search: PG ecosystem 内のみ toolchain 保証（pgvector の index 種別 / 距離関数変更、別 PG operator への移行）。族外 standalone vector engine（Qdrant / Milvus / Weaviate）への移行は業務コード再設計が必要、L1+ toolchain 保証範囲外、独立プロジェクトとして 14_OSS ライフサイクル適合仕様 の `project_spawn` 経路で扱う

1.0.0 ship 前 dry-run green 必須 pair の確定形と機械可読仕様は [移行 Pair 適合仕様](../../04_詳細設計/01_適合仕様/02_移行Pair適合仕様.md) を参照。

## Companion（Library 非対応言語向け追加提供）

### 位置づけ
本 Library を組み込めない言語（.NET Framework など）に対し、二つの役割を担う薄物として提供:
- 役割 A: Observability / 認証コンテキスト伝播
- 役割 B: Transport Negotiation Runtime（bidi を含む RPC の transport 選択と等価実装。tier1 Server 系 の Transport Adapter Layer と対をなすクライアント実装）

### 共通スコープ
- ビジネス API 抽象（Messaging / Relational Store / Workflow など）は含まない。ビジネス API のスキーマ / セマンティクスは引き続き Gateway 経由で `.proto` を真として利用
- Companion は「proto に書かれた意味論を、レガシー言語の HTTP / IO スタックの上で透過に実行する runtime」と位置付ける

### 役割 A: Observability / 認証コンテキスト伝播
- 射程は span / log / metric の自動生成および認証コンテキスト伝播のみで、transport stack には及ばない
- 標準形態: .NET Framework 4.6.2+ 向けに OpenTelemetry .NET Auto-Instrumentation を採用。CLR Profiler によるアタッチ方式
- 補助形態: Auto-Instrumentation 非対応環境（4.6.1 以前など）向けに NuGet 参照型の Companion を提供
- tenant / subject attribute の注入経路: CLR Profiler の標準計装のみでは Authorization header 中の JWT claim を decode して span attribute に注入する経路を持たない。`k1s0.Companion.NetFx.OTelExt`（splunk-otel-dotnet fork）が CLR Profiler の IL rewrite 経由で登録する k1s0 processor で行う（4 stack 全体の主経路は IL rewrite 側、DelegatingHandler は HttpClient 経路の補助）
- core スキーマとの整合: 構造化ログ / span attribute 名は言語中立のスキーマ定義（YAML、OpenTelemetry Semantic Conventions 拡張形式）を単一の真とし、OpenTelemetry Weaver で各言語の定数コードを生成

詳細は [Companion OTel 拡張](../../04_詳細設計/03_クロスカッティング適合仕様/11_companion_otel_extension.md) を参照。

### 役割 B: Transport Negotiation Runtime
- tier1 Server 系 Transport Adapter Layer と対をなすクライアント実装
- 提供 API: bidi RPC に対しては言語ごとに統一インタフェース（C# 例: `IBidiChannel<TReq, TResp>`）を提供し、その裏で Gateway が選んだ `chosen_transport` に応じた adapter 実装が動作。アプリ側は transport 種別を意識しない
- 同梱 adapter（Companion 側受信実装）:
  - `sse_paired`: 既定。`HttpClient.ResponseHeadersRead` を用いた SSE フレーミングパーサ + 送信用 unary POST。`resume_token` は SSE 仕様の `id:` フィールドをそのまま使う（Last-Event-ID リクエストヘッダで再接続時に server へ送出）
  - `long_poll`: cursor 付き short poll の組合せ
  - `webhook`: レガシー側で IIS / WCF / HttpListener が利用可能な場合の opt-in
  - `websocket`: ClientWebSocket ベース。.NET Framework 4.6.2+ で動作確認可能な範囲のみ提供
  - `web_transport`: .NET 8+ 等、QUIC / WebTransport クライアントを持つ環境向け。.NET Framework Companion には載せない。v1 では opt-in adapter 扱い
  - `messaging_bridge`: Kafka / AMQP の REST Proxy / 単純クライアント越しに bidi メッセージを搬送する形態
- Conformance proof: 各 adapter は tier1 Server 系 の Transport Adapter Layer と同一の conformance scenario corpus に対して green を merge 条件
- capability 宣言: Companion は起動時に自身が利用可能な adapter 一覧 / TLS バージョン / inbound 可否 / max message size 等を Open RPC の `client_capabilities` として送出
- resume の透過化: `resumable: REQUIRED` の bidi RPC は、Companion が transport 切替を跨いで session を継続。アプリは `onMessage` / `send` の API しか見えず、再接続は Companion 内部で完結
- スコープ外: proto に書かれた意味論を実行することに留め、Library が提供する機能カテゴリの抽象は持たない

### 長期収斂
- tier1 側の Adapter Layer 退役に追随し、Companion 側 adapter も同期して退役。Adapter の追加 / 退役はいずれもアプリのコード変更を伴わない
- レガシー言語そのものが対象 fleet から退役した時点で Companion 自体の保守も終了。その時点で Proto 二層化も単層へ collapse

## 関連参照
- [tier1 設計方針 index](README.md)
- [Server 系](01_Server系.md)
- [提供機能カテゴリ](04_提供機能カテゴリ.md)
- [tier1 強制機構](../../04_詳細設計/02_強制機構/01_tier1強制機構.md)
- [移行 Pair 適合仕様](../../04_詳細設計/01_適合仕様/02_移行Pair適合仕様.md)
- [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [Companion OTel 拡張](../../04_詳細設計/03_クロスカッティング適合仕様/11_companion_otel_extension.md)
