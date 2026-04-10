# system-master-maintenance-server データベース設計

## スキーマ

スキーマ名: `master_maintenance`

```sql
CREATE SCHEMA IF NOT EXISTS master_maintenance;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| table_definitions | マスタテーブル定義（メタデータ） |
| column_definitions | テーブル内カラム定義 |
| table_relationships | テーブル間リレーション定義 |
| consistency_rules | 整合性ルール定義 |
| rule_conditions | ルール条件詳細 |
| display_configs | 画面表示設定 |
| change_logs | マスタ変更履歴 |
| import_jobs | CSV インポートジョブ |

---

## ER 図

```
table_definitions 1──* column_definitions
table_definitions 1──* table_relationships (source_table_id)
table_definitions 1──* table_relationships (target_table_id)
table_definitions 1──* consistency_rules
table_definitions 1──* display_configs
table_definitions 1──* import_jobs
consistency_rules 1──* rule_conditions
rule_conditions.left_table_id → table_definitions.id
rule_conditions.right_table_id → table_definitions.id
```

---

## テーブル定義

### table_definitions（テーブル定義）

マスタメンテナンス対象テーブルのメタデータを管理する。CRUD 操作の許可制御と表示順、ドメインスコープを持つ。

```sql
CREATE TABLE master_maintenance.table_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    schema_name VARCHAR(100) NOT NULL,
    database_name VARCHAR(100) NOT NULL DEFAULT 'default',
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    allow_create BOOLEAN NOT NULL DEFAULT TRUE,
    allow_update BOOLEAN NOT NULL DEFAULT TRUE,
    allow_delete BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    domain_scope VARCHAR(100) DEFAULT NULL,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_table_definitions_name_domain
    ON master_maintenance.table_definitions (name, COALESCE(domain_scope, '__system__'));
CREATE INDEX idx_table_definitions_category ON master_maintenance.table_definitions(category);
CREATE INDEX idx_table_definitions_active ON master_maintenance.table_definitions(is_active);
CREATE INDEX idx_table_definitions_domain_scope ON master_maintenance.table_definitions(domain_scope);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | UNIQUE(name, domain_scope), NOT NULL | テーブル名 |
| schema_name | VARCHAR(100) | NOT NULL | スキーマ名 |
| database_name | VARCHAR(100) | NOT NULL, DEFAULT 'default' | データベース名 |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| description | TEXT | | 説明 |
| category | VARCHAR(100) | | カテゴリ |
| is_active | BOOLEAN | NOT NULL, DEFAULT TRUE | 有効フラグ |
| allow_create | BOOLEAN | NOT NULL, DEFAULT TRUE | 作成許可 |
| allow_update | BOOLEAN | NOT NULL, DEFAULT TRUE | 更新許可 |
| allow_delete | BOOLEAN | NOT NULL, DEFAULT FALSE | 削除許可 |
| sort_order | INTEGER | NOT NULL, DEFAULT 0 | 表示順 |
| domain_scope | VARCHAR(100) | DEFAULT NULL | ドメインスコープ |
| created_by | VARCHAR(255) | NOT NULL | 作成者 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

---

### column_definitions（カラム定義）

テーブル内カラムの型・制約・表示設定を管理する。入力フォームの制御（input_type, select_options）も含む。

```sql
CREATE TABLE master_maintenance.column_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    column_name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    data_type VARCHAR(50) NOT NULL,
    is_primary_key BOOLEAN NOT NULL DEFAULT FALSE,
    is_nullable BOOLEAN NOT NULL DEFAULT TRUE,
    is_unique BOOLEAN NOT NULL DEFAULT FALSE,
    default_value TEXT,
    max_length INTEGER,
    min_value DOUBLE PRECISION,
    max_value DOUBLE PRECISION,
    regex_pattern TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_searchable BOOLEAN NOT NULL DEFAULT FALSE,
    is_sortable BOOLEAN NOT NULL DEFAULT TRUE,
    is_filterable BOOLEAN NOT NULL DEFAULT FALSE,
    is_visible_in_list BOOLEAN NOT NULL DEFAULT TRUE,
    is_visible_in_form BOOLEAN NOT NULL DEFAULT TRUE,
    is_readonly BOOLEAN NOT NULL DEFAULT FALSE,
    input_type VARCHAR(50) NOT NULL DEFAULT 'text',
    select_options JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_column_definitions_table_column UNIQUE (table_id, column_name)
);

CREATE INDEX idx_column_definitions_table ON master_maintenance.column_definitions(table_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| table_id | UUID | FK → table_definitions.id, NOT NULL | 所属テーブル |
| column_name | VARCHAR(255) | UNIQUE(table_id, column_name), NOT NULL | カラム名 |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| data_type | VARCHAR(50) | NOT NULL | データ型 |
| is_primary_key | BOOLEAN | NOT NULL, DEFAULT FALSE | 主キーフラグ |
| is_nullable | BOOLEAN | NOT NULL, DEFAULT TRUE | NULL 許可 |
| is_unique | BOOLEAN | NOT NULL, DEFAULT FALSE | ユニーク制約 |
| default_value | TEXT | | デフォルト値 |
| max_length | INTEGER | | 最大長 |
| min_value | DOUBLE PRECISION | | 最小値 |
| max_value | DOUBLE PRECISION | | 最大値 |
| regex_pattern | TEXT | | 正規表現パターン |
| display_order | INTEGER | NOT NULL, DEFAULT 0 | 表示順 |
| is_searchable | BOOLEAN | NOT NULL, DEFAULT FALSE | 検索可能フラグ |
| is_sortable | BOOLEAN | NOT NULL, DEFAULT TRUE | ソート可能フラグ |
| is_filterable | BOOLEAN | NOT NULL, DEFAULT FALSE | フィルタ可能フラグ |
| is_visible_in_list | BOOLEAN | NOT NULL, DEFAULT TRUE | 一覧表示フラグ |
| is_visible_in_form | BOOLEAN | NOT NULL, DEFAULT TRUE | フォーム表示フラグ |
| is_readonly | BOOLEAN | NOT NULL, DEFAULT FALSE | 読み取り専用フラグ |
| input_type | VARCHAR(50) | NOT NULL, DEFAULT 'text' | 入力タイプ |
| select_options | JSONB | | セレクトオプション |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

---

### table_relationships（テーブル間リレーション）

テーブル間の外部キー関係を定義する。

```sql
CREATE TABLE master_maintenance.table_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    source_column VARCHAR(255) NOT NULL,
    target_table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    target_column VARCHAR(255) NOT NULL,
    relationship_type VARCHAR(50) NOT NULL,
    display_name VARCHAR(255),
    is_cascade_delete BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_table_relationships_source ON master_maintenance.table_relationships(source_table_id);
CREATE INDEX idx_table_relationships_target ON master_maintenance.table_relationships(target_table_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| source_table_id | UUID | FK → table_definitions.id, NOT NULL | ソーステーブル |
| source_column | VARCHAR(255) | NOT NULL | ソースカラム |
| target_table_id | UUID | FK → table_definitions.id, NOT NULL | ターゲットテーブル |
| target_column | VARCHAR(255) | NOT NULL | ターゲットカラム |
| relationship_type | VARCHAR(50) | NOT NULL | リレーション種別 |
| display_name | VARCHAR(255) | | 表示名 |
| is_cascade_delete | BOOLEAN | NOT NULL, DEFAULT FALSE | カスケード削除フラグ |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |

---

### consistency_rules（整合性ルール）

マスタデータの整合性チェックルールを定義する。zen_rule_json で JSON ルールエンジン用の定義を保持する。

```sql
CREATE TABLE master_maintenance.consistency_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    rule_type VARCHAR(50) NOT NULL,
    severity VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    source_table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    evaluation_timing VARCHAR(50) NOT NULL,
    error_message_template TEXT NOT NULL,
    zen_rule_json JSONB,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_consistency_rules_source_table ON master_maintenance.consistency_rules(source_table_id);
CREATE INDEX idx_consistency_rules_active ON master_maintenance.consistency_rules(is_active);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | NOT NULL | ルール名 |
| description | TEXT | | 説明 |
| rule_type | VARCHAR(50) | NOT NULL | ルール種別 |
| severity | VARCHAR(50) | NOT NULL | 重大度 |
| is_active | BOOLEAN | NOT NULL, DEFAULT TRUE | 有効フラグ |
| source_table_id | UUID | FK → table_definitions.id, NOT NULL | 対象テーブル |
| evaluation_timing | VARCHAR(50) | NOT NULL | 評価タイミング |
| error_message_template | TEXT | NOT NULL | エラーメッセージテンプレート |
| zen_rule_json | JSONB | | JSON ルールエンジン定義 |
| created_by | VARCHAR(255) | NOT NULL | 作成者 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

---

### rule_conditions（ルール条件）

整合性ルールの個別条件を定義する。condition_order で評価順序を制御し、logical_connector で条件を結合する。

```sql
CREATE TABLE master_maintenance.rule_conditions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES master_maintenance.consistency_rules(id) ON DELETE CASCADE,
    condition_order INTEGER NOT NULL,
    left_table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    left_column VARCHAR(255) NOT NULL,
    operator VARCHAR(50) NOT NULL,
    right_table_id UUID REFERENCES master_maintenance.table_definitions(id) ON DELETE SET NULL,
    right_column VARCHAR(255),
    right_value TEXT,
    logical_connector VARCHAR(10),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_rule_conditions_rule ON master_maintenance.rule_conditions(rule_id, condition_order);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| rule_id | UUID | FK → consistency_rules.id, NOT NULL | 所属ルール |
| condition_order | INTEGER | NOT NULL | 条件評価順序 |
| left_table_id | UUID | FK → table_definitions.id, NOT NULL | 左辺テーブル |
| left_column | VARCHAR(255) | NOT NULL | 左辺カラム |
| operator | VARCHAR(50) | NOT NULL | 比較演算子 |
| right_table_id | UUID | FK → table_definitions.id | 右辺テーブル |
| right_column | VARCHAR(255) | | 右辺カラム |
| right_value | TEXT | | 右辺固定値 |
| logical_connector | VARCHAR(10) | | 論理接続子（AND/OR） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |

---

### display_configs（画面表示設定）

テーブルごとの画面表示設定（一覧表示・フォーム・フィルタ等）を JSON で管理する。

```sql
CREATE TABLE master_maintenance.display_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    config_type VARCHAR(50) NOT NULL,
    config_json JSONB NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_display_configs_table ON master_maintenance.display_configs(table_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| table_id | UUID | FK → table_definitions.id, NOT NULL | 対象テーブル |
| config_type | VARCHAR(50) | NOT NULL | 設定種別 |
| config_json | JSONB | NOT NULL | 表示設定 JSON |
| is_default | BOOLEAN | NOT NULL, DEFAULT FALSE | デフォルト設定フラグ |
| created_by | VARCHAR(255) | NOT NULL | 作成者 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 更新日時 |

---

### change_logs（変更履歴）

マスタデータの変更履歴を記録する。before/after の差分と変更カラム一覧を保持する。

```sql
CREATE TABLE master_maintenance.change_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_table VARCHAR(255) NOT NULL,
    target_record_id VARCHAR(255) NOT NULL,
    operation VARCHAR(50) NOT NULL,
    before_data JSONB,
    after_data JSONB,
    changed_columns TEXT[],
    changed_by VARCHAR(255) NOT NULL,
    change_reason TEXT,
    trace_id VARCHAR(255),
    domain_scope VARCHAR(100) DEFAULT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_change_logs_table ON master_maintenance.change_logs(target_table, created_at DESC);
CREATE INDEX idx_change_logs_record ON master_maintenance.change_logs(target_table, target_record_id, created_at DESC);
CREATE INDEX idx_change_logs_domain_scope ON master_maintenance.change_logs(domain_scope);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| target_table | VARCHAR(255) | NOT NULL | 対象テーブル名 |
| target_record_id | VARCHAR(255) | NOT NULL | 対象レコード ID |
| operation | VARCHAR(50) | NOT NULL | 操作種別（INSERT/UPDATE/DELETE） |
| before_data | JSONB | | 変更前データ |
| after_data | JSONB | | 変更後データ |
| changed_columns | TEXT[] | | 変更カラム一覧 |
| changed_by | VARCHAR(255) | NOT NULL | 変更者 |
| change_reason | TEXT | | 変更理由 |
| trace_id | VARCHAR(255) | | トレース ID |
| domain_scope | VARCHAR(100) | DEFAULT NULL | ドメインスコープ |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 記録日時 |

---

### import_jobs（インポートジョブ）

CSV 等のインポートジョブの進捗・結果を管理する。

```sql
CREATE TABLE master_maintenance.import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    total_rows INTEGER NOT NULL DEFAULT 0,
    processed_rows INTEGER NOT NULL DEFAULT 0,
    error_rows INTEGER NOT NULL DEFAULT 0,
    error_details JSONB,
    started_by VARCHAR(255) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_import_jobs_table ON master_maintenance.import_jobs(table_id, started_at DESC);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| table_id | UUID | FK → table_definitions.id, NOT NULL | 対象テーブル |
| file_name | VARCHAR(255) | NOT NULL | ファイル名 |
| status | VARCHAR(50) | NOT NULL | ステータス |
| total_rows | INTEGER | NOT NULL, DEFAULT 0 | 総行数 |
| processed_rows | INTEGER | NOT NULL, DEFAULT 0 | 処理済み行数 |
| error_rows | INTEGER | NOT NULL, DEFAULT 0 | エラー行数 |
| error_details | JSONB | | エラー詳細 |
| started_by | VARCHAR(255) | NOT NULL | 実行者 |
| started_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 開始日時 |
| completed_at | TIMESTAMPTZ | | 完了日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/master-maintenance-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `master_maintenance` スキーマ・pgcrypto 拡張作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_master_maintenance_tables.up.sql` | 全テーブル作成 |
| `002_create_master_maintenance_tables.down.sql` | テーブル削除 |
| `003_add_domain_scope.up.sql` | domain_scope カラム追加・ユニーク制約変更 |
| `003_add_domain_scope.down.sql` | domain_scope カラム削除 |
| `004_add_updated_at_trigger.up.sql` | updated_at 自動更新トリガー追加 |
| `005_add_table_rbac_roles.up.sql` | RBAC ロールカラム追加 |
| `006_add_tenant_id_rls.up.sql` | 全テーブルに `tenant_id TEXT NOT NULL` と RLS ポリシー（FORCE / AS RESTRICTIVE / WITH CHECK）を追加（CRITICAL-DB-001 対応） |
| `006_add_tenant_id_rls.down.sql` | `tenant_id` カラムと RLS ポリシー削除 |

---

## マルチテナント対応（CRITICAL-DB-001）

全テーブル（`table_definitions` / `column_definitions` / `table_relationships` / `consistency_rules` / `rule_conditions` / `display_configs` / `change_logs` / `import_jobs`）に `tenant_id TEXT NOT NULL` カラムと RLS ポリシーを追加（migration 006）。

```sql
ALTER TABLE master_maintenance.{table} ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.{table} FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON master_maintenance.{table}
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```
