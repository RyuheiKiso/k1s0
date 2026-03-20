# system-service-catalog-server 設計

system tier のサービスカタログサーバー。サービスメタデータ・依存関係・ヘルスステータス・ドキュメントの集約管理を提供する。Backstage inspired service catalog for k1s0。Rust 実装。

## 概要

| 機能 | 説明 |
| --- | --- |
| サービス一覧・詳細取得 | 登録済みサービスのメタデータ、オーナー、ライフサイクル情報の取得 |
| 依存関係管理 | サービス間の依存グラフの登録・取得・サイクル検出 |
| ヘルスステータス集約 | 各サービスのヘルス情報をポーリング/push で集約 |
| ドキュメントリンク管理 | API 仕様、設計書へのリンクを一元管理 |
| サービス検索 | タグ、カテゴリ、チーム別の横断検索 |
| サービススコアカード | 品質メトリクス、SLO 達成率の可視化 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| JWT 認証 | k1s0-auth ライブラリ |
| キャッシュ | k1s0-cache（Redis） |
| イベント発行 | k1s0-kafka |

### 配置パス

配置: `regions/system/server/rust/service-catalog/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_SCAT_` とする。REST API + gRPC を提供する。

#### 公開エンドポイント（認証不要）

| Method | Path | Description |
| --- | --- | --- |
| GET | `/healthz` | ヘルスチェック |
| GET | `/readyz` | レディネスチェック（DB 接続確認） |
| GET | `/metrics` | Prometheus メトリクス |

#### 保護エンドポイント（Bearer トークン + RBAC 必要）

| Method | Path | Description | RBAC リソース | RBAC 権限 |
| --- | --- | --- | --- | --- |
| GET | `/api/v1/services` | サービス一覧取得 | `services` | `read` |
| GET | `/api/v1/services/{id}` | サービス詳細取得 | `services` | `read` |
| POST | `/api/v1/services` | サービス登録 | `services` | `write` |
| PUT | `/api/v1/services/{id}` | サービス更新 | `services` | `write` |
| DELETE | `/api/v1/services/{id}` | サービス削除 | `services` | `write` |
| GET | `/api/v1/services/{id}/dependencies` | 依存関係取得 | `services` | `read` |
| PUT | `/api/v1/services/{id}/dependencies` | 依存関係更新 | `services` | `write` |
| GET | `/api/v1/services/{id}/health` | ヘルスステータス取得 | `services` | `read` |
| POST | `/api/v1/services/{id}/health` | ヘルス報告 | `services` | `write` |
| GET | `/api/v1/services/{id}/docs` | ドキュメント一覧取得 | `services` | `read` |
| PUT | `/api/v1/services/{id}/docs` | ドキュメント更新 | `services` | `write` |
| GET | `/api/v1/services/{id}/scorecard` | スコアカード取得 | `services` | `read` |
| GET | `/api/v1/services/search` | サービス検索 | `services` | `read` |
| GET | `/api/v1/teams` | チーム一覧取得 | `teams` | `read` |
| GET | `/api/v1/teams/{team_id}` | チーム詳細取得 | `teams` | `read` |
| POST | `/api/v1/teams` | チーム作成 | `teams` | `write` |
| PUT | `/api/v1/teams/{team_id}` | チーム更新 | `teams` | `write` |
| DELETE | `/api/v1/teams/{team_id}` | チーム削除 | `teams` | `write` |
| GET | `/api/v1/teams/{team_id}/services` | チーム別サービス取得 | `teams` | `read` |

### 共通エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_SCAT_INTERNAL_ERROR` | 500 | 内部エラー |
| `SYS_SCAT_SERVICE_NOT_FOUND` | 404 | 指定されたサービスが見つからない |
| `SYS_SCAT_TEAM_NOT_FOUND` | 404 | 指定されたチームが見つからない |
| `SYS_SCAT_DUPLICATE_SERVICE` | 409 | サービス名が既に登録されている |
| `SYS_SCAT_INVALID_LIFECYCLE` | 400 | 無効なライフサイクル値 |
| `SYS_SCAT_DEPENDENCY_CYCLE` | 400 | 依存関係にサイクルが検出された |
| `SYS_SCAT_MISSING_TOKEN` | 401 | Authorization ヘッダーが存在しない |
| `SYS_SCAT_PERMISSION_DENIED` | 403 | 要求された resource/action の権限がない |

#### GET /api/v1/services

サービス一覧をタグ・ライフサイクル・ティア・検索条件で取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `lifecycle` | string | No | - | ライフサイクルでフィルター（development/staging/production/deprecated） |
| `tier` | string | No | - | ティアでフィルター（system/business/service） |
| `tag` | string | No | - | タグでフィルター |
| `owner_team_id` | string | No | - | オーナーチーム ID でフィルター |

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `services[].id` | string | サービス ID |
| `services[].name` | string | サービス名 |
| `services[].display_name` | string | 表示名 |
| `services[].description` | string | 説明 |
| `services[].owner_team_id` | string | オーナーチーム ID |
| `services[].lifecycle` | string | ライフサイクル（development/staging/production/deprecated） |
| `services[].tier` | string | ティア（system/business/service） |
| `services[].repository_url` | string | リポジトリ URL |
| `services[].tags` | string[] | タグ一覧 |
| `services[].metadata` | object | メタデータ（任意の key-value） |
| `services[].created_at` | string | 作成日時（RFC 3339） |
| `services[].updated_at` | string | 更新日時（RFC 3339） |

#### GET /api/v1/services/{id}

指定された ID のサービス詳細を取得する。レスポンスフィールドは GET /api/v1/services の `services[]` と同一。

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`

#### POST /api/v1/services

新しいサービスを登録する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `name` | string | Yes | サービス名（一意） |
| `display_name` | string | No | 表示名 |
| `description` | string | No | 説明 |
| `owner_team_id` | string | Yes | オーナーチーム ID |
| `lifecycle` | string | Yes | ライフサイクル（development/staging/production/deprecated） |
| `tier` | string | Yes | ティア（system/business/service） |
| `repository_url` | string | No | リポジトリ URL |
| `tags` | string[] | No | タグ一覧 |
| `metadata` | object | No | メタデータ（任意の key-value） |

**レスポンスフィールド（201 Created）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string | サービス ID |
| `created_at` | string | 作成日時（RFC 3339） |

**エラーレスポンス（409）**: `SYS_SCAT_DUPLICATE_SERVICE`
**エラーレスポンス（400）**: `SYS_SCAT_INVALID_LIFECYCLE`

#### PUT /api/v1/services/{id}

指定されたサービスの情報を更新する。リクエストフィールドは POST /api/v1/services と同一。

**レスポンス**: 200 OK（更新後のサービス詳細）

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`
**エラーレスポンス（400）**: `SYS_SCAT_INVALID_LIFECYCLE`

#### DELETE /api/v1/services/{id}

指定されたサービスを削除する。関連する依存関係・ヘルスステータス・ドキュメント・スコアカードも連鎖削除される。

**レスポンス**: 204 No Content

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`

#### GET /api/v1/services/{id}/dependencies

指定されたサービスの依存関係一覧を取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `dependencies[].id` | string | 依存関係 ID |
| `dependencies[].source_service_id` | string | 依存元サービス ID |
| `dependencies[].target_service_id` | string | 依存先サービス ID |
| `dependencies[].dependency_type` | string | 依存種別（runtime/build/optional） |
| `dependencies[].description` | string | 説明 |
| `dependencies[].created_at` | string | 作成日時（RFC 3339） |

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`

#### PUT /api/v1/services/{id}/dependencies

指定されたサービスの依存関係を更新する。サイクル検出を行い、循環依存がある場合はエラーを返す。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `dependencies[].target_service_id` | string | Yes | 依存先サービス ID |
| `dependencies[].dependency_type` | string | Yes | 依存種別（runtime/build/optional） |
| `dependencies[].description` | string | No | 説明 |

**レスポンス**: 200 OK（更新後の依存関係一覧）

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`
**エラーレスポンス（400）**: `SYS_SCAT_DEPENDENCY_CYCLE`

#### GET /api/v1/services/{id}/health

指定されたサービスのヘルスステータスを取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `service_id` | string | サービス ID |
| `status` | string | ステータス（healthy/degraded/unhealthy/unknown） |
| `last_check_at` | string | 最終チェック日時（RFC 3339） |
| `details` | object | 詳細情報（任意の key-value） |

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`

#### POST /api/v1/services/{id}/health

サービスのヘルスステータスを報告（push）する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `status` | string | Yes | ステータス（healthy/degraded/unhealthy） |
| `details` | object | No | 詳細情報 |

**レスポンス**: 200 OK

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`

#### GET /api/v1/services/{id}/docs

指定されたサービスのドキュメントリンク一覧を取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `docs[].id` | string | ドキュメント ID |
| `docs[].service_id` | string | サービス ID |
| `docs[].title` | string | タイトル |
| `docs[].url` | string | ドキュメント URL |
| `docs[].doc_type` | string | ドキュメント種別（api_spec/design/runbook/other） |
| `docs[].created_at` | string | 作成日時（RFC 3339） |
| `docs[].updated_at` | string | 更新日時（RFC 3339） |

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`

#### PUT /api/v1/services/{id}/docs

指定されたサービスのドキュメントリンクを更新する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `docs[].title` | string | Yes | タイトル |
| `docs[].url` | string | Yes | ドキュメント URL |
| `docs[].doc_type` | string | Yes | ドキュメント種別（api_spec/design/runbook/other） |

**レスポンス**: 200 OK（更新後のドキュメント一覧）

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`

#### GET /api/v1/services/{id}/scorecard

指定されたサービスのスコアカードを取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `service_id` | string | サービス ID |
| `documentation_score` | int | ドキュメント充実度スコア（0-100） |
| `test_coverage` | int | テストカバレッジスコア（0-100） |
| `slo_compliance` | int | SLO 達成率スコア（0-100） |
| `security_score` | int | セキュリティスコア（0-100） |
| `overall_score` | int | 総合スコア（0-100） |
| `evaluated_at` | string | 最終評価日時（RFC 3339） |

**エラーレスポンス（404）**: `SYS_SCAT_SERVICE_NOT_FOUND`

#### GET /api/v1/services/search

タグ・カテゴリ・チーム名でサービスを横断検索する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `q` | string | Yes | - | 検索クエリ（サービス名・説明の部分一致） |
| `tag` | string | No | - | タグでフィルター |
| `tier` | string | No | - | ティアでフィルター |
| `lifecycle` | string | No | - | ライフサイクルでフィルター |

**レスポンスフィールド（200 OK）**

レスポンスフィールドは GET /api/v1/services と同一構造。

#### GET /api/v1/teams

チーム一覧を取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `teams[].id` | string | チーム ID |
| `teams[].name` | string | チーム名 |
| `teams[].description` | string | 説明 |
| `teams[].contact_email` | string | 連絡先メール |
| `teams[].slack_channel` | string | Slack チャンネル |
| `teams[].created_at` | string | 作成日時（RFC 3339） |
| `teams[].updated_at` | string | 更新日時（RFC 3339） |

#### GET /api/v1/teams/{id}/services

指定されたチームが所有するサービス一覧を取得する。レスポンスフィールドは GET /api/v1/services と同一構造。

**エラーレスポンス（404）**: `SYS_SCAT_TEAM_NOT_FOUND`

#### GET /healthz

**レスポンス**: `{ "status": "ok" }`（200 OK）

#### GET /readyz

PostgreSQL・Redis への接続を確認する。

**レスポンスフィールド（200 OK / 503）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `status` | string | `ready` / `not ready` |
| `checks.database` | string | DB 接続状態 |
| `checks.cache` | string | Redis 接続状態 |

---

## RBAC

### RBAC エンドポイント設定

| エンドポイントグループ | リソース | 必要権限 |
| --- | --- | --- |
| `GET /api/v1/services/**` | `services` | `read` |
| `POST /api/v1/services` | `services` | `write` |
| `PUT /api/v1/services/**` | `services` | `write` |
| `DELETE /api/v1/services/{id}` | `services` | `write` |
| `POST /api/v1/services/{id}/health` | `services` | `write` |
| `GET /api/v1/teams/**` | `teams` | `read` |
| `POST /api/v1/teams` | `teams` | `write` |
| `PUT /api/v1/teams/{team_id}` | `teams` | `write` |
| `DELETE /api/v1/teams/{team_id}` | `teams` | `write` |

> **権限表記**: RBAC は `resource/action` で表記する（例: `services/read`, `services/write`）。HTTP メソッドに基づいて自動判定（GET→read、POST/PUT/DELETE→write）。

---

## デプロイ

デプロイに関する詳細（Dockerfile・Helm values・Kubernetes マニフェスト・ヘルスチェック設定等）は以下を参照。

- [system-server-deploy.md](../_common/deploy.md)

ポート構成:

| プロトコル | ポート | 説明 |
| --- | --- | --- |
| REST (HTTP) | 8080 | REST API |
| gRPC | 50051 | gRPC API |

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-auth-server.md](../auth/server.md) -- 認証サーバー設計（JWT 認証の参考）
- [implementation.md](implementation.md) -- Rust 実装詳細
- [database.md](database.md) -- データベース設計
- [deploy.md](deploy.md) -- DB マイグレーション・テスト・デプロイ
