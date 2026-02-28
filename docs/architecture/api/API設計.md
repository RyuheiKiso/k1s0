# API 設計

k1s0 における REST API / gRPC / GraphQL の設計方針と、バージョニング・レート制限・コード自動生成を定義する。
Tier アーキテクチャの詳細は [tier-architecture.md](../../architecture/overview/tier-architecture.md) を参照。

## 基本方針

- サービス間通信は **gRPC** を標準とする
- 外部クライアント向けには **REST API** を Kong API Gateway 経由で公開する
- BFF が必要な場合は **GraphQL** をオプション採用する
- 全 API に統一的なエラーレスポンス・バージョニング・レート制限を適用する

---

## Tier 別 API 種別パターン

各 Tier のサービスが提供する API 種別のパターンを以下に定義する。具体的なエンドポイント一覧は各サービスの設計フェーズで定義する。

### system Tier

基盤サービスとして、認証・設定・共通マスタデータを提供する。

| API 種別 | プロトコル | 用途 | 例 |
| --- | --- | --- | --- |
| 認証 API | REST | OAuth 2.0 フローに基づくトークン発行・検証 | `/api/v1/auth/token`, `/api/v1/auth/refresh` |
| 認証 API | gRPC | サービス間のトークン検証（高速な内部呼び出し） | `AuthService.ValidateToken` |
| config API | REST | 設定値の取得・更新 | `/api/v1/config/:namespace/:key` |
| config API | gRPC | サービス間の設定値参照 | `ConfigService.GetConfig` |
| 共通マスタ API | REST | マスタデータの CRUD（外部向け） | `/api/v1/master/{resource}` |
| 共通マスタ API | gRPC | マスタデータ参照（サービス間） | `MasterService.GetMaster` |

### business Tier

ドメイン固有のビジネスロジックを提供する。

| API 種別 | プロトコル | 用途 | 例 |
| --- | --- | --- | --- |
| ドメイン CRUD API | REST | ドメインエンティティの作成・参照・更新・削除 | `/api/v1/ledger/entries`, `/api/v1/accounts` |
| ドメインイベント配信 | gRPC Stream | ドメインイベントのリアルタイム配信（サーバーストリーミング） | `LedgerService.StreamLedgerEvents` |
| ドメイン操作 API | gRPC | サービス間のドメイン操作呼び出し | `AccountingService.CloseLedger` |

### service Tier

フロントエンド向けの BFF やサードパーティ向けの外部連携を提供する。

| API 種別 | プロトコル | 用途 | 例 |
| --- | --- | --- | --- |
| BFF API | REST | フロントエンドからの標準的な API 呼び出し | `/api/v1/orders`, `/api/v1/dashboard` |
| BFF API | GraphQL | 複数サービスのデータ集約（導入基準を満たす場合） | `query { dashboard { ... } }` |
| 外部連携 API | REST | サードパーティシステムとのデータ連携 | `/api/v1/integrations/{provider}` |

> **注記**: 上記はパターンの定義であり、具体的なエンドポイント一覧・リクエスト/レスポンス仕様は各サービスの設計フェーズで定義する。

---

## 詳細設計ドキュメント

各設計の詳細は以下の分割ドキュメントを参照。

| ドキュメント | 内容 |
| --- | --- |
| [REST-API設計.md](./REST-API設計.md) | D-007 エラーレスポンス、D-008 バージョニング、D-012 レート制限、D-123 OpenAPI コード自動生成 |
| [gRPC設計.md](./gRPC設計.md) | D-009 サービス定義パターン、D-010 バージョニング |
| [GraphQL設計.md](./GraphQL設計.md) | D-011 GraphQL 設計、D-124 実装技術選定 |

---

## 関連ドキュメント

- [tier-architecture.md](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャの詳細
- [config.md](../../cli/config/config設計.md) -- config.yaml スキーマと環境別管理
- [kubernetes設計.md](../../infrastructure/kubernetes/kubernetes設計.md) -- Namespace・NetworkPolicy 設計
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart と values 設計
- [認証認可設計.md](../auth/認証認可設計.md) -- 認証・認可・Kong 認証フロー
- [インフラ設計.md](../../infrastructure/overview/インフラ設計.md) -- オンプレミスインフラ全体構成
- [APIゲートウェイ設計.md](APIゲートウェイ設計.md) -- Kong 構成管理
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka・イベント駆動設計
- [CI-CD設計.md](../../infrastructure/cicd/CI-CD設計.md) -- CI/CD パイプライン設計
- [ディレクトリ構成図.md](../../architecture/overview/ディレクトリ構成図.md) -- プロジェクトのディレクトリ構成
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- Docker イメージ戦略
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
- [proto設計.md](./proto設計.md) -- gRPC サービス定義・Protobuf スキーマ・buf 設定
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート（REST/gRPC/GraphQL ハンドラー）
