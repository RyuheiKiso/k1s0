# business-project-master-server データベース設計

business Tier のプロジェクトマスタデータベース（project-master-db）の設計を定義する。
配置先: `regions/business/taskmanagement/database/project-master-db/`

## スキーマ

スキーマ名: `project_master`

```sql
CREATE SCHEMA IF NOT EXISTS project_master;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| project_types | プロジェクトタイプ定義（開発・運用・マーケティング等） |
| status_definitions | プロジェクトタイプごとのステータス定義 |
| status_definition_versions | ステータス定義の変更履歴 |
| tenant_project_extensions | テナント別プロジェクトタイプカスタマイズ |

---

## ER 図

```
project_types 1──* status_definitions 1──* status_definition_versions
project_types 1──* tenant_project_extensions
```

---

## テーブル定義

### project_types（プロジェクトタイプ定義）

プロジェクトの分類を定義する。`version` フィールドにより楽観的ロックを実現する。

```sql
CREATE TABLE project_master.project_types (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code         VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description  TEXT,
    is_active    BOOLEAN NOT NULL DEFAULT true,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    version      INTEGER NOT NULL DEFAULT 1,
    created_by   VARCHAR(255) NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_project_types_code ON project_master.project_types(code);
CREATE INDEX idx_project_types_active ON project_master.project_types(is_active);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| code | VARCHAR(100) | UNIQUE, NOT NULL | プロジェクトタイプコード（API パスで使用） |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| description | TEXT | | 説明 |
| is_active | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| sort_order | INTEGER | NOT NULL, DEFAULT 0 | 表示順 |
| version | INTEGER | NOT NULL, DEFAULT 1 | 楽観的ロック用バージョン番号 |
| created_by | VARCHAR(255) | NOT NULL | 作成者 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

---

### status_definitions（ステータス定義）

プロジェクトタイプごとのステータスを定義する。テナント固有のステータスは `tenant_id` で識別する。RLS ポリシーにより、テナントは自身のデータのみアクセス可能。

```sql
CREATE TABLE project_master.status_definitions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_type_id UUID NOT NULL REFERENCES project_master.project_types(id) ON DELETE CASCADE,
    code            VARCHAR(100) NOT NULL,
    display_name    VARCHAR(255) NOT NULL,
    description     TEXT,
    color           VARCHAR(7),
    is_terminal     BOOLEAN NOT NULL DEFAULT false,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    version         INTEGER NOT NULL DEFAULT 1,
    tenant_id       VARCHAR(255),
    created_by      VARCHAR(255) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_status_definitions_type_code UNIQUE (project_type_id, code)
);

CREATE INDEX idx_status_definitions_project_type ON project_master.status_definitions(project_type_id);
CREATE INDEX idx_status_definitions_tenant ON project_master.status_definitions(tenant_id);
CREATE INDEX idx_status_definitions_active_sort ON project_master.status_definitions(project_type_id, sort_order);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| project_type_id | UUID | FK → project_types.id, NOT NULL | 所属プロジェクトタイプ |
| code | VARCHAR(100) | UNIQUE(project_type_id, code), NOT NULL | ステータスコード |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| description | TEXT | | 説明 |
| color | VARCHAR(7) | | 表示色（HEX 形式: #RRGGBB） |
| is_terminal | BOOLEAN | NOT NULL, DEFAULT false | 終端ステータスフラグ |
| sort_order | INTEGER | NOT NULL, DEFAULT 0 | 表示順 |
| version | INTEGER | NOT NULL, DEFAULT 1 | 楽観的ロック用バージョン番号 |
| tenant_id | VARCHAR(255) | | テナント ID（NULL = 共通定義） |
| created_by | VARCHAR(255) | NOT NULL | 作成者 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

#### RLS（Row Level Security）

テナント別ステータス定義は RLS ポリシーにより、`tenant_id` が一致するレコードまたは `tenant_id IS NULL`（共通定義）のみアクセスを許可する。

```sql
ALTER TABLE project_master.status_definitions ENABLE ROW LEVEL SECURITY;

CREATE POLICY status_definitions_tenant_policy ON project_master.status_definitions
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.tenant_id', true));
```

---

### status_definition_versions（ステータス定義変更履歴）

ステータス定義更新時に before/after 差分を自動記録する。

```sql
CREATE TABLE project_master.status_definition_versions (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status_definition_id UUID NOT NULL REFERENCES project_master.status_definitions(id) ON DELETE CASCADE,
    version_number       INTEGER NOT NULL,
    before_data          JSONB,
    after_data           JSONB,
    changed_by           VARCHAR(255) NOT NULL,
    change_reason        TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_status_definition_versions_def_version UNIQUE (status_definition_id, version_number)
);

CREATE INDEX idx_status_definition_versions_def ON project_master.status_definition_versions(status_definition_id, created_at DESC);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| status_definition_id | UUID | FK → status_definitions.id, NOT NULL | 対象ステータス定義 |
| version_number | INTEGER | NOT NULL | バージョン番号（定義ごとに連番） |
| before_data | JSONB | | 変更前データ |
| after_data | JSONB | | 変更後データ |
| changed_by | VARCHAR(255) | NOT NULL | 変更者 |
| change_reason | TEXT | | 変更理由 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 記録日時 |

---

### tenant_project_extensions（テナントプロジェクト拡張）

テナントごとにプロジェクトタイプの表示名・属性をオーバーライドし、有効/無効を制御する。RLS ポリシーにより tenant_id ベースのアクセス制御を行う。

```sql
CREATE TABLE project_master.tenant_project_extensions (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             VARCHAR(255) NOT NULL,
    project_type_id       UUID NOT NULL REFERENCES project_master.project_types(id) ON DELETE CASCADE,
    display_name_override VARCHAR(255),
    attributes_override   JSONB,
    is_enabled            BOOLEAN NOT NULL DEFAULT true,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_tenant_project_extensions_tenant_type UNIQUE (tenant_id, project_type_id)
);

CREATE INDEX idx_tenant_project_extensions_tenant ON project_master.tenant_project_extensions(tenant_id);
CREATE INDEX idx_tenant_project_extensions_type ON project_master.tenant_project_extensions(project_type_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| tenant_id | VARCHAR(255) | UNIQUE(tenant_id, project_type_id), NOT NULL | テナント ID |
| project_type_id | UUID | FK → project_types.id, UNIQUE(tenant_id, project_type_id), NOT NULL | 対象プロジェクトタイプ |
| display_name_override | VARCHAR(255) | | 表示名オーバーライド（null の場合ベースの表示名を使用） |
| attributes_override | JSONB | | 属性オーバーライド（ベース attributes に JSONB マージ） |
| is_enabled | BOOLEAN | NOT NULL, DEFAULT true | テナントでの有効/無効制御 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

#### RLS（Row Level Security）

```sql
ALTER TABLE project_master.tenant_project_extensions ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_project_extensions_tenant_policy ON project_master.tenant_project_extensions
    USING (tenant_id = current_setting('app.tenant_id', true));
```

---

## マイグレーション

マイグレーションファイルは `regions/business/taskmanagement/database/project-master-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `project_master` スキーマ作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_project_master_tables.up.sql` | 全テーブル（project_types, status_definitions, status_definition_versions, tenant_project_extensions）・インデックス・制約の作成 |
| `002_create_project_master_tables.down.sql` | 全テーブル削除 |
| `003_add_rls_policies.up.sql` | RLS ポリシー設定（status_definitions, tenant_project_extensions） |
| `003_add_rls_policies.down.sql` | RLS ポリシー削除 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION project_master.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_project_types_updated_at
    BEFORE UPDATE ON project_master.project_types
    FOR EACH ROW EXECUTE FUNCTION project_master.update_updated_at();

CREATE TRIGGER trg_status_definitions_updated_at
    BEFORE UPDATE ON project_master.status_definitions
    FOR EACH ROW EXECUTE FUNCTION project_master.update_updated_at();

CREATE TRIGGER trg_tenant_project_extensions_updated_at
    BEFORE UPDATE ON project_master.tenant_project_extensions
    FOR EACH ROW EXECUTE FUNCTION project_master.update_updated_at();
```

---

## DB 初期化スクリプト

Docker Compose による開発環境起動時に `infra/docker/init-db/14-project-master-schema.sql` を実行してスキーマを初期化する。

```sql
-- 開発環境用初期データ（プロジェクトタイプサンプル）
INSERT INTO project_master.project_types (code, display_name, description, created_by)
VALUES
    ('development', '開発プロジェクト', 'ソフトウェア開発プロジェクト', 'system'),
    ('operation', '運用プロジェクト', 'システム運用・保守プロジェクト', 'system'),
    ('marketing', 'マーケティングプロジェクト', 'マーケティング・キャンペーンプロジェクト', 'system');
```

---

## 関連ドキュメント

- [server.md](server.md) -- Project Master サーバー設計（API・アーキテクチャ）
- [implementation.md](implementation.md) -- Rust 実装詳細
- [system-database設計](../_common/database.md) -- 共通データベース設計パターン
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
