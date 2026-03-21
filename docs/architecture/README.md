# アーキテクチャ設計書

k1s0 システム全体の設計方針・技術選定・規約をまとめたドキュメント群。

## overview — システム概要

| ドキュメント | 内容 |
|------------|------|
| [コンセプト.md](./overview/コンセプト.md) | プロジェクトのミッション・技術スタック・設計哲学 |
| [tier-architecture.md](./overview/tier-architecture.md) | system / business / service の3層 Tier 設計・依存方向 |
| [ディレクトリ構成図.md](./overview/ディレクトリ構成図.md) | モノリポのディレクトリ構成と配置ルール |
| [module-status.md](./overview/module-status.md) | 各モジュールの実装状況 |
| [modules-maturity-roadmap.md](./overview/modules-maturity-roadmap.md) | モジュール成熟度ロードマップ |
| [maturity-levels.md](./overview/maturity-levels.md) | 成熟度レベル定義 |
| [db-multi-role.md](./overview/db-multi-role.md) | データベースマルチロール設計 |
| [gRPCセキュリティ方針.md](./overview/gRPCセキュリティ方針.md) | gRPC 通信のセキュリティ方針 |

## api — API 設計

| ドキュメント | 内容 |
|------------|------|
| [API設計.md](./api/API設計.md) | API 設計の全体方針・原則 |
| [REST-API設計.md](./api/REST-API設計.md) | RESTful API 設計規約・命名ルール |
| [gRPC設計.md](./api/gRPC設計.md) | gRPC サービス設計・Proto 規約 |
| [GraphQL設計.md](./api/GraphQL設計.md) | GraphQL スキーマ設計・クエリ規約 |
| [proto設計.md](./api/proto設計.md) | Protobuf ファイル設計規約 |
| [Proto-APIリファレンス.md](./api/Proto-APIリファレンス.md) | Proto API リファレンス |
| [APIゲートウェイ設計.md](./api/APIゲートウェイ設計.md) | Kong API ゲートウェイ設計 |
| [ページネーション設計.md](./api/ページネーション設計.md) | カーソル / オフセットページネーション設計 |
| [schema-registry.md](./api/schema-registry.md) | スキーマレジストリ設計 |

## auth — 認証・認可設計

| ドキュメント | 内容 |
|------------|------|
| [認証設計.md](./auth/認証設計.md) | 認証フロー全体設計 |
| [認証認可設計.md](./auth/認証認可設計.md) | OAuth2.0・JWT・RBAC の統合設計 |
| [JWT設計.md](./auth/JWT設計.md) | JWT トークン構造・検証方針 |
| [RBAC設計.md](./auth/RBAC設計.md) | ロールベースアクセス制御設計 |
| [サービス間認証設計.md](./auth/サービス間認証設計.md) | サービス間 Client Credentials 認証 |
| [auth-layer-design.md](./auth/auth-layer-design.md) | 認証レイヤー設計 |
| [device-authorization-grant.md](./auth/device-authorization-grant.md) | デバイス認証フロー設計 |
| [token-storage-security.md](./auth/token-storage-security.md) | トークンストレージセキュリティ |

## conventions — 規約・方針

| ドキュメント | 内容 |
|------------|------|
| [コーディング規約.md](./conventions/コーディング規約.md) | Go / Rust / TypeScript / Dart の言語別コーディングルール |
| [エラーハンドリング方針.md](./conventions/エラーハンドリング方針.md) | エラーコード体系・構造化エラーレスポンス |
| [設定管理方針.md](./conventions/設定管理方針.md) | config.yaml スキーマ・環境別管理 |
| [version-policy.md](./conventions/version-policy.md) | バージョニングポリシー・互換性ルール |

## testing — テスト戦略

| ドキュメント | 内容 |
|------------|------|
| [test-strategy.md](./testing/test-strategy.md) | テスト戦略全体・TDD 方針・カバレッジ目標 |
| [e2e-strategy.md](./testing/e2e-strategy.md) | E2E テスト戦略 |
| [performance-strategy.md](./testing/performance-strategy.md) | パフォーマンステスト戦略 |

## messaging — メッセージング設計

| ドキュメント | 内容 |
|------------|------|
| [メッセージング設計.md](./messaging/メッセージング設計.md) | Kafka イベント駆動アーキテクチャ設計・トピック命名規約 |

## observability — 可観測性設計

| ドキュメント | 内容 |
|------------|------|
| [可観測性設計.md](./observability/可観測性設計.md) | OpenTelemetry・ログ・メトリクス・トレースの全体設計 |
| [ログ設計.md](./observability/ログ設計.md) | 構造化ログ設計・ログレベル方針 |
| [トレーシング設計.md](./observability/トレーシング設計.md) | 分散トレーシング設計 |
| [監視アラート設計.md](./observability/監視アラート設計.md) | Prometheus アラートルール設計 |
| [SLO設計.md](./observability/SLO設計.md) | SLO / SLA 定義・エラーバジェット |

## deployment — デプロイ設計

| ドキュメント | 内容 |
|------------|------|
| [プログレッシブデリバリー設計.md](./deployment/プログレッシブデリバリー設計.md) | カナリアリリース・フィーチャーフラグ連携デプロイ |

## その他

| ドキュメント | 内容 |
|------------|------|
| [ai/AIゲートウェイ設計.md](./ai/AIゲートウェイ設計.md) | AI Gateway 統合設計 |
| [chaos-engineering/カオスエンジニアリング設計.md](./chaos-engineering/カオスエンジニアリング設計.md) | カオスエンジニアリング実施方針 |
| [developer-portal/デベロッパーポータル設計.md](./developer-portal/デベロッパーポータル設計.md) | デベロッパーポータル設計 |
| [multi-tenancy.md](./multi-tenancy.md) | マルチテナント設計 |
| [soft-delete.md](./soft-delete.md) | ソフトデリート設計方針 |

---

## 関連ドキュメント

- [オンボーディングガイド](../onboarding/README.md) — Tier 別の入門ガイド
- [ライブラリ設計書](../libraries/README.md) — 共通ライブラリ一覧
- [サーバー設計書](../servers/README.md) — 全サーバー設計書
