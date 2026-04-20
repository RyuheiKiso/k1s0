# Service Invoke API

本書は、tier1 が公開する Service Invoke API の機能要件を定義する。tier2/tier3 のサービス間同期呼び出しを、Dapr Service Invocation の stable Go SDK で実装するファサードとして提供する。

## API 概要

tier2 の業務アプリ A が、別の tier2 業務アプリ B の公開エンドポイントを同期的に呼び出すユースケースを主対象とする。tier3 の UI サーバから tier2 ドメインサービスを呼び出す、tier2 複数サービス間の REST/gRPC 呼び出し、といった単発リクエスト / レスポンスも対象。長期実行や非同期配信は Workflow API / PubSub API の責務であり、本 API では扱わない。

内部的には Dapr Service Invocation を使い、サービスディスカバリは Dapr の名前解決（k8s Service ベース）を流用する。tier2/tier3 からは Dapr 非依存に見えるよう、SDK は `k1s0.Invoke("service-b", "method", req)` のような自然な言語ネイティブ呼び出しに抽象化する。

## 機能要件

### FR-T1-INVOKE-001: 同期サービス呼び出し（gRPC）

**現状**: 素の Dapr Go SDK を使うと、`daprd` サイドカーを介した gRPC 呼び出しで tier2 開発者は Dapr の AppID・namespace・metadata ヘッダを意識する必要がある。C# からは Dapr C# SDK が必要だが、stable 版の機能差と .NET Framework 4.x との互換性で実装が分岐する。

**要件達成後**: tier2 開発者は各言語 SDK の `k1s0.Invoke(<target>, <method>, <request>)` を呼ぶだけで、内部で Dapr Service Invocation が実行される。AppID・namespace の解決、認証トークン伝搬、分散トレース span 生成、エラー型変換は tier1 ファサードが自動処理する。.NET Framework 4.x は HTTP/1.1 互換プロキシ経由で同じ体験を得る（FR-T1-INVOKE-002）。

**崩れた時**: tier2 開発者は Dapr の実装詳細（daprd サイドカーの存在、AppID 命名規則、名前解決の失敗モード）を学習する必要がある。言語間で呼び出し API がバラつき、結合テストで毎回紛糾する。Dapr のバージョンアップで signature が変わると tier2 全件が影響を受ける。

**動作要件**（実装すべき機能、本要件定義で確定）:
- Go / C# / Rust / Python の各 SDK で同名のメソッド（`k1s0.Invoke`）を提供する
- 対象サービスが存在しない場合、`K1s0Error.ServiceNotFound` を返す（Dapr 生エラーを露出しない）
- 呼び出しに失敗した場合、`K1s0Error` のいずれかのバリアントで原因を明示する

**品質基準**（検収時の非機能目標、NFR とテスト計画で検証）:
- p99 レイテンシは NFR-B-PERF-001（< 500ms、中規模 150 RPS）に従う
- Dapr SDK 固有の import が tier2 コードに出現しないこと（[70_プロジェクト管理/04_テスト計画.md](../../70_プロジェクト管理/04_テスト計画.md) の「契約テスト」で CI 検証、`grep -r "dapr.io" <tier2-code>` がゼロ）
- 性能試験手順は [70_プロジェクト管理/04_テスト計画.md](../../70_プロジェクト管理/04_テスト計画.md) を参照

### FR-T1-INVOKE-002: HTTP/1.1 互換プロキシ

**現状**: .NET Framework 4.x は gRPC クライアントライブラリのサポートが貧弱で、ネイティブ gRPC 呼び出しのための依存追加・ビルド変更が重い。レガシー資産の最小改修で k1s0 を利用するには HTTP/1.1 互換が必要。

**要件達成後**: tier1 が HTTP/1.1 エンドポイント（`POST /invoke/<target>/<method>` 形式）を提供し、内部で gRPC に変換して対象サービスを呼び出す。.NET Framework 4.x アプリは既存の `HttpClient` をそのまま使える。レスポンスは JSON エンコーディングで返される。

**崩れた時**: .NET Framework 4.x 資産の呼び出しコードを全面書き換える必要が生じ、移行コストが数人月単位で増える。既存資産との共存目標（10_業務要件/01_業務背景.md 参照）が崩れる。

**受け入れ基準**:
- `POST /invoke/<service>/<method>` 形式のエンドポイントを提供
- gRPC ステータスコードを HTTP ステータスコードに適切にマッピング（例: UNAVAILABLE → 503）
- JSON ↔ Protobuf 変換の整合（oneof、enum、timestamp の扱い）を明示
- Phase 1a では未提供、Phase 1b で提供

### FR-T1-INVOKE-003: タイムアウト・リトライ制御

**現状**: 素の Dapr でタイムアウトを設定するには、Component YAML と gRPC context で別々の設定が必要。リトライポリシー（指数バックオフ、回数制限）は Dapr Resiliency 機能で YAML 定義するが、tier2 開発者が直接 YAML を書くのは避けたい。

**要件達成後**: SDK メソッドのオプション引数でタイムアウト秒数とリトライ回数を指定する。デフォルトはタイムアウト 5 秒、リトライ 3 回（指数バックオフ、100ms → 200ms → 400ms）。tier1 ファサードが Dapr Resiliency ポリシーに変換する。

**崩れた時**: tier2 開発者は個別に context.WithTimeout やリトライループを実装する羽目になり、挙動がアプリごとにバラつく。サービスメッシュ側の設定と重複してタイムアウトが意図と違うタイミングで発火する。

**受け入れ基準**:
- タイムアウトは呼び出しごとに任意の秒数で上書き可能
- リトライ回数 0 指定で完全無効化可能
- タイムアウト発生時は `K1s0Error.Timeout` を返す
- リトライ試行はログに記録され、Grafana Tempo の span に attributes として残る

### FR-T1-INVOKE-004: サーキットブレーカ

**現状**: 素の Dapr には Circuit Breaker 機能があるが、状態（closed / open / half-open）の可視化とチューニングが YAML 編集経由となり、実運用のフィードバックループが遅い。

**要件達成後**: tier1 ファサードが Circuit Breaker の状態を Prometheus metrics として公開する。閾値（連続失敗回数、half-open 試行間隔）は Dapr Component YAML で管理するが、チューニングは Grafana ダッシュボード → YAML 改訂の流れで回す。tier2 開発者は個別にブレーカを書かない。

**崩れた時**: tier2 ごとにブレーカを実装するか、実装しないかのバラつきが発生する。下流サービスの障害が tier2 各アプリへ個別に波及する。

**受け入れ基準**:
- 連続失敗回数の閾値、half-open 遷移までの時間を Component YAML で設定可能
- Circuit Breaker の状態（closed / open / half-open）を Prometheus で可視化
- Phase 1a では未提供、Phase 1b で提供

### FR-T1-INVOKE-005: 認証トークン自動伝搬

**現状**: 素の Dapr は呼び出し間で Authorization ヘッダを自動伝搬しない（設定による）。tier2 開発者が context から抽出して次の呼び出しに付与するコードを書く必要がある。コピペ忘れで認証が切れるケースが頻発する。

**要件達成後**: 呼び出し元の request から抽出した JWT と W3C Trace Context を、tier1 ファサードが自動で伝搬する。tier2 開発者は認証トークンを意識しない。tenant_id クレームは tier1 側で呼び出し先でも検証される。

**崩れた時**: 認証が切れたまま下流呼び出しが成功してしまう、あるいは不必要にエラーになるケースが混在する。監査ログで「誰の操作か」が追えなくなる。

**受け入れ基準**:
- 呼び出し元 request の Authorization ヘッダが呼び出し先 request に自動付与される
- 呼び出し先で JWT が検証されない場合は `K1s0Error.Unauthorized` を返す
- tenant_id クレームが呼び出し先 request の metadata として展開される
- サービスアカウントトークン（ユーザー context がない場合）も同様に伝搬

## 入出力仕様

本 API の機械可読な契約骨格（Protobuf IDL）は [40_tier1_API契約IDL.md の 01. Service Invoke API セクション](../40_tier1_API契約IDL.md#01-service-invoke-api) に定義されている。SDK 生成・tier2/tier3 連携実装・契約テストはすべて IDL 側を正とし、本書は要件記述と疑似インタフェースで補足する。

### 疑似インタフェース（言語共通の利用イメージ）

疑似インタフェースを以下に示す。実 SDK の signature は言語別に調整するが、意味論は IDL の `InvokeService.Invoke` RPC と一致する。

```
k1s0.Invoke(
    target: string,           // 対象サービス名（IDL の InvokeRequest.app_id に対応）
    method: string,           // 呼び出しメソッド名（IDL の InvokeRequest.method に対応）
    request: Proto message,   // gRPC リクエスト（IDL の InvokeRequest.data + content_type に対応）
    options?: {
        timeout_seconds?: int,  // IDL の InvokeRequest.timeout_ms（ミリ秒、本書の秒指定から SDK で変換）
        retry_count?: int,      // IDL 未定義、SDK ローカルで Resiliency Policy に変換
        propagate_auth?: bool   // default true、IDL の TenantContext 自動設定
    }
) -> (response: Proto message, error: K1s0Error?)
```

SDK の option フィールド名と IDL の message フィールド名は必ずしも一致しない（言語慣習に合わせる）。対応関係はコメントで明示する。IDL 変更時は本表も同時更新する（[40_tier1_API契約IDL.md](../40_tier1_API契約IDL.md) の責任分界表の「API 要件ファイルとの対応」節参照）。

### エラー型 K1s0Error のバリアント

エラー型 `K1s0Error` は以下のバリアントを持つ。IDL の `ErrorDetail.code` と 1 対 1 対応する。

- `ServiceNotFound`: 対象サービスが解決できない（IDL code: `E-INVOKE-NOTFOUND`）
- `Unauthorized`: JWT 検証失敗（IDL code: `E-AUTH-UNAUTHORIZED`）
- `Forbidden`: tenant_id 越境等の認可失敗（IDL code: `E-AUTH-FORBIDDEN`）
- `Timeout`: タイムアウト（IDL code: `E-INVOKE-TIMEOUT`、gRPC status `DEADLINE_EXCEEDED`）
- `Unavailable`: 対象サービスが 503 / gRPC UNAVAILABLE（IDL code: `E-INVOKE-UNAVAILABLE`）
- `Internal`: その他のサーバエラー（IDL code: `E-INVOKE-INTERNAL`）

## Phase 対応

- **Phase 1a**: FR-T1-INVOKE-001、003、005（Go SDK、gRPC のみ）
- **Phase 1b**: FR-T1-INVOKE-002、004 を追加（HTTP/1.1 プロキシ、Circuit Breaker、C# SDK）
- **Phase 1c**: Python SDK 追加
- **Phase 2+**: Rust SDK（自作）追加、Feature Flag 連動のサーキットブレーカ

## 関連非機能要件

- **NFR-B-PERF-002**: tier1 API p99 < 500ms（定常 150 RPS）
- **NFR-A-CONT-001**: tier1 対外 SLA 稼働率 99%（24h 基準）／ NFR-I-SLO-001: 内部 SLO 99.9%
- **NFR-E-AC-001**: JWT 認証強制
- **NFR-E-AC-003**: tenant_id クレーム検証
- **NFR-C-IR-002**: Circuit Breaker 状態の監視
