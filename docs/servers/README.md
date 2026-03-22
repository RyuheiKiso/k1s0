# サーバー設計書

k1s0 プロジェクトの全サーバー設計書一覧。system / business / service の3 Tier で構成される。
各サーバーは `server.md`（設計概要）・`implementation.md`（実装詳細）・`database.md`（DB設計）・`deploy.md`（デプロイ設計）の4文書セットで管理される。

## 共通ドキュメント

| ドキュメント | 内容 |
|------------|------|
| [_common/Go共通実装.md](./_common/Go共通実装.md) | Go サーバー共通実装パターン（K1s0App・ミドルウェア・ヘルスチェック） |
| [_common/Rust共通実装.md](./_common/Rust共通実装.md) | Rust サーバー共通実装パターン（axum・build.rs・エラー型） |
| [_common/client.md](./_common/client.md) | クライアント SDK 共通設計 |
| [_common/database.md](./_common/database.md) | DB 共通設計・マイグレーション方針 |
| [_common/deploy.md](./_common/deploy.md) | デプロイ共通設計・Helm チャート方針 |

---

## system Tier — 共通基盤サーバー

全サービスが依存する認証・設定・AI・イベント・ファイル等の共通基盤サーバー群。

| サーバー | 用途 | 設計書 |
|---------|------|--------|
| **auth** | OAuth2 / OIDC 認証認可・JWT 発行・RBAC | [server.md](./system/auth/server.md) / [impl](./system/auth/implementation.md) / [db](./system/auth/database.md) / [deploy](./system/auth/deploy.md) |
| **tenant** | マルチテナント管理・プロビジョニング・メンバー管理 | [server.md](./system/tenant/server.md) / [db](./system/tenant/database.md) / [deploy](./system/tenant/deploy.md) |
| **session** | セッション作成・取得・更新・失効管理 | [server.md](./system/session/server.md) / [db](./system/session/database.md) / [deploy](./system/session/deploy.md) |
| **config** | 全サービスへの設定値配信・変更通知・監査ログ | [server.md](./system/config/server.md) / [impl](./system/config/implementation.md) / [db](./system/config/database.md) / [deploy](./system/config/deploy.md) |
| **featureflag** | フィーチャーフラグ管理・動的機能制御 | [server.md](./system/featureflag/server.md) / [db](./system/featureflag/database.md) / [deploy](./system/featureflag/deploy.md) |
| **vault** | Vault シークレット管理・リース管理 | [server.md](./system/vault/server.md) / [db](./system/vault/database.md) / [deploy](./system/vault/deploy.md) |
| **file** | ローカルFS統一API・メタデータ管理・プリサインドURL（AWS/S3依存なし） | [server.md](./system/file/server.md) / [db](./system/file/database.md) / [deploy](./system/file/deploy.md) |
| **notification** | メール・Slack・Webhook・SMS・Push 通知配信 | [server.md](./system/notification/server.md) / [deploy](./system/notification/deploy.md) |
| **scheduler** | cron 式ジョブスケジューリング・分散実行管理 | [server.md](./system/scheduler/server.md) / [db](./system/scheduler/database.md) / [deploy](./system/scheduler/deploy.md) |
| **ratelimit** | Redis トークンバケットレート制限・Kong連携 | [server.md](./system/ratelimit/server.md) / [db](./system/ratelimit/database.md) / [deploy](./system/ratelimit/deploy.md) |
| **quota** | テナント・ユーザー・APIキー別クォータ管理 | [server.md](./system/quota/server.md) / [db](./system/quota/database.md) / [deploy](./system/quota/deploy.md) |
| **policy** | ポリシーベースアクセス制御（OPA連携） | [server.md](./system/policy/server.md) / [db](./system/policy/database.md) / [deploy](./system/policy/deploy.md) |
| **rule-engine** | ビジネスルール定義・実行エンジン | [server.md](./system/rule-engine/server.md) / [db](./system/rule-engine/database.md) / [deploy](./system/rule-engine/deploy.md) |
| **workflow** | ワークフロー定義・実行・状態管理 | [server.md](./system/workflow/server.md) / [db](./system/workflow/database.md) / [deploy](./system/workflow/deploy.md) |
| **saga** | YAML定義分散トランザクションオーケストレーション（Rust実装） | [server.md](./system/saga/server.md) / [db](./system/saga/database.md) / [deploy](./system/saga/deploy.md) |
| **event-store** | イベントソーシング向けイベント永続化・再生基盤 | [server.md](./system/event-store/server.md) / [db](./system/event-store/database.md) / [deploy](./system/event-store/deploy.md) |
| **event-monitor** | Kafka イベント監視・デッドレター管理 | [server.md](./system/event-monitor/server.md) / [db](./system/event-monitor/database.md) / [deploy](./system/event-monitor/deploy.md) |
| **dlq-manager** | DLQ（Dead Letter Queue）メッセージ集約管理（Rust実装） | [server.md](./system/dlq-manager/server.md) / [impl](./system/dlq-manager/implementation.md) / [db](./system/dlq-manager/database.md) / [deploy](./system/dlq-manager/deploy.md) |
| **search** | Elasticsearch / OpenSearch 全文検索 | [server.md](./system/search/server.md) / [db](./system/search/database.md) / [deploy](./system/search/deploy.md) |
| **navigation** | ロールベースルーティング・ガード設定配信 | [server.md](./system/navigation/server.md) / [deploy](./system/navigation/deploy.md) |
| **master-maintenance** | メタデータ駆動型マスタデータ CRUD 自動生成 | [server.md](./system/master-maintenance/server.md) / [db](./system/master-maintenance/database.md) / [deploy](./system/master-maintenance/deploy.md) |
| **api-registry** | OpenAPI / Protobuf スキーマ集中管理・破壊的変更検出（Rust実装） | [server.md](./system/api-registry/server.md) / [impl](./system/api-registry/implementation.md) / [db](./system/api-registry/database.md) / [deploy](./system/api-registry/deploy.md) |
| **app-registry** | アプリバージョン管理・ダウンロードURL生成（Rust実装） | [server.md](./system/app-registry/server.md) / [impl](./system/app-registry/implementation.md) / [db](./system/app-registry/database.md) / [deploy](./system/app-registry/deploy.md) |
| **service-catalog** | サービスメタデータ・依存関係・ヘルス集約管理（Rust実装） | [server.md](./system/service-catalog/server.md) / [impl](./system/service-catalog/implementation.md) / [db](./system/service-catalog/database.md) / [deploy](./system/service-catalog/deploy.md) |
| **graphql-gateway** | 複数 gRPC バックエンドを GraphQL に集約（Rust実装） | [server.md](./system/graphql-gateway/server.md) / [deploy](./system/graphql-gateway/deploy.md) |
| **bff-proxy** | OAuth2/OIDC セッション管理プロキシ・Cookie認証（Go実装） | [server.md](./system/bff-proxy/server.md) / [deploy](./system/bff-proxy/deploy.md) |
| **ai-gateway** | LLM プロバイダールーティング・ガードレール・使用量トラッキング | [server.md](./system/ai-gateway/server.md) / [impl](./system/ai-gateway/implementation.md) / [db](./system/ai-gateway/database.md) / [deploy](./system/ai-gateway/deploy.md) |
| **ai-agent** | ReAct ループ LLM エージェント実行・ツール呼び出し | [server.md](./system/ai-agent/server.md) / [impl](./system/ai-agent/implementation.md) / [db](./system/ai-agent/database.md) / [deploy](./system/ai-agent/deploy.md) |

---

## business Tier — 領域共通サーバー

特定業務領域（taskmanagement 等）内で共通利用する基盤サーバー群。

| サーバー | 用途 | 設計書 |
|---------|------|--------|
| **project-master** | taskmanagement 領域プロジェクトマスタデータハブ・テナント別カスタマイズ | [server.md](./business/project-master/server.md) / [impl](./business/project-master/implementation.md) / [db](./business/project-master/database.md) / [deploy](./business/project-master/deploy.md) |

---

## service Tier — 個別業務サーバー

実際にデプロイされる個別業務サービス群（サンプル実装）。

| サーバー | 用途 | 設計書 |
|---------|------|--------|
| **board** | ボードカラム管理・WIP制限・照会・更新・Kafka イベント配信 | [server.md](./service/board/server.md) / [db](./service/board/database.md) / [deploy](./service/board/deploy.md) |
| **task** | タスク作成・照会・ステータス管理・Kafka イベント配信 | [server.md](./service/task/server.md) / [db](./service/task/database.md) / [deploy](./service/task/deploy.md) |
| **activity** | アクティビティ記録・承認フロー・完了・失敗・Kafka イベント配信 | [server.md](./service/activity/server.md) / [db](./service/activity/database.md) / [deploy](./service/activity/deploy.md) |

---

## 関連ドキュメント

- [アーキテクチャ設計書](../architecture/README.md) — 全体設計方針
- [ライブラリ設計書](../libraries/README.md) — サーバーが依存する共通ライブラリ
- [インフラ設計書](../infrastructure/README.md) — デプロイ・Kubernetes・CI/CD
