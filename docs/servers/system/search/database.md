# system-search-server データベース設計

## スキーマ

スキーマ名: `search`

```sql
CREATE SCHEMA IF NOT EXISTS search;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| search_indices | 検索インデックス定義 |
| search_documents | 検索ドキュメント |

---

## ER 図

```
search_indices 1──* search_documents
```

---

## テーブル定義

### search_indices（検索インデックス）

検索対象のインデックスを定義する。mapping でフィールドマッピングを保持し、doc_count でドキュメント数を追跡する。

```sql
CREATE TABLE IF NOT EXISTS search.search_indices (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(255) NOT NULL UNIQUE,
    mapping    JSONB        NOT NULL DEFAULT '{}',
    doc_count  BIGINT       NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_search_indices_name ON search.search_indices (name);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | UNIQUE, NOT NULL | インデックス名 |
| mapping | JSONB | NOT NULL, DEFAULT '{}' | フィールドマッピング定義 |
| doc_count | BIGINT | NOT NULL, DEFAULT 0 | ドキュメント数 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### search_documents（検索ドキュメント）

インデックスに登録されたドキュメントを格納する。PostgreSQL の全文検索（TSVECTOR）と JSONB GIN インデックスで検索性能を確保する。pg_trgm 拡張による trigram 検索もサポートする。

```sql
CREATE TABLE IF NOT EXISTS search.search_documents (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    index_id      UUID         NOT NULL REFERENCES search.search_indices(id) ON DELETE CASCADE,
    document_id   VARCHAR(255) NOT NULL,
    content       JSONB        NOT NULL DEFAULT '{}',
    search_vector TSVECTOR,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_search_documents_index_doc UNIQUE (index_id, document_id)
);

CREATE INDEX IF NOT EXISTS idx_search_documents_index_id ON search.search_documents (index_id);
CREATE INDEX IF NOT EXISTS idx_search_documents_search_vector ON search.search_documents USING GIN (search_vector);
CREATE INDEX IF NOT EXISTS idx_search_documents_content ON search.search_documents USING GIN (content jsonb_path_ops);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| index_id | UUID | FK → search_indices.id, NOT NULL | インデックス ID |
| document_id | VARCHAR(255) | UNIQUE(index_id, document_id), NOT NULL | ドキュメント ID |
| content | JSONB | NOT NULL, DEFAULT '{}' | ドキュメント本体 |
| search_vector | TSVECTOR | | 全文検索ベクトル（自動生成） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/search-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `search` スキーマ・pgcrypto/pg_trgm 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_search_indices.up.sql` | search_indices テーブル作成 |
| `002_create_search_indices.down.sql` | テーブル削除 |
| `003_create_search_documents.up.sql` | search_documents テーブル・search_vector トリガー作成 |
| `003_create_search_documents.down.sql` | テーブル削除 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION search.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_search_indices_update_updated_at
    BEFORE UPDATE ON search.search_indices
    FOR EACH ROW EXECUTE FUNCTION search.update_updated_at();
```

## search_vector 自動生成トリガー

```sql
CREATE OR REPLACE FUNCTION search.update_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector = to_tsvector('simple', COALESCE(NEW.content::text, ''));
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_search_documents_update_vector
    BEFORE INSERT OR UPDATE ON search.search_documents
    FOR EACH ROW EXECUTE FUNCTION search.update_search_vector();
```
