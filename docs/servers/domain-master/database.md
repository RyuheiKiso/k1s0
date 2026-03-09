# business-domain-master-server データベース設計

## スキーマ

スキーマ名: `domain_master`

```sql
CREATE SCHEMA IF NOT EXISTS domain_master;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| master_categories | マスタカテゴリ定義（勘定科目、部門コード等） |
| master_items | カテゴリ配下のマスタ項目 |
| master_item_versions | マスタ項目の変更履歴 |
| tenant_master_extensions | テナント別マスタ項目カスタマイズ |

---

## ER 図

```
master_categories 1──* master_items 1──* master_item_versions
                                    1──* tenant_master_extensions
master_items (self FK: parent_item_id → master_items.id)
```

---

## テーブル定義

### master_categories（カテゴリ定義）

マスタデータのカテゴリを定義する。`validation_schema` でカテゴリ配下項目の attributes バリデーションルールを保持。

```sql
CREATE TABLE domain_master.master_categories (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code              VARCHAR(100) NOT NULL UNIQUE,
    display_name      VARCHAR(255) NOT NULL,
    description       TEXT,
    validation_schema JSONB,
    is_active         BOOLEAN NOT NULL DEFAULT true,
    sort_order        INTEGER NOT NULL DEFAULT 0,
    created_by        VARCHAR(255) NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_master_categories_code ON domain_master.master_categories(code);
CREATE INDEX idx_master_categories_active ON domain_master.master_categories(is_active);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| code | VARCHAR(100) | UNIQUE, NOT NULL | カテゴリコード（API パスで使用） |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| description | TEXT | | 説明 |
| validation_schema | JSONB | | JSON Schema 形式のバリデーションルール |
| is_active | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| sort_order | INTEGER | NOT NULL, DEFAULT 0 | 表示順 |
| created_by | VARCHAR(255) | NOT NULL | 作成者 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

---

### master_items（マスタ項目）

カテゴリ配下の個々のマスタ項目。階層構造（parent_item_id）と有効期間（effective_from/until）をサポート。

```sql
CREATE TABLE domain_master.master_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id     UUID NOT NULL REFERENCES domain_master.master_categories(id) ON DELETE CASCADE,
    code            VARCHAR(255) NOT NULL,
    display_name    VARCHAR(255) NOT NULL,
    description     TEXT,
    attributes      JSONB,
    parent_item_id  UUID REFERENCES domain_master.master_items(id) ON DELETE SET NULL,
    effective_from  TIMESTAMPTZ,
    effective_until TIMESTAMPTZ,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_by      VARCHAR(255) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_master_items_category_code UNIQUE (category_id, code)
);

CREATE INDEX idx_master_items_category ON domain_master.master_items(category_id);
CREATE INDEX idx_master_items_parent ON domain_master.master_items(parent_item_id);
CREATE INDEX idx_master_items_active ON domain_master.master_items(is_active);
CREATE INDEX idx_master_items_effective ON domain_master.master_items(effective_from, effective_until);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| category_id | UUID | FK → master_categories.id, NOT NULL | 所属カテゴリ |
| code | VARCHAR(255) | UNIQUE(category_id, code), NOT NULL | 項目コード |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| description | TEXT | | 説明 |
| attributes | JSONB | | 項目属性（カテゴリの validation_schema でバリデーション） |
| parent_item_id | UUID | FK → master_items.id | 親項目（階層構造用） |
| effective_from | DATE | | 有効開始日 |
| effective_until | DATE | | 有効終了日 |
| is_active | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| sort_order | INTEGER | NOT NULL, DEFAULT 0 | 表示順 |
| created_by | VARCHAR(255) | NOT NULL | 作成者 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

---

### master_item_versions（変更履歴）

項目更新時に before/after 差分を自動記録する。

```sql
CREATE TABLE domain_master.master_item_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id         UUID NOT NULL REFERENCES domain_master.master_items(id) ON DELETE CASCADE,
    version_number  INTEGER NOT NULL,
    before_data     JSONB,
    after_data      JSONB,
    changed_by      VARCHAR(255) NOT NULL,
    change_reason   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_master_item_versions_item_version UNIQUE (item_id, version_number)
);

CREATE INDEX idx_master_item_versions_item ON domain_master.master_item_versions(item_id, created_at DESC);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| item_id | UUID | FK → master_items.id, NOT NULL | 対象項目 |
| version_number | INTEGER | NOT NULL | バージョン番号（項目ごとに連番） |
| before_data | JSONB | | 変更前データ |
| after_data | JSONB | | 変更後データ |
| changed_by | VARCHAR(255) | NOT NULL | 変更者 |
| change_reason | TEXT | | 変更理由 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 記録日時 |

---

### tenant_master_extensions（テナントカスタマイズ）

テナントごとにマスタ項目の表示名・属性をオーバーライドし、有効/無効を制御する。

```sql
CREATE TABLE domain_master.tenant_master_extensions (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             VARCHAR(255) NOT NULL,
    item_id               UUID NOT NULL REFERENCES domain_master.master_items(id) ON DELETE CASCADE,
    display_name_override VARCHAR(255),
    attributes_override   JSONB,
    is_enabled            BOOLEAN NOT NULL DEFAULT true,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_tenant_master_extensions_tenant_item UNIQUE (tenant_id, item_id)
);

CREATE INDEX idx_tenant_master_extensions_tenant ON domain_master.tenant_master_extensions(tenant_id);
CREATE INDEX idx_tenant_master_extensions_item ON domain_master.tenant_master_extensions(item_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| tenant_id | VARCHAR(255) | UNIQUE(tenant_id, item_id), NOT NULL | テナント ID |
| item_id | UUID | FK → master_items.id, UNIQUE(tenant_id, item_id), NOT NULL | 対象項目 |
| display_name_override | VARCHAR(255) | | 表示名オーバーライド（null の場合ベース項目の表示名を使用） |
| attributes_override | JSONB | | 属性オーバーライド（ベース attributes に JSONB マージ） |
| is_enabled | BOOLEAN | NOT NULL, DEFAULT true | テナントでの有効/無効制御 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

---

## マイグレーション

マイグレーションファイルは `regions/business/accounting/database/domain-master-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `domain_master` スキーマ作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_domain_master_tables.up.sql` | 全テーブル（master_categories, master_items, master_item_versions, tenant_master_extensions）・インデックス・制約の作成 |
| `002_create_domain_master_tables.down.sql` | 全テーブル削除 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION domain_master.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_master_categories_updated_at
    BEFORE UPDATE ON domain_master.master_categories
    FOR EACH ROW EXECUTE FUNCTION domain_master.update_updated_at();

CREATE TRIGGER trg_master_items_updated_at
    BEFORE UPDATE ON domain_master.master_items
    FOR EACH ROW EXECUTE FUNCTION domain_master.update_updated_at();

CREATE TRIGGER trg_tenant_master_extensions_updated_at
    BEFORE UPDATE ON domain_master.tenant_master_extensions
    FOR EACH ROW EXECUTE FUNCTION domain_master.update_updated_at();
```
