# モジュール成熟度ステータス一覧

> **情報源**: モジュールの正式なステータス・成熟度は [`modules.yaml`](/modules.yaml) を参照してください。このドキュメントは概要説明を提供しますが、詳細は `modules.yaml` が正規の情報源（Authoritative Source）です。

<!-- 全モジュールの現在の成熟度レベルを一覧で管理する。定期的に更新すること -->

## 概要

本ドキュメントは、k1s0プロジェクト内の全モジュールの成熟度レベルを一覧で管理する。各レベルの定義は [maturity-levels.md](./maturity-levels.md) を参照。

**最終更新日**: 2026-03-16

---

## サマリー

<!-- プロジェクト全体の成熟度分布を把握するためのサマリー -->

| レベル | モジュール数 | 割合 |
|--------|-------------|------|
| `production` | 0 | 0% |
| `beta` | 約20 | 約10% |
| `experimental` | 3 | 約1% |
| `template-only` | 約179 | 約89% |

---

## Systemティア — サーバー（Rust）

<!-- Rustで実装されたシステム層サーバー。認証・設定など基盤サービス -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| auth | `beta` | テスト実装あり、リポジトリ層整備済み、gRPC/REST二重サーバー対応 |
| config | `beta` | 設定管理機能実装済み、リポジトリ層・エラー型整備済み |
| saga | `beta` | サーガオーケストレーション実装済み |
| dlq-manager | `beta` | DLQ管理機能実装済み |
| workflow | `beta` | ワークフローCRUDハンドラ実装済み |
| ai-agent | `experimental` | AI統合の実験的実装 |
| ai-gateway | `experimental` | AIゲートウェイの実験的実装 |
| api-registry | `template-only` | scaffold生成のみ |
| app-registry | `template-only` | scaffold生成のみ |
| event-monitor | `template-only` | scaffold生成のみ |
| event-store | `template-only` | scaffold生成のみ |
| featureflag | `template-only` | scaffold生成のみ |
| file | `template-only` | scaffold生成のみ |
| graphql-gateway | `template-only` | scaffold生成のみ |
| master-maintenance | `template-only` | scaffold生成のみ |
| navigation | `template-only` | scaffold生成のみ |
| notification | `beta` | 通知機能実装済み、REST/gRPC二重サーバー対応 |
| policy | `template-only` | scaffold生成のみ |
| quota | `template-only` | scaffold生成のみ |
| ratelimit | `template-only` | scaffold生成のみ |
| rule-engine | `template-only` | scaffold生成のみ |
| scheduler | `template-only` | scaffold生成のみ |
| search | `template-only` | scaffold生成のみ |
| service-catalog | `template-only` | scaffold生成のみ |
| session | `template-only` | scaffold生成のみ |
| tenant | `beta` | テナント管理機能実装済み、REST/gRPC二重サーバー対応 |
| vault | `template-only` | scaffold生成のみ |

---

## Systemティア — サーバー（Go）

<!-- Goで実装されたシステム層サーバー -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| bff-proxy | `beta` | BFFプロキシ機能実装済み |

---

## Businessティア

<!-- ビジネスロジック層。会計・ドメインマスタなど -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| accounting | `template-only` | server/client/library/database構造のみ |

---

## Serviceティア

<!-- サービス層。注文・在庫・決済などの業務サービス -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| order | `template-only` | scaffold生成のみ |
| inventory | `template-only` | scaffold生成のみ |
| payment | `template-only` | scaffold生成のみ |

---

## Systemティア — ライブラリ（Go）

<!-- Go言語のシステム共通ライブラリ群 -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| auth | `beta` | 認証・認可クライアントライブラリ |
| config | `beta` | 設定管理ライブラリ |
| correlation | `beta` | リクエスト相関ID管理 |
| health | `beta` | ヘルスチェック機能 |
| kafka | `beta` | Kafkaクライアントラッパー |
| telemetry | `beta` | テレメトリ・監視連携 |
| server-common | `beta` | サーバー共通ユーティリティ |
| messaging | `beta` | メッセージング抽象化 |
| validation | `beta` | バリデーション共通処理 |
| app-updater | `template-only` | scaffold生成のみ |
| audit-client | `template-only` | scaffold生成のみ |
| bb-ai-client | `template-only` | scaffold生成のみ |
| binding | `template-only` | scaffold生成のみ |
| building-blocks | `template-only` | scaffold生成のみ |
| bulkhead | `template-only` | scaffold生成のみ |
| cache | `template-only` | scaffold生成のみ |
| circuit-breaker | `template-only` | scaffold生成のみ |
| codegen | `experimental` | コード生成機能の実験的実装 |
| distributed-lock | `template-only` | scaffold生成のみ |
| dlq-client | `template-only` | scaffold生成のみ |
| encryption | `template-only` | scaffold生成のみ |
| event-bus | `template-only` | scaffold生成のみ |
| eventstore | `template-only` | scaffold生成のみ |
| featureflag | `template-only` | scaffold生成のみ |
| file-client | `template-only` | scaffold生成のみ |
| graphql-client | `template-only` | scaffold生成のみ |
| idempotency | `template-only` | scaffold生成のみ |
| migration | `template-only` | scaffold生成のみ |
| notification-client | `template-only` | scaffold生成のみ |
| outbox | `template-only` | scaffold生成のみ |
| pagination | `template-only` | scaffold生成のみ |
| pubsub | `template-only` | scaffold生成のみ |
| quota-client | `template-only` | scaffold生成のみ |
| ratelimit-client | `template-only` | scaffold生成のみ |
| resiliency | `template-only` | scaffold生成のみ |
| retry | `template-only` | scaffold生成のみ |
| saga | `template-only` | scaffold生成のみ |
| scheduler-client | `template-only` | scaffold生成のみ |
| schemaregistry | `template-only` | scaffold生成のみ |
| search-client | `template-only` | scaffold生成のみ |
| secret-store | `template-only` | scaffold生成のみ |
| serviceauth | `template-only` | scaffold生成のみ |
| session-client | `template-only` | scaffold生成のみ |
| statestore | `template-only` | scaffold生成のみ |
| telemetry-macros | `template-only` | scaffold生成のみ |
| tenant-client | `template-only` | scaffold生成のみ |
| test-helper | `template-only` | scaffold生成のみ |
| tracing | `template-only` | scaffold生成のみ |
| vault-client | `template-only` | scaffold生成のみ |
| webhook-client | `template-only` | scaffold生成のみ |
| websocket | `template-only` | scaffold生成のみ |

---

## Systemティア — ライブラリ（Rust）

<!-- Rust言語のシステム共通ライブラリ群 -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| auth | `beta` | 認証・認可クライアントライブラリ |
| config | `beta` | 設定管理ライブラリ |
| correlation | `beta` | リクエスト相関ID管理 |
| health | `beta` | ヘルスチェック機能 |
| kafka | `beta` | Kafkaクライアントラッパー |
| telemetry | `beta` | テレメトリ・監視連携 |
| server-common | `beta` | サーバー共通ユーティリティ |
| messaging | `beta` | メッセージング抽象化 |
| validation | `beta` | バリデーション共通処理 |
| app-updater | `template-only` | scaffold生成のみ |
| audit-client | `template-only` | scaffold生成のみ |
| bb-ai-client | `template-only` | scaffold生成のみ |
| bb-binding | `template-only` | scaffold生成のみ |
| bb-core | `template-only` | scaffold生成のみ |
| bb-pubsub | `template-only` | scaffold生成のみ |
| bb-secretstore | `template-only` | scaffold生成のみ |
| bb-statestore | `template-only` | scaffold生成のみ |
| building-blocks | `template-only` | scaffold生成のみ |
| bulkhead | `template-only` | scaffold生成のみ |
| cache | `template-only` | scaffold生成のみ |
| circuit-breaker | `template-only` | scaffold生成のみ |
| codegen | `template-only` | scaffold生成のみ |
| distributed-lock | `template-only` | scaffold生成のみ |
| dlq-client | `template-only` | scaffold生成のみ |
| encryption | `template-only` | scaffold生成のみ |
| event-bus | `template-only` | scaffold生成のみ |
| eventstore | `template-only` | scaffold生成のみ |
| featureflag | `template-only` | scaffold生成のみ |
| file-client | `template-only` | scaffold生成のみ |
| graphql-client | `template-only` | scaffold生成のみ |
| idempotency | `template-only` | scaffold生成のみ |
| migration | `template-only` | scaffold生成のみ |
| notification-client | `template-only` | scaffold生成のみ |
| outbox | `template-only` | scaffold生成のみ |
| pagination | `template-only` | scaffold生成のみ |
| quota-client | `template-only` | scaffold生成のみ |
| ratelimit-client | `template-only` | scaffold生成のみ |
| resiliency | `template-only` | scaffold生成のみ |
| retry | `template-only` | scaffold生成のみ |
| saga | `template-only` | scaffold生成のみ |
| scheduler-client | `template-only` | scaffold生成のみ |
| schemaregistry | `template-only` | scaffold生成のみ |
| search-client | `template-only` | scaffold生成のみ |
| serviceauth | `template-only` | scaffold生成のみ |
| session-client | `template-only` | scaffold生成のみ |
| telemetry-macros | `template-only` | scaffold生成のみ |
| tenant-client | `template-only` | scaffold生成のみ |
| test-helper | `template-only` | scaffold生成のみ |
| tracing | `template-only` | scaffold生成のみ |
| vault-client | `template-only` | scaffold生成のみ |
| webhook-client | `template-only` | scaffold生成のみ |
| websocket | `template-only` | scaffold生成のみ |

---

## Systemティア — ライブラリ（TypeScript）

<!-- TypeScript言語のシステム共通ライブラリ群 -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| auth | `beta` | 認証・認可クライアントライブラリ |
| config | `beta` | 設定管理ライブラリ |
| correlation | `beta` | リクエスト相関ID管理 |
| health | `beta` | ヘルスチェック機能 |
| kafka | `beta` | Kafkaクライアントラッパー |
| telemetry | `beta` | テレメトリ・監視連携 |
| server-common | `beta` | サーバー共通ユーティリティ |
| messaging | `beta` | メッセージング抽象化 |
| validation | `beta` | バリデーション共通処理 |
| app-updater | `template-only` | scaffold生成のみ |
| audit-client | `template-only` | scaffold生成のみ |
| bb-ai-client | `template-only` | scaffold生成のみ |
| building-blocks | `template-only` | scaffold生成のみ |
| bulkhead | `template-only` | scaffold生成のみ |
| cache | `template-only` | scaffold生成のみ |
| circuit-breaker | `template-only` | scaffold生成のみ |
| codegen | `template-only` | scaffold生成のみ |
| distributed-lock | `template-only` | scaffold生成のみ |
| dlq-client | `template-only` | scaffold生成のみ |
| encryption | `template-only` | scaffold生成のみ |
| event-bus | `template-only` | scaffold生成のみ |
| eventstore | `template-only` | scaffold生成のみ |
| featureflag | `template-only` | scaffold生成のみ |
| file-client | `template-only` | scaffold生成のみ |
| graphql-client | `template-only` | scaffold生成のみ |
| idempotency | `template-only` | scaffold生成のみ |
| migration | `template-only` | scaffold生成のみ |
| notification-client | `template-only` | scaffold生成のみ |
| outbox | `template-only` | scaffold生成のみ |
| pagination | `template-only` | scaffold生成のみ |
| quota-client | `template-only` | scaffold生成のみ |
| ratelimit-client | `template-only` | scaffold生成のみ |
| resiliency | `template-only` | scaffold生成のみ |
| retry | `template-only` | scaffold生成のみ |
| saga | `template-only` | scaffold生成のみ |
| scheduler-client | `template-only` | scaffold生成のみ |
| schemaregistry | `template-only` | scaffold生成のみ |
| search-client | `template-only` | scaffold生成のみ |
| serviceauth | `template-only` | scaffold生成のみ |
| session-client | `template-only` | scaffold生成のみ |
| tenant-client | `template-only` | scaffold生成のみ |
| test-helper | `template-only` | scaffold生成のみ |
| tracing | `template-only` | scaffold生成のみ |
| vault-client | `template-only` | scaffold生成のみ |
| webhook-client | `template-only` | scaffold生成のみ |
| websocket | `template-only` | scaffold生成のみ |

---

## Systemティア — ライブラリ（Dart）

<!-- Dart言語のシステム共通ライブラリ群（Flutter向け） -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| auth | `beta` | 認証・認可クライアントライブラリ |
| config | `beta` | 設定管理ライブラリ |
| correlation | `beta` | リクエスト相関ID管理 |
| health | `beta` | ヘルスチェック機能 |
| kafka | `beta` | Kafkaクライアントラッパー |
| telemetry | `beta` | テレメトリ・監視連携 |
| server_common | `beta` | サーバー共通ユーティリティ |
| messaging | `beta` | メッセージング抽象化 |
| validation | `beta` | バリデーション共通処理 |
| app_updater | `template-only` | scaffold生成のみ |
| audit_client | `template-only` | scaffold生成のみ |
| bb-ai-client | `template-only` | scaffold生成のみ |
| building_blocks | `template-only` | scaffold生成のみ |
| bulkhead | `template-only` | scaffold生成のみ |
| cache | `template-only` | scaffold生成のみ |
| circuit_breaker | `template-only` | scaffold生成のみ |
| codegen | `template-only` | scaffold生成のみ |
| distributed_lock | `template-only` | scaffold生成のみ |
| dlq_client | `template-only` | scaffold生成のみ |
| encryption | `template-only` | scaffold生成のみ |
| event_bus | `template-only` | scaffold生成のみ |
| eventstore | `template-only` | scaffold生成のみ |
| featureflag | `template-only` | scaffold生成のみ |
| file_client | `template-only` | scaffold生成のみ |
| graphql_client | `template-only` | scaffold生成のみ |
| idempotency | `template-only` | scaffold生成のみ |
| migration | `template-only` | scaffold生成のみ |
| notification_client | `template-only` | scaffold生成のみ |
| outbox | `template-only` | scaffold生成のみ |
| pagination | `template-only` | scaffold生成のみ |
| quota_client | `template-only` | scaffold生成のみ |
| ratelimit_client | `template-only` | scaffold生成のみ |
| resiliency | `template-only` | scaffold生成のみ |
| retry | `template-only` | scaffold生成のみ |
| saga | `template-only` | scaffold生成のみ |
| scheduler_client | `template-only` | scaffold生成のみ |
| schemaregistry | `template-only` | scaffold生成のみ |
| search_client | `template-only` | scaffold生成のみ |
| serviceauth | `template-only` | scaffold生成のみ |
| session_client | `template-only` | scaffold生成のみ |
| tenant_client | `template-only` | scaffold生成のみ |
| test_helper | `template-only` | scaffold生成のみ |
| tracing | `template-only` | scaffold生成のみ |
| vault_client | `template-only` | scaffold生成のみ |
| webhook_client | `template-only` | scaffold生成のみ |
| websocket | `template-only` | scaffold生成のみ |

---

## CLI

<!-- Rust製の対話式CLIツール群 -->

| モジュール | 成熟度 | 備考 |
|-----------|--------|------|
| k1s0-cli | `beta` | 対話式CLIメインエントリポイント |
| k1s0-core | `beta` | CLIコアロジック（sparse-checkout連携） |
| k1s0-gui | `beta` | GUI関連機能 |

---

## 更新履歴

<!-- 成熟度レベルの変更履歴を記録する -->

| 日付 | モジュール | 変更前 | 変更後 | 理由 |
|------|-----------|--------|--------|------|
| 2026-03-21 | notification (Rust server) | `template-only` | `beta` | 通知機能実装確認・成熟度更新 |
| 2026-03-21 | tenant (Rust server) | `template-only` | `beta` | テナント管理機能実装確認・成熟度更新 |
| 2026-03-16 | Go codegen | `template-only` | `experimental` | コード生成機能の実験的実装開始 |
| 2026-03-15 | — | — | — | 初版作成 |
