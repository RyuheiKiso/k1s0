# Proto API リファレンス

本ドキュメントは k1s0 プロジェクトの gRPC Proto API 構造を一覧で管理する。
各サービスのパッケージ、RPC メソッド、主要メッセージ型を記載する。

---

## 共通パッケージ

### `k1s0.system.common.v1`

全サービスで共通利用される基盤メッセージ型。

| メッセージ型 | 説明 |
|------------|------|
| `Timestamp` | Unix 秒 + ナノ秒のタイムスタンプ |
| `Pagination` | ページネーションリクエスト（page, page_size） |
| `PaginationResult` | ページネーションレスポンス（total_count, page, page_size, has_next） |

---

## System ティア — サービス一覧

### `k1s0.system.auth.v1` — AuthService

認証・認可の基盤サービス。JWT トークン検証・ユーザー情報管理・パーミッション確認を提供する。

| RPC | 説明 |
|-----|------|
| `ValidateToken` | JWT トークン検証（TokenClaims を返す） |
| `GetUser` | ユーザー詳細取得 |
| `ListUsers` | ユーザー一覧取得（ページネーション・検索対応） |
| `GetUserRoles` | ユーザーのグローバルロール・クライアント別ロール取得 |
| `CheckPermission` | リソースアクセスの認可判定（role ベース） |

#### `k1s0.system.auth.v1` — AuditService

監査ログの記録・検索サービス。

| RPC | 説明 |
|-----|------|
| `RecordAuditLog` | 監査ログ記録（`AuditEventType` enum・`AuditResult` enum 対応） |
| `SearchAuditLogs` | 監査ログ検索（ページネーション・enum フィルタ対応） |

### `k1s0.system.config.v1` — ConfigService

設定値管理サービス。namespace/key ベースの設定 CRUD・スキーマ管理・ストリーミング監視を提供する。

| RPC | 説明 |
|-----|------|
| `GetConfig` | 設定値取得（namespace + key） |
| `ListConfigs` | namespace 内の設定値一覧取得（ページネーション・キー部分一致検索対応） |
| `UpdateConfig` | 設定値更新（楽観的排他制御: version フィールド） |
| `DeleteConfig` | 設定値削除 |
| `GetServiceConfig` | サービス名・環境指定の設定一括取得 |
| `WatchConfig` | 設定変更の Server-Side Streaming（`ChangeType` enum 対応） |
| `GetConfigSchema` | 設定エディタスキーマ取得 |
| `UpsertConfigSchema` | 設定エディタスキーマ作成・更新 |
| `ListConfigSchemas` | 設定エディタスキーマ一覧取得 |

### `k1s0.system.tenant.v1` — TenantService

テナント管理サービス。

| RPC | 説明 |
|-----|------|
| `CreateTenant` | テナント作成 |
| `GetTenant` | テナント取得 |
| `ListTenants` | テナント一覧取得 |
| `UpdateTenant` | テナント更新 |
| `DeleteTenant` | テナント削除 |

### `k1s0.system.featureflag.v1` — FeatureFlagService

フィーチャーフラグ管理サービス。

| RPC | 説明 |
|-----|------|
| `GetFlag` | フラグ取得 |
| `ListFlags` | フラグ一覧取得 |
| `CreateFlag` | フラグ作成 |
| `UpdateFlag` | フラグ更新 |
| `DeleteFlag` | フラグ削除 |
| `EvaluateFlag` | フラグ評価（コンテキストベース） |

### `k1s0.system.saga.v1` — SagaService

サーガオーケストレーションサービス。分散トランザクションの開始・追跡・補償・ワークフロー管理を提供する。

| RPC | 説明 |
|-----|------|
| `StartSaga` | Saga 開始（非同期実行） |
| `GetSaga` | Saga 詳細取得（ステップログ含む） |
| `ListSagas` | Saga 一覧取得（ページネーション・ステータス/相関ID フィルタ対応） |
| `CancelSaga` | Saga キャンセル |
| `CompensateSaga` | 補償トランザクション実行 |
| `RegisterWorkflow` | ワークフロー登録（YAML 文字列） |
| `ListWorkflows` | ワークフロー一覧取得 |

### `k1s0.system.dlq.v1` — DlqService

デッドレターキュー管理サービス。`DlqMessageStatus` enum（PENDING/RETRYING/SUCCEEDED/FAILED）をサポート。

| RPC | 説明 |
|-----|------|
| `ListMessages` | DLQ メッセージ一覧取得（トピック・ページネーション対応） |
| `GetMessage` | DLQ メッセージ取得（ID 指定） |
| `RetryMessage` | メッセージ個別リトライ |
| `DeleteMessage` | メッセージ削除（ID 指定） |
| `RetryAll` | トピック内メッセージ一括リトライ |

### `k1s0.system.workflow.v1` — WorkflowService

ワークフロー管理サービス。人間タスク・承認フロー込みのワークフローインスタンス管理を提供する。
`WorkflowStepType` enum（APPROVAL/AUTOMATED/NOTIFICATION）をサポート。

| RPC | 説明 |
|-----|------|
| `ListWorkflows` | ワークフロー定義一覧取得（enabled_only フィルタ対応） |
| `CreateWorkflow` | ワークフロー定義作成 |
| `GetWorkflow` | ワークフロー定義取得 |
| `UpdateWorkflow` | ワークフロー定義更新 |
| `DeleteWorkflow` | ワークフロー定義削除 |
| `StartInstance` | ワークフローインスタンス開始 |
| `GetInstance` | インスタンス詳細取得 |
| `ListInstances` | インスタンス一覧取得（ステータス・ワークフロー・開始者フィルタ） |
| `CancelInstance` | インスタンスキャンセル |
| `ListTasks` | タスク一覧取得（担当者・ステータス・期限超過フィルタ） |
| `ReassignTask` | タスク担当者変更 |
| `ApproveTask` | タスク承認 |
| `RejectTask` | タスク却下 |

### `k1s0.system.eventstore.v1` — EventStoreService

イベントストアサービス。イベントソーシング基盤。

| RPC | 説明 |
|-----|------|
| `AppendEvents` | イベント追記 |
| `ReadEvents` | イベント読み取り |
| `ReadEventBySequence` | シーケンス番号指定イベント読み取り |
| `ListStreams` | ストリーム一覧取得 |
| `CreateSnapshot` | スナップショット作成 |
| `GetLatestSnapshot` | 最新スナップショット取得 |
| `DeleteStream` | ストリーム削除 |

### `k1s0.system.event_monitor.v1` — EventMonitorService

イベント監視・フロー管理サービス。

| RPC | 説明 |
|-----|------|
| `ListEvents` | イベント一覧取得 |
| `TraceByCorrelation` | 相関 ID によるイベントトレース |
| `ListFlows` / `GetFlow` | フロー定義の CRUD |
| `GetFlowKpi` / `GetKpiSummary` | KPI メトリクス取得 |
| `GetSloStatus` / `GetSloBurnRate` | SLO ステータス・バーンレート取得 |
| `PreviewReplay` / `ExecuteReplay` | イベントリプレイ |

### `k1s0.system.scheduler.v1` — SchedulerService

スケジューラサービス。ジョブの定期実行管理。

| RPC | 説明 |
|-----|------|
| `CreateJob` | ジョブ作成 |
| `GetJob` | ジョブ取得 |
| `ListJobs` | ジョブ一覧取得 |
| `UpdateJob` | ジョブ更新 |
| `DeleteJob` | ジョブ削除 |

### `k1s0.system.notification.v1` — NotificationService

通知サービス。チャンネル・テンプレート管理と通知配信履歴を提供する。
`NotificationStatus` enum（PENDING/SENT/FAILED/RETRYING）をサポート。

| RPC | 説明 |
|-----|------|
| `SendNotification` | 通知送信（channel_id + template_id/body） |
| `GetNotification` | 通知詳細取得 |
| `RetryNotification` | 通知リトライ |
| `ListNotifications` | 通知一覧取得（channel・status フィルタ・ページネーション対応） |
| `ListChannels` | チャンネル一覧取得 |
| `CreateChannel` | チャンネル作成 |
| `GetChannel` | チャンネル取得 |
| `UpdateChannel` | チャンネル更新 |
| `DeleteChannel` | チャンネル削除 |
| `ListTemplates` | テンプレート一覧取得 |
| `CreateTemplate` | テンプレート作成 |
| `GetTemplate` | テンプレート取得 |
| `UpdateTemplate` | テンプレート更新 |
| `DeleteTemplate` | テンプレート削除 |

### `k1s0.system.session.v1` — SessionService

セッション管理サービス。デバイス情報・TTL・メタデータ管理を提供する。

| RPC | 説明 |
|-----|------|
| `CreateSession` | セッション作成（device_id・TTL・max_devices 対応） |
| `GetSession` | セッション取得 |
| `RefreshSession` | セッション更新（有効期限延長） |
| `RevokeSession` | セッション個別失効 |
| `RevokeAllSessions` | ユーザーの全セッション一括失効 |
| `ListUserSessions` | ユーザーのセッション一覧取得 |

### `k1s0.system.navigation.v1` — NavigationService

ナビゲーション（メニュー構造）サービス。

| RPC | 説明 |
|-----|------|
| `GetNavigation` | ナビゲーション構造取得 |

### `k1s0.system.policy.v1` — PolicyService

ポリシー管理サービス。OPA ベースのアクセス制御ポリシー。

| RPC | 説明 |
|-----|------|
| `CreatePolicy` | ポリシー作成 |
| `GetPolicy` | ポリシー取得 |
| `ListPolicies` | ポリシー一覧取得 |
| `UpdatePolicy` | ポリシー更新 |
| `DeletePolicy` | ポリシー削除 |
| `EvaluatePolicy` | ポリシー評価 |

### `k1s0.system.ratelimit.v1` — RateLimitService

レート制限サービス。`RateLimitAlgorithm` enum（SLIDING_WINDOW/TOKEN_BUCKET/FIXED_WINDOW/LEAKY_BUCKET）をサポート。

| RPC | 説明 |
|-----|------|
| `CheckRateLimit` | レート制限チェック（scope + identifier + window） |
| `CreateRule` | レートリミットルール作成 |
| `GetRule` | ルール取得（ID 指定） |
| `UpdateRule` | ルール更新 |
| `DeleteRule` | ルール削除 |
| `ListRules` | ルール一覧取得（scope・enabled_only フィルタ対応） |
| `GetUsage` | レートリミット使用状況取得 |
| `ResetLimit` | レートリミット状態リセット |

### `k1s0.system.vault.v1` — VaultService

シークレット管理サービス。

| RPC | 説明 |
|-----|------|
| `GetSecret` | シークレット取得 |
| `SetSecret` | シークレット設定 |
| `DeleteSecret` | シークレット削除 |
| `ListSecrets` | シークレット一覧取得 |

### `k1s0.system.search.v1` — SearchService

全文検索サービス。

| RPC | 説明 |
|-----|------|
| `Search` | 検索実行 |
| `IndexDocument` | ドキュメントインデックス |
| `DeleteDocument` | ドキュメント削除 |

### `k1s0.system.quota.v1` — QuotaService

クォータポリシー管理サービス。subject_type/subject_id ベースのポリシー CRUD・使用量管理を提供する。

| RPC | 説明 |
|-----|------|
| `CreateQuotaPolicy` | クォータポリシー作成 |
| `GetQuotaPolicy` | クォータポリシー取得 |
| `ListQuotaPolicies` | クォータポリシー一覧取得（subject フィルタ・enabled_only 対応） |
| `UpdateQuotaPolicy` | クォータポリシー更新 |
| `DeleteQuotaPolicy` | クォータポリシー削除 |
| `GetQuotaUsage` | 使用量取得（期間・超過状況含む） |
| `CheckQuota` | クォータ超過チェック |
| `IncrementQuotaUsage` | 使用量加算（冪等性: request_id 対応） |
| `ResetQuotaUsage` | 使用量リセット |

### `k1s0.system.file.v1` — FileService

ファイル管理サービス。

| RPC | 説明 |
|-----|------|
| `Upload` | ファイルアップロード |
| `Download` | ファイルダウンロード |
| `Delete` | ファイル削除 |
| `GetMetadata` | ファイルメタデータ取得 |

### `k1s0.system.apiregistry.v1` — ApiRegistryService

API スキーマレジストリサービス。スキーマ登録・バージョン管理・互換性チェック・差分取得を提供する。

| RPC | 説明 |
|-----|------|
| `ListSchemas` | スキーマ一覧取得（schema_type フィルタ・ページネーション対応） |
| `RegisterSchema` | スキーマ新規登録（初回バージョン同時作成） |
| `GetSchema` | スキーマ取得（最新コンテンツ含む） |
| `ListVersions` | スキーマバージョン一覧取得 |
| `RegisterVersion` | スキーマ新バージョン登録 |
| `GetSchemaVersion` | 指定バージョン取得 |
| `DeleteVersion` | バージョン削除 |
| `CheckCompatibility` | バージョン間互換性チェック（破壊的変更検出） |
| `GetDiff` | バージョン間差分取得（added/modified/removed） |

### `k1s0.system.mastermaintenance.v1` — MasterMaintenanceService

マスタメンテナンスサービス。テーブル定義・レコード CRUD・整合性チェック。

| RPC | 説明 |
|-----|------|
| `CreateTableDefinition` / `UpdateTableDefinition` / `DeleteTableDefinition` | テーブル定義 CRUD |
| `GetTableDefinition` / `ListTableDefinitions` | テーブル定義参照 |
| `GetRecord` / `CreateRecord` / `UpdateRecord` / `DeleteRecord` | レコード CRUD |
| `ListRecords` | レコード一覧取得 |
| `CheckConsistency` | 整合性チェック |
| `CreateRule` / `GetRule` / `UpdateRule` / `DeleteRule` | ルール CRUD |
| `ListRules` / `ExecuteRule` | ルール管理 |

### `k1s0.system.rule_engine.v1` — RuleEngineService

ルールエンジンサービス。ビジネスルールの定義・評価。

| RPC | 説明 |
|-----|------|
| `CreateRule` | ルール作成 |
| `GetRule` | ルール取得 |
| `ListRules` | ルール一覧取得 |
| `EvaluateRule` | ルール評価 |

### `k1s0.system.ai_agent.v1` — AiAgentService

AI エージェントサービス。

| RPC | 説明 |
|-----|------|
| `Execute` | エージェント実行 |
| `GetStatus` | 実行状態取得 |

### `k1s0.system.ai_gateway.v1` — AiGatewayService

AI ゲートウェイサービス。LLM プロバイダーへの統一アクセス。

| RPC | 説明 |
|-----|------|
| `Complete` | テキスト補完 |
| `Chat` | チャット |
| `Embed` | エンベディング生成 |

---

## Business ティア

### `k1s0.business.accounting.domainmaster.v1` — DomainMasterService

会計ドメインのマスタデータ管理サービス。

| RPC | 説明 |
|-----|------|
| テーブル定義・レコード CRUD | MasterMaintenanceService と同等の CRUD 操作 |

---

## Service ティア

### `k1s0.service.order.v1` — OrderService

注文管理サービス。

| RPC | 説明 |
|-----|------|
| `CreateOrder` | 注文作成 |
| `GetOrder` | 注文取得 |
| `ListOrders` | 注文一覧取得 |

### `k1s0.service.inventory.v1` — InventoryService

在庫管理サービス。

| RPC | 説明 |
|-----|------|
| `GetStock` | 在庫取得 |
| `ReserveStock` | 在庫予約 |
| `ReleaseStock` | 在庫解放 |

### `k1s0.service.payment.v1` — PaymentService

決済サービス。

| RPC | 説明 |
|-----|------|
| `CreatePayment` | 決済作成 |
| `GetPayment` | 決済取得 |
| `RefundPayment` | 返金処理 |

---

## イベントパッケージ

| パッケージ | 説明 |
|-----------|------|
| `k1s0.event.service.order.v1` | 注文ドメインイベント（OrderCreated, OrderUpdated 等） |
| `k1s0.event.service.inventory.v1` | 在庫ドメインイベント（StockReserved, StockReleased 等） |
| `k1s0.event.service.payment.v1` | 決済ドメインイベント（PaymentCompleted, PaymentRefunded 等） |

---

## パッケージ命名規約

```
k1s0.{tier}.{domain}.v{version}
```

- **tier**: `system` / `business` / `service`
- **domain**: サービスドメイン名（snake_case）
- **version**: API バージョン（v1, v2, ...）

イベントパッケージ:
```
k1s0.event.{tier}.{domain}.v{version}
```
