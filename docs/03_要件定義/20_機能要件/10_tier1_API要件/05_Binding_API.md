# Binding API

本書は、tier1 が公開する Binding API の機能要件を定義する。外部システム（オブジェクトストレージ、メール、HTTP、定期実行）との入出力を、Dapr Binding Building Block のファサードで提供する。

## API 概要

Output Binding は tier2 から外部システムへの一方向送信（例: MinIO へのファイル保存、SMTP によるメール送信）。Input Binding は外部イベント（例: Cron による定期実行、HTTP Webhook 受信）をトリガとする tier2 の処理起動。

内部実装は Dapr Binding Building Block を利用し、バックエンドは MinIO / SMTP / HTTP / k8s CronJob を Component YAML で定義する。

## 機能要件

### FR-T1-BINDING-001: MinIO Output Binding

**現状**: tier2 が MinIO SDK を直接使うと、S3 互換の認証情報管理、bucket 命名、権限管理を個別に実装する。バケット名にテナント ID を含める規約の徹底が手作業になる。

**要件達成後**: `k1s0.Binding.Send("minio", { bucket, key, data })` で MinIO にオブジェクトを書き込む。bucket は tier1 が `tenant-<id>-<logical-bucket>` 形式で自動展開。アクセス権限は OpenBao Database Engine 相当の動的 STS でプーリング（Phase 2+）。

**崩れた時**: テナント越境ファイルアクセスが発生するリスク、MinIO SDK バージョン差による tier2 アプリの挙動ばらつきが発生する。

**受け入れ基準**:
- bucket 名に tenant_id が自動付与され、tier2 から見えない
- 他テナントの bucket を指定すると `K1s0Error.Forbidden` を返す
- オブジェクトサイズ上限 5GB（MinIO multipart upload を内部で使用）
- 保存結果の URL / ETag が返される

### FR-T1-BINDING-002: SMTP Output Binding

**現状**: tier2 がメール送信を行うには、SMTP ライブラリ、From / To 検証、HTML テンプレート、添付ファイル処理を個別実装する必要がある。

**要件達成後**: `k1s0.Binding.Send("smtp", { to, subject, body_html, body_text, attachments })` で社内 SMTP サーバへ送信する。送信元アドレスは tenant_id に紐づくデフォルトアドレスから自動決定。添付ファイルは MinIO 経由または直接バイト列で指定可能。

**崩れた時**: tier2 がメール送信ライブラリを個別選定し、送信元なりすまし、SPF 設定の食い違い、大量送信のレート制限違反が発生する。

**受け入れ基準**:
- 送信元は tenant_id 単位で事前設定、tier2 は上書き不可
- レート制限（分あたり 100 通 / tenant デフォルト）を Component YAML で調整可能
- 送信失敗時は `K1s0Error.RelayFailed` を返す
- 送信履歴は Audit API に記録

### FR-T1-BINDING-003: HTTP Output Binding

**業務根拠**: BR-PLATGOV-004（SSRF 攻撃踏み台化リスクの構造的封じ込め）。

**現状**: tier2 が外部 HTTP API を呼ぶには、各言語の HTTP クライアント、リトライ、タイムアウト、認証ヘッダ管理を個別実装する。外部 URL の allowlist 管理が tier2 ごとにバラつく。社内既存システムでは 2023 年度に 1 件の SSRF 類似インシデント（外部 API 誤設定で意図しない内部リソースへアクセス）が発生し、調査・是正に合計 240 人時を要した。allowlist を各アプリ実装に委ねる構造は、20 チーム × 年 2 件程度の設定ミス発生を前提に、累計 960 人時/年 のリスクを抱え続けることになる。

**要件達成後**: `k1s0.Binding.Send("http", { method, url, headers, body })` で外部 HTTP API を呼び出す。URL の allowlist は Component YAML で tenant 単位に管理し、allowlist 外の URL への送信は `K1s0Error.Forbidden` を返す。allowlist 管理を tier1 で集約することで、SSRF リスクは構造的に封じ込められ、セキュリティ部門の承認プロセスも Component YAML PR レビュー 1 箇所に集約される。

**崩れた時**: tier2 が許可外の外部 URL にアクセスし、データ流出や SSRF 攻撃の踏み台となるリスクが発生する。インシデント発生時は原因調査に数百人時、外部報告義務が発生する場合は法務・広報連携で追加 500 人時規模の対応が必要。

**動作要件**:
- URL の allowlist 管理は tenant 単位
- allowlist 外 URL への送信は禁止
- HTTP ヘッダの機密情報（Authorization 等）は自動マスキングでログ出力

**品質基準**:
- allowlist 違反は NFR-E-NW-001（外部通信制御）に従い Audit に記録
- HTTP リクエストレイテンシは NFR-B-PERF-002 に従う

### FR-T1-BINDING-004: Input Binding（定期実行）

**現状**: tier2 の定期バッチは k8s CronJob として個別マニフェスト管理される。cron スケジュールの変更にはマニフェスト更新が必要で、実行履歴と監査ログの連携が弱い。

**要件達成後**: `@k1s0.Binding.OnSchedule(cron="0 */3 * * *")` 相当のアノテーションで tier2 のハンドラを cron 起動する。実行履歴は Audit API に自動記録、失敗時のリトライは Workflow API 経由で制御可能。

**崩れた時**: tier2 の定期処理が個別マニフェスト管理となり、cron 変更の都度マニフェスト PR が発生する。実行失敗時の検知が属人的になる。

**受け入れ基準**:
- cron 式は標準 5 フィールド表記
- 実行開始と完了の Audit 記録
- 失敗時のリトライポリシーを指定可能
- Phase 1c で提供、Phase 1b では k8s CronJob を手動マニフェスト管理

## 入出力仕様

本 API の機械可読な契約骨格（Protobuf IDL）は [40_tier1_API契約IDL.md の 05. Binding API セクション](../40_tier1_API契約IDL.md#05-binding-api) に定義されている。SDK 生成・契約テストは IDL 側を正とする。以下は SDK 利用者向けの疑似インタフェースであり、IDL の `BindingService` RPC と意味論的に対応する。

```
k1s0.Binding.Send(
    binding_name: string,
    operation: string,   // "create" | "send" | "get" 等、Binding 種別で異なる
    data: map<string, any>,
    metadata?: map<string, string>
) -> (response: map<string, any>, error: K1s0Error?)

k1s0.Binding.Subscribe(
    binding_name: string,
    handler: func(event, context) -> error?
) -> Subscription
```

エラー型には `BindingNotFound`、`Forbidden`（allowlist 外）、`RelayFailed`、`RateLimitExceeded` を追加。

## 受け入れ基準（全要件共通）

- すべての Binding 操作が Audit API に記録される
- Binding Component YAML は tier1 が集中管理、tier2 は参照のみ可能
- Binding 失敗時のエラーメッセージは機密情報を含まない

## Phase 対応

- **Phase 1a**: 未提供
- **Phase 1b**: FR-T1-BINDING-001、002、003（MinIO / SMTP / HTTP Output）
- **Phase 1c**: FR-T1-BINDING-004（Cron Input）
- **Phase 2+**: HTTP Webhook Input、Kafka Binding、Service Bus Binding 等

## 関連非機能要件

- **NFR-E-NW-001**: 外部 URL allowlist
- **NFR-E-MON-002**: Binding 操作の Audit 記録
- **NFR-B-RES-002**: MinIO ストレージ拡張性
- **NFR-A-CONT-003**: SMTP サーバ障害時の degrade（キューイング）
