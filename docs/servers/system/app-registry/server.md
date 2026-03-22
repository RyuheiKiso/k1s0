# system-app-registry-server 設計

system tier のアプリケーションレジストリサーバー。REST API でアプリ管理・バージョン管理・ファイル直接配信機能を提供する。Rust 実装。

## 概要

| 機能 | 説明 |
| --- | --- |
| アプリ一覧・詳細取得 | 登録済みアプリケーションの一覧取得・詳細参照 |
| バージョン管理 | アプリケーションのバージョン作成・削除・一覧取得 |
| 最新バージョン取得 | プラットフォーム・アーキテクチャ指定で最新バージョンを取得 |
| ファイル直接配信 | サーバーがローカルFS（PV）からファイルをバイナリストリーミング配信。外部ストレージ不要 |
| ダウンロード統計 | ダウンロードイベントの記録・集計 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| ストレージバックエンド | ローカルファイルシステム（tokio::fs）。AWS 依存なし |
| JWT 認証 | k1s0-auth ライブラリ |

### 配置パス

配置: `regions/system/server/rust/app-registry/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_APPS_` とする。gRPC は提供しない（REST only）。

#### 公開エンドポイント（認証不要）

| Method | Path | Description |
| --- | --- | --- |
| GET | `/healthz` | ヘルスチェック |
| GET | `/readyz` | レディネスチェック（DB 接続確認） |
| GET | `/metrics` | Prometheus メトリクス |

#### 保護エンドポイント（Bearer トークン + RBAC 必要）

| Method | Path | Description | RBAC リソース | RBAC 権限 |
| --- | --- | --- | --- | --- |
| GET | `/api/v1/apps` | アプリ一覧取得 | `apps` | `read` |
| GET | `/api/v1/apps/{id}` | アプリ詳細取得 | `apps` | `read` |
| GET | `/api/v1/apps/{id}/versions` | バージョン一覧取得 | `apps` | `read` |
| GET | `/api/v1/apps/{id}/latest` | 最新バージョン取得 | `apps` | `read` |
| GET | `/api/v1/apps/{id}/versions/{version}/download` | ダウンロードURL生成 | `apps` | `read` |
| GET | `/api/v1/apps/{id}/stats` | ダウンロード統計取得 | `apps` | `read` |
| POST | `/api/v1/apps` | アプリ作成 | `apps` | `write` |
| PUT | `/api/v1/apps/{id}` | アプリ更新 | `apps` | `write` |
| DELETE | `/api/v1/apps/{id}` | アプリ削除 | `apps` | `write` |
| POST | `/api/v1/apps/{id}/versions` | バージョン作成 | `apps` | `write` |
| DELETE | `/api/v1/apps/{id}/versions/{version}` | バージョン削除 | `apps` | `write` |

### 共通エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_APPS_INTERNAL_ERROR` | 500 | 内部エラー |
| `SYS_APPS_APP_NOT_FOUND` | 404 | 指定されたアプリが見つからない |
| `SYS_APPS_VERSION_NOT_FOUND` | 404 | 指定されたバージョンが見つからない |
| `SYS_APPS_INVALID_PLATFORM` | 400 | 無効なプラットフォーム指定 |
| `SYS_APPS_CREATE_VERSION_FAILED` | 400 | バージョン作成に失敗 |
| `SYS_APPS_VALIDATION_ERROR` | 400 | バリデーションエラー（空の名前・カテゴリ等） |
| `SYS_APPS_MISSING_TOKEN` | 401 | Authorization ヘッダーが存在しない |
| `SYS_APPS_PERMISSION_DENIED` | 403 | 要求された resource/action の権限がない |

#### POST /api/v1/apps

新しいアプリケーションを作成する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `name` | string | Yes | アプリ名（空不可） |
| `description` | string | No | 説明 |
| `category` | string | Yes | カテゴリ（空不可） |
| `icon_url` | string | No | アイコン URL |

**レスポンスフィールド（201 Created）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string | アプリ ID（UUID v4、サーバー生成） |
| `name` | string | アプリ名 |
| `description` | string | 説明 |
| `category` | string | カテゴリ |
| `icon_url` | string | アイコン URL |
| `created_at` | string | 作成日時（RFC 3339） |
| `updated_at` | string | 更新日時（RFC 3339） |

**エラーレスポンス（400）**: `SYS_APPS_VALIDATION_ERROR`（名前またはカテゴリが空）

#### PUT /api/v1/apps/{id}

指定された ID のアプリケーションを更新する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `name` | string | Yes | アプリ名（空不可） |
| `description` | string | No | 説明 |
| `category` | string | Yes | カテゴリ |
| `icon_url` | string | No | アイコン URL |

**レスポンスフィールド（200 OK）**

レスポンスフィールドは POST /api/v1/apps と同一構造。`created_at` は変更されず、`updated_at` が更新される。

**エラーレスポンス（404）**: `SYS_APPS_APP_NOT_FOUND`
**エラーレスポンス（400）**: `SYS_APPS_VALIDATION_ERROR`

#### DELETE /api/v1/apps/{id}

指定された ID のアプリケーションを削除する。

**レスポンス**: 204 No Content

**エラーレスポンス（404）**: `SYS_APPS_APP_NOT_FOUND`

#### GET /api/v1/apps

アプリケーション一覧をカテゴリ・検索条件で取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `category` | string | No | - | カテゴリでフィルター |
| `search` | string | No | - | アプリ名・説明の部分一致検索 |

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `apps[].id` | string | アプリ ID |
| `apps[].name` | string | アプリ名 |
| `apps[].description` | string | 説明 |
| `apps[].category` | string | カテゴリ |
| `apps[].icon_url` | string | アイコン URL |
| `apps[].created_at` | string | 作成日時（RFC 3339） |
| `apps[].updated_at` | string | 更新日時（RFC 3339） |

#### GET /api/v1/apps/{id}

指定された ID のアプリケーション詳細を取得する。レスポンスフィールドは GET /api/v1/apps の `apps[]` と同一。

**エラーレスポンス（404）**: `SYS_APPS_APP_NOT_FOUND`

#### GET /api/v1/apps/{id}/versions

指定されたアプリのバージョン一覧を取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `versions[].id` | string | バージョン ID（UUID） |
| `versions[].app_id` | string | アプリ ID |
| `versions[].version` | string | バージョン文字列 |
| `versions[].platform` | string | プラットフォーム（Windows/Linux/Macos） |
| `versions[].arch` | string | アーキテクチャ |
| `versions[].size_bytes` | int64 | ファイルサイズ（バイト） |
| `versions[].checksum_sha256` | string | SHA-256 チェックサム |
| `versions[].release_notes` | string | リリースノート |
| `versions[].mandatory` | bool | 強制アップデートフラグ |
| `versions[].published_at` | string | 公開日時（RFC 3339） |
| `versions[].created_at` | string | 作成日時（RFC 3339） |

**エラーレスポンス（404）**: `SYS_APPS_APP_NOT_FOUND`

#### POST /api/v1/apps/{id}/versions

新しいバージョンを作成する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `version` | string | Yes | バージョン文字列 |
| `platform` | string | Yes | プラットフォーム（Windows/Linux/Macos） |
| `arch` | string | Yes | アーキテクチャ |
| `size_bytes` | int64 | Yes | ファイルサイズ（バイト） |
| `checksum_sha256` | string | Yes | SHA-256 チェックサム |
| `storage_key` | string | Yes | サーバー上のファイル保存パス（例: `app-id/1.0.0/windows-x64/app.exe`） |
| `release_notes` | string | No | リリースノート |
| `mandatory` | bool | No | 強制アップデートフラグ（デフォルト: false） |

**レスポンスフィールド（201 Created）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string | バージョン ID（UUID） |
| `created_at` | string | 作成日時（RFC 3339） |

**エラーレスポンス（400）**: `SYS_APPS_CREATE_VERSION_FAILED` / `SYS_APPS_INVALID_PLATFORM`

#### DELETE /api/v1/apps/{id}/versions/{version}

指定されたバージョンを削除する。

**レスポンス**: 204 No Content

**エラーレスポンス（404）**: `SYS_APPS_VERSION_NOT_FOUND`

#### GET /api/v1/apps/{id}/latest

指定されたアプリの最新バージョンを取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `platform` | string | No | - | プラットフォームでフィルター |
| `arch` | string | No | - | アーキテクチャでフィルター |

**レスポンスフィールド（200 OK）**

レスポンスフィールドは GET /api/v1/apps/{id}/versions の `versions[]` と同一構造。

**エラーレスポンス（404）**: `SYS_APPS_APP_NOT_FOUND` / `SYS_APPS_VERSION_NOT_FOUND`

#### GET /api/v1/apps/{id}/versions/{version}/download

指定されたバージョンのファイルを直接バイナリストリーミング配信する。ダウンロード統計も記録する。
サーバーはローカルFS（PV）からファイルを読み込み、`Content-Type` と `Content-Disposition: attachment` ヘッダーを付与してレスポンスとして返す。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `platform` | string | No | - | プラットフォームでフィルター |
| `arch` | string | No | - | アーキテクチャでフィルター |

**レスポンス（200 OK）**

バイナリファイルストリーム。主要レスポンスヘッダー:

| ヘッダー | 説明 |
| --- | --- |
| `Content-Type` | ファイル拡張子から推定（例: `application/octet-stream`） |
| `Content-Disposition` | `attachment; filename="<filename>"` |
| `Content-Length` | ファイルサイズ（バイト） |

**エラーレスポンス（404）**: `SYS_APPS_VERSION_NOT_FOUND`
**エラーレスポンス（400）**: `SYS_APPS_INVALID_PLATFORM`

#### GET /api/v1/apps/{id}/stats

指定されたアプリのダウンロード統計を取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `total_downloads` | int64 | 全バージョンの合計ダウンロード数 |
| `version_downloads` | int64 | 最新バージョンのダウンロード数 |
| `latest_version` | string \| null | 最新バージョン番号（バージョン未登録時は null） |

**エラーレスポンス（404）**: `SYS_APPS_APP_NOT_FOUND`

#### GET /healthz

**レスポンス**: `{ "status": "ok" }`（200 OK）

#### GET /readyz

PostgreSQL への接続を確認する。

**レスポンスフィールド（200 OK / 503）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `status` | string | `ready` / `not ready` |
| `checks.database` | string | DB 接続状態 |

---

## RBAC

### RBAC エンドポイント設定

| エンドポイントグループ | リソース | 必要権限 |
| --- | --- | --- |
| `GET /api/v1/apps/**` | `apps` | `read` |
| `POST /api/v1/apps` | `apps` | `write` |
| `PUT /api/v1/apps/{id}` | `apps` | `write` |
| `DELETE /api/v1/apps/{id}` | `apps` | `write` |
| `POST /api/v1/apps/{id}/versions` | `apps` | `write` |
| `DELETE /api/v1/apps/{id}/versions/{version}` | `apps` | `write` |

> **権限表記**: RBAC は `resource/action` で表記する（例: `apps/read`, `apps/write`）。

---

## デプロイ

デプロイに関する詳細（Dockerfile・Helm values・Kubernetes マニフェスト・ヘルスチェック設定等）は以下を参照。

- [system-server-deploy.md](../_common/deploy.md)

ポート構成:

| プロトコル | ポート | 説明 |
| --- | --- | --- |
| REST (HTTP) | 8080 | REST API |

> **注**: 本サーバーは REST API のみ提供する。gRPC は提供しない。

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-auth-server.md](../auth/server.md) -- 認証サーバー設計（JWT 認証の参考）
