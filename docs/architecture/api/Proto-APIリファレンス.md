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

認証・認可の基盤サービス。

| RPC | 説明 |
|-----|------|
| `Authenticate` | トークン検証・ユーザー認証 |
| `Authorize` | リソースアクセスの認可判定 |
| `ListUsers` | ユーザー一覧取得 |
| `GetUser` | ユーザー詳細取得 |

関連サービス: `AuditService` — 監査ログの記録・取得

### `k1s0.system.config.v1` — ConfigService

設定値管理サービス。namespace/key ベースの設定 CRUD。

| RPC | 説明 |
|-----|------|
| `GetConfig` | 設定値取得（namespace + key） |
| `ListConfigs` | namespace 内の設定値一覧取得（ページネーション対応） |
| `UpdateConfig` | 設定値更新（楽観的排他制御） |
| `WatchConfig` | 設定変更の Server-Side Streaming |

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

サーガオーケストレーションサービス。

| RPC | 説明 |
|-----|------|
| `StartSaga` | サーガ開始 |
| `GetSaga` | サーガ状態取得 |
| `CompensateSaga` | 補償トランザクション実行 |

### `k1s0.system.dlq.v1` — DlqService

デッドレターキュー管理サービス。

| RPC | 説明 |
|-----|------|
| `ListMessages` | DLQ メッセージ一覧取得 |
| `RetryMessage` | メッセージ再処理 |
| `PurgeMessages` | メッセージ削除 |

### `k1s0.system.workflow.v1` — WorkflowService

ワークフロー管理サービス。

| RPC | 説明 |
|-----|------|
| `CreateWorkflow` | ワークフロー定義作成 |
| `GetWorkflow` | ワークフロー取得 |
| `ListWorkflows` | ワークフロー一覧取得 |
| `UpdateWorkflow` | ワークフロー更新 |
| `DeleteWorkflow` | ワークフロー削除 |
| `ExecuteWorkflow` | ワークフロー実行 |

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

通知サービス。メール・Slack・Push 通知の統一管理。

| RPC | 説明 |
|-----|------|
| `SendNotification` | 通知送信 |
| `ListNotifications` | 通知一覧取得 |
| `GetNotification` | 通知詳細取得 |

### `k1s0.system.session.v1` — SessionService

セッション管理サービス。

| RPC | 説明 |
|-----|------|
| `CreateSession` | セッション作成 |
| `GetSession` | セッション取得 |
| `DeleteSession` | セッション削除 |
| `ListSessions` | セッション一覧取得 |

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

レート制限サービス。

| RPC | 説明 |
|-----|------|
| `CheckRateLimit` | レート制限チェック |
| `GetRateLimitConfig` | レート制限設定取得 |
| `SetRateLimitConfig` | レート制限設定更新 |

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

クォータ管理サービス。

| RPC | 説明 |
|-----|------|
| `GetQuota` | クォータ取得 |
| `SetQuota` | クォータ設定 |
| `CheckQuota` | クォータチェック |
| `ConsumeQuota` | クォータ消費 |

### `k1s0.system.file.v1` — FileService

ファイル管理サービス。

| RPC | 説明 |
|-----|------|
| `Upload` | ファイルアップロード |
| `Download` | ファイルダウンロード |
| `Delete` | ファイル削除 |
| `GetMetadata` | ファイルメタデータ取得 |

### `k1s0.system.apiregistry.v1` — ApiRegistryService

API レジストリサービス。サービスカタログと API バージョン管理。

| RPC | 説明 |
|-----|------|
| `RegisterApi` | API 登録 |
| `GetApi` | API 取得 |
| `ListApis` | API 一覧取得 |
| `UpdateApi` | API 更新 |
| `DeleteApi` | API 削除 |

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
