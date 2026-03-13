# system-file-database 設計

system Tier のファイルサーバーデータベース（file-db）の設計を定義する。
配置先: `regions/system/database/file-db/`

## 概要

file-db は system Tier に属する PostgreSQL 17 データベースであり、S3 互換ストレージに保存されるファイルのメタデータを管理する。ファイル実体はオブジェクトストレージに格納し、メタデータ（ファイル名・サイズ・MIME type・タグ・所有者情報）のみを PostgreSQL で管理する。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、file-db へのアクセスは **system Tier のサーバーからのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli | - |
| ORM / クエリビルダー | sqlx | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## テーブル定義

### file_metadata テーブル

S3 互換ストレージに格納されるファイルのメタデータを管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | ファイルメタデータの一意識別子 |
| tenant_id | VARCHAR(255) | NOT NULL | テナント ID（テナント分離用） |
| bucket | VARCHAR(255) | NOT NULL | S3 バケット名 |
| object_key | VARCHAR(1024) | NOT NULL | S3 オブジェクトキー（パス） |
| file_name | VARCHAR(255) | NOT NULL | 元のファイル名 |
| content_type | VARCHAR(255) | NOT NULL DEFAULT 'application/octet-stream' | MIME タイプ |
| size_bytes | BIGINT | NOT NULL | ファイルサイズ（バイト） |
| checksum | VARCHAR(255) | | ファイルチェックサム（SHA-256） |
| tags | JSONB | NOT NULL DEFAULT '{}' | ユーザー定義タグ |
| owner_id | VARCHAR(255) | | ファイル所有者 ID |
| status | VARCHAR(50) | NOT NULL DEFAULT 'ACTIVE', CHECK制約 | ステータス（ACTIVE / ARCHIVED / DELETED） |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |
| | | UNIQUE(bucket, object_key) | バケットとオブジェクトキーの組み合わせで一意 |

---

## インデックス設計

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| file_metadata | idx_file_metadata_tenant_id | tenant_id | B-tree | テナント ID によるフィルタリング |
| file_metadata | idx_file_metadata_bucket_object_key | (bucket, object_key) | B-tree（UNIQUE） | バケット・キーによる一意検索 |
| file_metadata | idx_file_metadata_owner_id | owner_id (WHERE NOT NULL) | B-tree（部分） | 所有者によるファイル検索 |
| file_metadata | idx_file_metadata_content_type | content_type | B-tree | MIME タイプによるフィルタリング |
| file_metadata | idx_file_metadata_status | status | B-tree | ステータスによるフィルタリング |
| file_metadata | idx_file_metadata_created_at | created_at | B-tree | 作成日時による範囲検索・ソート |
| file_metadata | idx_file_metadata_tags | tags | GIN | タグによる JSONB 検索 |

---

## マイグレーションファイル

配置先: `regions/system/database/file-db/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠する。

```
migrations/
├── 001_create_schema.up.sql                    # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_file_metadata.up.sql             # file_metadata テーブル
└── 002_create_file_metadata.down.sql
```

### マイグレーション SQL

#### 001_create_schema.up.sql

```sql
-- file-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE SCHEMA IF NOT EXISTS file;

CREATE OR REPLACE FUNCTION file.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

#### 002_create_file_metadata.up.sql

```sql
-- file-db: file_metadata テーブル作成

CREATE TABLE IF NOT EXISTS file.file_metadata (
    id            UUID          PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     VARCHAR(255)  NOT NULL,
    bucket        VARCHAR(255)  NOT NULL,
    object_key    VARCHAR(1024) NOT NULL,
    file_name     VARCHAR(255)  NOT NULL,
    content_type  VARCHAR(255)  NOT NULL DEFAULT 'application/octet-stream',
    size_bytes    BIGINT        NOT NULL,
    checksum      VARCHAR(255),
    tags          JSONB         NOT NULL DEFAULT '{}',
    owner_id      VARCHAR(255),
    status        VARCHAR(50)   NOT NULL DEFAULT 'ACTIVE',
    created_at    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_file_metadata_bucket_object_key UNIQUE (bucket, object_key),
    CONSTRAINT chk_file_metadata_status CHECK (status IN ('ACTIVE', 'ARCHIVED', 'DELETED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_file_metadata_tenant_id
    ON file.file_metadata (tenant_id);
CREATE INDEX IF NOT EXISTS idx_file_metadata_owner_id
    ON file.file_metadata (owner_id)
    WHERE owner_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_file_metadata_content_type
    ON file.file_metadata (content_type);
CREATE INDEX IF NOT EXISTS idx_file_metadata_status
    ON file.file_metadata (status);
CREATE INDEX IF NOT EXISTS idx_file_metadata_created_at
    ON file.file_metadata (created_at);
CREATE INDEX IF NOT EXISTS idx_file_metadata_tags
    ON file.file_metadata USING GIN (tags);

-- updated_at トリガー
CREATE TRIGGER trigger_file_metadata_update_updated_at
    BEFORE UPDATE ON file.file_metadata
    FOR EACH ROW
    EXECUTE FUNCTION file.update_updated_at();
```

---

## 接続設定

[config設計](../../cli/config/config設計.md) の database セクションに従い、file-db への接続を設定する。

### 環境別設定

| 環境 | host | ssl_mode | max_open_conns | max_idle_conns |
|------|------|----------|----------------|----------------|
| dev | localhost (docker-compose) | disable | 10 | 3 |
| staging | postgres.k1s0-system.svc.cluster.local | require | 25 | 5 |
| prod | postgres.k1s0-system.svc.cluster.local | verify-full | 50 | 10 |

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/file/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/system-file-rw` | Vault Database エンジンで自動生成（TTL: 24時間） |
| 動的クレデンシャル（読み取り専用） | `database/creds/system-file-ro` | Vault Database エンジンで自動生成（TTL: 24時間） |

### docker-compose（ローカル開発）

[docker-compose設計](../../infrastructure/docker/docker-compose設計.md) の共通 PostgreSQL インスタンスに `k1s0_system` データベースを使用する。file-db は `k1s0_system` データベース内の `file` スキーマとして共存する。

---

## 関連ドキュメント

- [system-file-server設計](server.md) -- ファイルサーバー設計
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [config設計](../../cli/config/config設計.md) -- config.yaml スキーマ（database セクション）
- [テンプレート仕様-データベース](../../templates/data/データベース.md) -- マイグレーション命名規則・テンプレート
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) -- ローカル開発用 PostgreSQL
