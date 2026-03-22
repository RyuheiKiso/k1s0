# system-file-database 設計

system Tier のファイルサーバーデータベース（file-db）の設計を定義する。
配置先: `regions/system/database/file-db/`

## 概要

file-db は system Tier に属する PostgreSQL 17 データベースであり、ローカルファイルシステム（PV マウント）に保存されるファイルのメタデータを管理する。ファイル実体は PV 上のローカルパスに格納し、メタデータ（ファイル名・サイズ・MIME type・タグ・所有者情報・ストレージパス）のみを PostgreSQL で管理する。AWS/S3 依存なし。

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

ローカルファイルシステム（PV）に格納されるファイルのメタデータを管理する。スキーマ名: `file_storage`。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | ファイルメタデータの一意識別子 |
| filename | VARCHAR(1024) | NOT NULL | ファイル名 |
| content_type | VARCHAR(255) | NOT NULL | MIME タイプ |
| size_bytes | BIGINT | NOT NULL, CHECK(>=0) | ファイルサイズ（バイト） |
| storage_path | VARCHAR(2048) | NOT NULL | PV 上のストレージパス |
| checksum | VARCHAR(128) | | ファイルチェックサム（SHA-256） |
| tags | JSONB | NOT NULL DEFAULT '{}' | ユーザー定義タグ |
| metadata | JSONB | NOT NULL DEFAULT '{}' | 追加メタデータ |
| status | VARCHAR(50) | NOT NULL DEFAULT 'active', CHECK制約 | ステータス（active / archived / deleted） |
| uploaded_by | UUID | | アップロードユーザー ID |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

---

## インデックス設計

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| file_metadata | idx_file_metadata_filename | filename | B-tree | ファイル名によるフィルタリング |
| file_metadata | idx_file_metadata_content_type | content_type | B-tree | MIME タイプによるフィルタリング |
| file_metadata | idx_file_metadata_status | status | B-tree | ステータスによるフィルタリング |
| file_metadata | idx_file_metadata_uploaded_by | uploaded_by | B-tree | アップロードユーザーによる検索 |
| file_metadata | idx_file_metadata_created_at | created_at | B-tree | 作成日時による範囲検索・ソート |

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
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS file_storage;

CREATE OR REPLACE FUNCTION file_storage.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

#### 002_create_file_metadata.up.sql

```sql
-- file_metadata: ファイルメタデータ管理

CREATE TABLE IF NOT EXISTS file_storage.file_metadata (
    id           UUID          PRIMARY KEY DEFAULT gen_random_uuid(),
    filename     VARCHAR(1024) NOT NULL,
    content_type VARCHAR(255)  NOT NULL,
    size_bytes   BIGINT        NOT NULL,
    storage_path VARCHAR(2048) NOT NULL,
    checksum     VARCHAR(128),
    tags         JSONB         NOT NULL DEFAULT '{}',
    metadata     JSONB         NOT NULL DEFAULT '{}',
    status       VARCHAR(50)   NOT NULL DEFAULT 'active',
    uploaded_by  UUID,
    created_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_file_metadata_status CHECK (status IN ('active', 'archived', 'deleted')),
    CONSTRAINT chk_file_metadata_size CHECK (size_bytes >= 0)
);

CREATE INDEX IF NOT EXISTS idx_file_metadata_filename
    ON file_storage.file_metadata (filename);
CREATE INDEX IF NOT EXISTS idx_file_metadata_content_type
    ON file_storage.file_metadata (content_type);
CREATE INDEX IF NOT EXISTS idx_file_metadata_status
    ON file_storage.file_metadata (status);
CREATE INDEX IF NOT EXISTS idx_file_metadata_uploaded_by
    ON file_storage.file_metadata (uploaded_by);
CREATE INDEX IF NOT EXISTS idx_file_metadata_created_at
    ON file_storage.file_metadata (created_at);

CREATE TRIGGER trigger_file_metadata_updated_at
    BEFORE UPDATE ON file_storage.file_metadata
    FOR EACH ROW
    EXECUTE FUNCTION file_storage.update_updated_at();
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
