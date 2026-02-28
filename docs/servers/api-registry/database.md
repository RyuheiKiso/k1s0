# system-api-registry-database 設計

system tier の API スキーマレジストリ用 PostgreSQL データベース設計を定義する。

## 概要

| 項目 | 設計 |
| --- | --- |
| DB | PostgreSQL 17 |
| スキーマ | `apiregistry` |
| テーブル数 | 2 (api_schemas, api_schema_versions) |

---

## テーブル定義

### api_schemas

API スキーマのメタデータを管理するテーブル。

| カラム | 型 | 制約 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `name` | VARCHAR(255) | PRIMARY KEY | - | スキーマ名（一意識別子） |
| `description` | TEXT | NOT NULL | `''` | スキーマの説明 |
| `schema_type` | VARCHAR(50) | NOT NULL, CHECK | - | スキーマ種別（`openapi` / `protobuf`） |
| `latest_version` | INT | NOT NULL | `0` | 最新バージョン番号 |
| `version_count` | INT | NOT NULL | `0` | 登録バージョン数 |
| `created_at` | TIMESTAMPTZ | NOT NULL | `NOW()` | 作成日時 |
| `updated_at` | TIMESTAMPTZ | NOT NULL | `NOW()` | 更新日時 |

**CHECK 制約:**

- `ck_schema_type`: `schema_type IN ('openapi', 'protobuf')`

### api_schema_versions

API スキーマの各バージョンのコンテンツを管理するテーブル。

| カラム | 型 | 制約 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `id` | UUID | PRIMARY KEY | `gen_random_uuid()` | バージョンの一意識別子 |
| `name` | VARCHAR(255) | NOT NULL, FK | - | スキーマ名（api_schemas.name を参照、CASCADE 削除） |
| `version` | INT | NOT NULL | - | バージョン番号 |
| `schema_type` | VARCHAR(50) | NOT NULL | - | スキーマ種別 |
| `content` | TEXT | NOT NULL | - | スキーマ本文（YAML/JSON/proto） |
| `content_hash` | VARCHAR(255) | NOT NULL | - | コンテンツの SHA-256 ハッシュ |
| `breaking_changes` | BOOLEAN | NOT NULL | `FALSE` | 破壊的変更フラグ |
| `registered_by` | VARCHAR(255) | NOT NULL | - | 登録者のユーザー ID |
| `created_at` | TIMESTAMPTZ | NOT NULL | `NOW()` | 登録日時 |

**UNIQUE 制約:**

- `uq_schema_version`: `(name, version)` の組み合わせで一意

**外部キー:**

- `name` -> `apiregistry.api_schemas(name)` ON DELETE CASCADE

---

## インデックス定義

| インデックス名 | テーブル | カラム | 説明 |
| --- | --- | --- | --- |
| `idx_api_schemas_schema_type` | api_schemas | `schema_type` | スキーマ種別によるフィルタリング |
| `idx_api_schemas_created_at` | api_schemas | `created_at DESC` | 作成日時の降順ソート |
| `idx_api_schema_versions_name` | api_schema_versions | `name` | スキーマ名による検索 |
| `idx_api_schema_versions_name_version` | api_schema_versions | `name, version DESC` | スキーマ名 + バージョン降順の複合検索 |
| `idx_api_schema_versions_created_at` | api_schema_versions | `created_at DESC` | 作成日時の降順ソート |

---

## 接続設定

| 項目 | デフォルト値 |
| --- | --- |
| ホスト | localhost |
| ポート | 5432 |
| DB 名 | k1s0_system |
| スキーマ | apiregistry |
| ユーザー | app |
| 最大接続数 | 10 |

---

## バックアップ方針

- 日次フルバックアップ
- WAL アーカイブによる PITR 対応
- 保持期間: 30日

---

## マイグレーション管理

`regions/system/database/api-registry-db/migrations/` 配下に連番 SQL ファイルで管理。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `apiregistry` スキーマ作成、pgcrypto 拡張有効化 |
| `002_create_api_schemas.up.sql` | `api_schemas` テーブル作成 |
| `003_create_api_schema_versions.up.sql` | `api_schema_versions` テーブル作成 |
| `004_create_indexes.up.sql` | インデックス作成 |
