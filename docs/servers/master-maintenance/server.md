# system-master-maintenance-server 設計

メタデータ駆動型マスタメンテナンスサーバー。テーブル定義・カラム定義・整合性ルール登録のみで CRUD 画面を自動生成する。

## 概要

| 機能 | 説明 |
| --- | --- |
| メタデータ駆動 CRUD | テーブル定義・カラム定義から動的に CRUD API と UI を自動生成 |
| 整合性チェックエンジン | テーブル間の整合性ルールをマスタデータとして管理し、CRUD 操作時に自動評価 |
| 動的フォーム生成 | カラム定義の型・制約・表示設定から React フォームを自動生成 |
| 監査証跡 | 全変更を before/after JSONB 形式で自動記録 |
| 一括操作 | CSV/Excel によるインポート・エクスポート |
| テーブル単位 RBAC | テーブル・カラム単位のアクセス制御 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | 技術 |
| --- | --- |
| ルールエンジン | zen-engine (ZEN Engine) |
| フロントエンド | React 18 + Refine v4 + Ant Design v5 |
| 状態管理 | Refine 内蔵 (@refinedev/core) + React Query |
| フォーム生成 | JSON Schema → Ant Design Form コンポーネント |

### 配置パス

配置: `regions/system/server/rust/master-maintenance/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

| 種別 | パス |
| --- | --- |
| React クライアント | `regions/system/client/react/master-maintenance/` |
| Proto 定義 | `api/proto/k1s0/system/mastermaintenance/v1/master_maintenance.proto` |
| DB マイグレーション | `regions/system/database/master-maintenance-db/migrations/` |

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust (バックエンド) / TypeScript (フロントエンド) |
| メタデータ管理 | テーブル定義・カラム定義・整合性ルールをすべて PostgreSQL テーブルで管理 |
| 動的 CRUD | メタデータから SQL を動的生成（sqlx のプリペアドステートメント使用） |
| ルールエンジン | ZEN Engine (Rust ネイティブ) を組み込み、整合性ルールを JSON Decision Table で管理 |
| フロントエンド | Refine v4 の DataProvider + Ant Design で CRUD UI を自動生成 |
| DB | PostgreSQL 17 の `master_maintenance` スキーマ |
| Kafka | プロデューサー（`k1s0.system.mastermaintenance.data_changed.v1`） |
| 認証 | JWT による認可。テーブル単位の RBAC で `sys_operator` / `sys_admin` ロールが必要 |
| ポート | 8110（REST）/ 50051（gRPC） |

---

## アーキテクチャ全体図

<img src="diagrams/master-maintenance-architecture.svg" width="1200" />

### レイヤー構成

| レイヤー | 責務 | コンポーネント |
| --- | --- | --- |
| Client Layer | エンドユーザー向け UI | React Client (Refine + Ant Design), Flutter Client (optional) |
| API Layer | ビジネスロジック・ルール評価 | Master Maintenance Server (axum + tonic), GraphQL Gateway (既存), Auth Server (JWT) |
| Data Layer | 永続化・イベント配信 | PostgreSQL 17, Kafka |

---

## メタデータ駆動フロー

<img src="diagrams/master-maintenance-metadata-flow.svg" width="1400" />

### 開発者のオンボーディングワークフロー

開発者がマスタメンテナンス対象テーブルを追加する手順：

```
1. テーブル定義登録
   POST /api/v1/tables
   → table_definitions にレコード追加

2. カラム定義登録
   POST /api/v1/tables/{name}/columns (バッチ)
   → column_definitions にレコード追加

3. テーブル間関係定義（任意）
   POST /api/v1/relationships
   → table_relationships にレコード追加

4. 整合性ルール定義（任意）
   POST /api/v1/rules
   → consistency_rules + rule_conditions にレコード追加

5. 表示設定カスタマイズ（任意）
   POST /api/v1/tables/{name}/display-configs
   → display_configs にレコード追加

→ 以上でエンドユーザー向け CRUD 画面が自動生成される
→ コード変更・デプロイ不要
```

### メタデータ → UI 自動生成の内部フロー

| ステップ | エンジンコンポーネント | 処理内容 |
| --- | --- | --- |
| 1 | Metadata Reader | `table_definitions` + `column_definitions` をロード |
| 2 | JSON Schema Generator | カラム定義から JSON Schema を生成（フォーム定義） |
| 3 | Dynamic Query Builder | メタデータから SELECT/INSERT/UPDATE/DELETE SQL を動的構築 |
| 4 | ZEN Rule Engine | `consistency_rules` をロードし Decision Table を構築 |
| 5 | Response Composer | クエリ結果 + スキーマ + ルール結果を統合してレスポンス生成 |

---

## データベース設計

### ER 図

<img src="diagrams/master-maintenance-er.svg" width="1400" />

### スキーマ: `master_maintenance`

#### table_definitions（テーブル定義）

管理対象テーブルのメタデータ。1レコード = 1テーブル。

```sql
CREATE TABLE master_maintenance.table_definitions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL UNIQUE,
    schema_name     VARCHAR(100) NOT NULL,
    database_name   VARCHAR(100) NOT NULL DEFAULT 'default',
    display_name    VARCHAR(255) NOT NULL,
    description     TEXT,
    category        VARCHAR(100),
    is_active       BOOLEAN NOT NULL DEFAULT true,
    allow_create    BOOLEAN NOT NULL DEFAULT true,
    allow_update    BOOLEAN NOT NULL DEFAULT true,
    allow_delete    BOOLEAN NOT NULL DEFAULT false,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_by      VARCHAR(255) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_table_definitions_category ON master_maintenance.table_definitions(category);
CREATE INDEX idx_table_definitions_active ON master_maintenance.table_definitions(is_active);
```

#### column_definitions（カラム定義）

テーブル内の各カラムのメタデータ。型・制約・表示設定を保持。

```sql
CREATE TABLE master_maintenance.column_definitions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id            UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    column_name         VARCHAR(255) NOT NULL,
    display_name        VARCHAR(255) NOT NULL,
    data_type           VARCHAR(50) NOT NULL,
    is_primary_key      BOOLEAN NOT NULL DEFAULT false,
    is_nullable         BOOLEAN NOT NULL DEFAULT true,
    is_unique           BOOLEAN NOT NULL DEFAULT false,
    default_value       TEXT,
    max_length          INTEGER,
    min_value           NUMERIC,
    max_value           NUMERIC,
    regex_pattern       TEXT,
    display_order       INTEGER NOT NULL DEFAULT 0,
    is_searchable       BOOLEAN NOT NULL DEFAULT false,
    is_sortable         BOOLEAN NOT NULL DEFAULT true,
    is_filterable       BOOLEAN NOT NULL DEFAULT false,
    is_visible_in_list  BOOLEAN NOT NULL DEFAULT true,
    is_visible_in_form  BOOLEAN NOT NULL DEFAULT true,
    is_readonly         BOOLEAN NOT NULL DEFAULT false,
    input_type          VARCHAR(50) NOT NULL DEFAULT 'text',
    select_options      JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(table_id, column_name)
);

COMMENT ON COLUMN master_maintenance.column_definitions.data_type IS
    'text | integer | decimal | boolean | date | datetime | uuid | jsonb';
COMMENT ON COLUMN master_maintenance.column_definitions.input_type IS
    'text | textarea | select | checkbox | date | number | file | json_editor';

CREATE INDEX idx_column_definitions_table ON master_maintenance.column_definitions(table_id);
```

#### table_relationships（テーブル間関係）

テーブル間の FK / 参照関係を定義。UI でのリレーション表示・ルックアップに使用。

```sql
CREATE TABLE master_maintenance.table_relationships (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_table_id     UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    source_column       VARCHAR(255) NOT NULL,
    target_table_id     UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    target_column       VARCHAR(255) NOT NULL,
    relationship_type   VARCHAR(20) NOT NULL,
    display_name        VARCHAR(255),
    is_cascade_delete   BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_relationship_type CHECK (relationship_type IN ('one_to_one', 'one_to_many', 'many_to_many'))
);

CREATE INDEX idx_relationships_source ON master_maintenance.table_relationships(source_table_id);
CREATE INDEX idx_relationships_target ON master_maintenance.table_relationships(target_table_id);
```

#### consistency_rules（整合性ルール）

整合性チェックルールの定義。ルール自体がマスタデータとして CRUD 管理される。

```sql
CREATE TABLE master_maintenance.consistency_rules (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                    VARCHAR(255) NOT NULL,
    description             TEXT,
    rule_type               VARCHAR(50) NOT NULL,
    severity                VARCHAR(20) NOT NULL DEFAULT 'error',
    is_active               BOOLEAN NOT NULL DEFAULT true,
    source_table_id         UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    evaluation_timing       VARCHAR(30) NOT NULL DEFAULT 'before_save',
    error_message_template  TEXT NOT NULL,
    zen_rule_json           JSONB,
    created_by              VARCHAR(255) NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_rule_type CHECK (rule_type IN ('cross_table', 'range', 'uniqueness', 'conditional', 'custom')),
    CONSTRAINT chk_severity CHECK (severity IN ('error', 'warning', 'info')),
    CONSTRAINT chk_evaluation_timing CHECK (evaluation_timing IN ('before_save', 'after_save', 'on_demand', 'scheduled'))
);

CREATE INDEX idx_rules_source_table ON master_maintenance.consistency_rules(source_table_id);
CREATE INDEX idx_rules_active ON master_maintenance.consistency_rules(is_active);
CREATE INDEX idx_rules_timing ON master_maintenance.consistency_rules(evaluation_timing);
```

#### rule_conditions（ルール条件）

各整合性ルールの具体的な条件式。複数条件を AND/OR で合成可能。

```sql
CREATE TABLE master_maintenance.rule_conditions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id             UUID NOT NULL REFERENCES master_maintenance.consistency_rules(id) ON DELETE CASCADE,
    condition_order     INTEGER NOT NULL,
    left_table_id       UUID NOT NULL REFERENCES master_maintenance.table_definitions(id),
    left_column         VARCHAR(255) NOT NULL,
    operator            VARCHAR(20) NOT NULL,
    right_table_id      UUID REFERENCES master_maintenance.table_definitions(id),
    right_column        VARCHAR(255),
    right_value         TEXT,
    logical_connector   VARCHAR(5) DEFAULT 'AND',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_operator CHECK (operator IN ('eq', 'neq', 'gt', 'gte', 'lt', 'lte', 'in', 'not_in', 'exists', 'not_exists', 'regex', 'between')),
    CONSTRAINT chk_logical_connector CHECK (logical_connector IN ('AND', 'OR')),
    CONSTRAINT chk_right_side CHECK (right_table_id IS NOT NULL OR right_value IS NOT NULL)
);

CREATE INDEX idx_conditions_rule ON master_maintenance.rule_conditions(rule_id);
```

#### display_configs（表示設定）

テーブルごとの画面レイアウト・カスタマイズ設定。

```sql
CREATE TABLE master_maintenance.display_configs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id        UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    config_type     VARCHAR(50) NOT NULL,
    config_json     JSONB NOT NULL,
    is_default      BOOLEAN NOT NULL DEFAULT false,
    created_by      VARCHAR(255) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_config_type CHECK (config_type IN ('list_view', 'form_view', 'detail_view'))
);

CREATE INDEX idx_display_configs_table ON master_maintenance.display_configs(table_id);
```

**display_configs.config_json の構造例（list_view）：**

```json
{
  "columns": [
    { "column_name": "name", "width": 200, "fixed": "left" },
    { "column_name": "status", "width": 100, "render": "tag" },
    { "column_name": "amount", "width": 120, "render": "currency" }
  ],
  "default_sort": { "column": "created_at", "order": "desc" },
  "row_actions": ["edit", "delete", "duplicate"],
  "bulk_actions": ["delete", "export"],
  "page_size": 20,
  "grouping": { "column": "category", "collapsed": false }
}
```

#### change_logs（変更監査ログ）

全 CRUD 操作の before/after を記録。

```sql
CREATE TABLE master_maintenance.change_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_table    VARCHAR(255) NOT NULL,
    target_record_id TEXT NOT NULL,
    operation       VARCHAR(10) NOT NULL,
    before_data     JSONB,
    after_data      JSONB,
    changed_columns TEXT[],
    changed_by      VARCHAR(255) NOT NULL,
    change_reason   TEXT,
    trace_id        VARCHAR(255),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_operation CHECK (operation IN ('INSERT', 'UPDATE', 'DELETE'))
);

CREATE INDEX idx_change_logs_table ON master_maintenance.change_logs(target_table);
CREATE INDEX idx_change_logs_record ON master_maintenance.change_logs(target_table, target_record_id);
CREATE INDEX idx_change_logs_created ON master_maintenance.change_logs(created_at);
CREATE INDEX idx_change_logs_user ON master_maintenance.change_logs(changed_by);
```

#### import_jobs（インポートジョブ）

CSV/Excel 一括インポートのジョブ管理。

```sql
CREATE TABLE master_maintenance.import_jobs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id        UUID NOT NULL REFERENCES master_maintenance.table_definitions(id),
    file_name       VARCHAR(500) NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',
    total_rows      INTEGER NOT NULL DEFAULT 0,
    processed_rows  INTEGER NOT NULL DEFAULT 0,
    error_rows      INTEGER NOT NULL DEFAULT 0,
    error_details   JSONB,
    started_by      VARCHAR(255) NOT NULL,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ,
    CONSTRAINT chk_import_status CHECK (status IN ('pending', 'processing', 'completed', 'failed'))
);

CREATE INDEX idx_import_jobs_table ON master_maintenance.import_jobs(table_id);
CREATE INDEX idx_import_jobs_status ON master_maintenance.import_jobs(status);
```

---

## 整合性チェックエンジン

<img src="diagrams/master-maintenance-rule-engine.svg" width="1200" />

### ルールの種類

| rule_type | 説明 | 評価方法 |
| --- | --- | --- |
| `cross_table` | テーブル A のカラム値がテーブル B に存在することを検証 | `EXISTS` サブクエリ |
| `range` | 値が指定範囲内（min/max）であることを検証 | 値比較 |
| `uniqueness` | 指定カラムの組み合わせがユニークであることを検証 | `COUNT` クエリ |
| `conditional` | 条件付きバリデーション（IF-THEN） | 条件評価 + ルール適用 |
| `custom` | ZEN Engine による複雑なビジネスルール | Decision Table 評価 |

### ルール評価フロー

```
1. CRUD 操作トリガー
   ↓
2. source_table_id でルールをロード
   ↓
3. evaluation_timing でフィルタ
   ├── before_save: 保存前に評価（失敗時はブロック）
   ├── after_save:  保存後に評価（失敗時はログ + 通知）
   ├── on_demand:   手動実行のみ
   └── scheduled:   Scheduler から定期実行
   ↓
4. rule_type ごとに評価エンジンにディスパッチ
   ↓
5. 結果を集約
   ├── error   → 操作をブロック + ValidationError 返却
   ├── warning → 操作は許可 + 警告をレスポンスに含める
   └── info    → ログ記録のみ
```

### ZEN Engine 統合

`custom` タイプのルールでは ZEN Engine (gorules.io) の Decision Table を使用する。ルール定義は `consistency_rules.zen_rule_json` に JSON 形式で格納される。

**ZEN Decision Table の例（部門コードの整合性チェック）：**

```json
{
  "nodes": [
    {
      "id": "input",
      "type": "inputNode",
      "content": {
        "fields": [
          { "field": "department_code", "type": "string" },
          { "field": "employee_count", "type": "number" },
          { "field": "budget", "type": "number" }
        ]
      }
    },
    {
      "id": "dt1",
      "type": "decisionTableNode",
      "content": {
        "rules": [
          {
            "department_code": "== 'SALES'",
            "employee_count": ">= 5",
            "budget": ">= 1000000",
            "_result": "pass"
          },
          {
            "department_code": "== 'SALES'",
            "employee_count": "< 5",
            "_result": "fail",
            "_message": "営業部門は最低5名必要です"
          }
        ]
      }
    },
    {
      "id": "output",
      "type": "outputNode"
    }
  ],
  "edges": [
    { "sourceId": "input", "targetId": "dt1" },
    { "sourceId": "dt1", "targetId": "output" }
  ]
}
```

**Rust 側での ZEN Engine 呼び出し：**

```rust
use zen_engine::DecisionEngine;
use serde_json::Value;

pub async fn evaluate_custom_rule(
    rule: &ConsistencyRule,
    record_data: &Value,
) -> Result<RuleResult, Error> {
    let engine = DecisionEngine::default();
    let decision = engine.create_decision(
        rule.zen_rule_json.as_ref().unwrap().clone().into()
    )?;

    let result = decision.evaluate(record_data).await?;

    match result.get("_result").and_then(|v| v.as_str()) {
        Some("pass") => Ok(RuleResult::pass()),
        Some("fail") => {
            let message = result.get("_message")
                .and_then(|v| v.as_str())
                .unwrap_or(&rule.error_message_template);
            Ok(RuleResult::fail(message.to_string()))
        }
        _ => Ok(RuleResult::pass()),
    }
}
```

### ルール自体の CRUD 管理

整合性ルールはそれ自体がマスタデータであり、REST API / 管理画面から CRUD 管理できる。

| 操作 | API | 説明 |
| --- | --- | --- |
| ルール一覧 | `GET /api/v1/rules` | フィルタ: table, type, severity, timing |
| ルール作成 | `POST /api/v1/rules` | ルール + 条件を一括登録 |
| ルール更新 | `PUT /api/v1/rules/{id}` | ルール定義の変更（即時反映） |
| ルール削除 | `DELETE /api/v1/rules/{id}` | `sys_admin` のみ |
| ルール実行（オンデマンド） | `POST /api/v1/rules/{id}/execute` | 指定テーブルの全レコードに対してルール評価 |
| 一括チェック | `POST /api/v1/rules/check` | 指定テーブルの全ルールを一括評価 |

---

## API 設計

### CRUD 操作シーケンス

<img src="diagrams/master-maintenance-crud-sequence.svg" width="1400" />

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_MM_` とする。

#### テーブル定義管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/tables` | 登録済みテーブル定義一覧 | `sys_auditor` 以上 |
| POST | `/api/v1/tables` | テーブル定義登録 | `sys_operator` 以上 |
| GET | `/api/v1/tables/{name}` | テーブル定義取得（カラム定義含む） | `sys_auditor` 以上 |
| PUT | `/api/v1/tables/{name}` | テーブル定義更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/tables/{name}` | テーブル定義削除 | `sys_admin` のみ |
| GET | `/api/v1/tables/{name}/schema` | JSON Schema 形式でフォーム定義を取得 | `sys_auditor` 以上 |

#### カラム定義管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/tables/{name}/columns` | カラム定義一覧 | `sys_auditor` 以上 |
| POST | `/api/v1/tables/{name}/columns` | カラム定義一括登録 | `sys_operator` 以上 |
| PUT | `/api/v1/tables/{name}/columns/{column}` | カラム定義更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/tables/{name}/columns/{column}` | カラム定義削除 | `sys_admin` のみ |

#### データ CRUD（動的）

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/tables/{name}/records` | レコード一覧（ページネーション・フィルタ・ソート） | テーブル設定による |
| POST | `/api/v1/tables/{name}/records` | レコード作成 | テーブル設定 `allow_create` |
| GET | `/api/v1/tables/{name}/records/{id}` | レコード取得 | テーブル設定による |
| PUT | `/api/v1/tables/{name}/records/{id}` | レコード更新 | テーブル設定 `allow_update` |
| DELETE | `/api/v1/tables/{name}/records/{id}` | レコード削除 | テーブル設定 `allow_delete` |

#### レコード一覧クエリパラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1ページあたり件数（最大100） |
| `sort` | string | No | - | ソートカラム（例: `name:asc,created_at:desc`） |
| `filter` | string | No | - | フィルタ条件（例: `status:eq:active,amount:gte:1000`） |
| `search` | string | No | - | `is_searchable` カラムに対する全文検索 |
| `columns` | string | No | - | 取得カラム指定（例: `id,name,status`） |

**レスポンス（200 OK）**

```json
{
  "records": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "サンプル部門",
      "status": "active",
      "created_at": "2026-01-15T09:00:00Z"
    }
  ],
  "total": 150,
  "page": 1,
  "page_size": 20,
  "metadata": {
    "table_name": "departments",
    "display_name": "部門マスタ",
    "allow_create": true,
    "allow_update": true,
    "allow_delete": false
  }
}
```

#### テーブル間関係管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/relationships` | 関係定義一覧 | `sys_auditor` 以上 |
| POST | `/api/v1/relationships` | 関係定義登録 | `sys_operator` 以上 |
| PUT | `/api/v1/relationships/{id}` | 関係定義更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/relationships/{id}` | 関係定義削除 | `sys_admin` のみ |
| GET | `/api/v1/tables/{name}/related-records/{id}` | 関連レコード取得 | テーブル設定による |

#### 整合性ルール管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/rules` | ルール一覧 | `sys_auditor` 以上 |
| POST | `/api/v1/rules` | ルール作成 | `sys_operator` 以上 |
| GET | `/api/v1/rules/{id}` | ルール詳細取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/rules/{id}` | ルール更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/rules/{id}` | ルール削除 | `sys_admin` のみ |
| POST | `/api/v1/rules/{id}/execute` | ルール単体実行 | `sys_operator` 以上 |
| POST | `/api/v1/rules/check` | テーブルの全ルール一括チェック | `sys_operator` 以上 |

#### インポート・エクスポート

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/tables/{name}/import` | CSV/Excel インポート | `sys_operator` 以上 |
| GET | `/api/v1/tables/{name}/export` | CSV/Excel エクスポート | `sys_auditor` 以上 |
| GET | `/api/v1/import-jobs/{id}` | インポートジョブ状態取得 | `sys_auditor` 以上 |

#### 監査ログ

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/tables/{name}/audit-logs` | テーブル別変更履歴 | `sys_auditor` 以上 |
| GET | `/api/v1/tables/{name}/records/{id}/audit-logs` | レコード別変更履歴 | `sys_auditor` 以上 |

#### ヘルスチェック

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

### gRPC サービス定義

`api/proto/k1s0/system/mastermaintenance/v1/master_maintenance.proto`

```protobuf
syntax = "proto3";

package k1s0.system.mastermaintenance.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

service MasterMaintenanceService {
  // テーブル定義
  rpc GetTableDefinition(GetTableDefinitionRequest) returns (TableDefinitionResponse);
  rpc ListTableDefinitions(ListTableDefinitionsRequest) returns (ListTableDefinitionsResponse);

  // データ CRUD
  rpc GetRecord(GetRecordRequest) returns (RecordResponse);
  rpc ListRecords(ListRecordsRequest) returns (ListRecordsResponse);
  rpc CreateRecord(CreateRecordRequest) returns (RecordResponse);
  rpc UpdateRecord(UpdateRecordRequest) returns (RecordResponse);
  rpc DeleteRecord(DeleteRecordRequest) returns (DeleteRecordResponse);

  // 整合性チェック
  rpc CheckConsistency(CheckConsistencyRequest) returns (CheckConsistencyResponse);

  // JSON Schema
  rpc GetTableSchema(GetTableSchemaRequest) returns (TableSchemaResponse);
}

message GetTableDefinitionRequest {
  string table_name = 1;
}

message TableDefinitionResponse {
  string id = 1;
  string name = 2;
  string schema_name = 3;
  string display_name = 4;
  string description = 5;
  bool allow_create = 6;
  bool allow_update = 7;
  bool allow_delete = 8;
  repeated ColumnDefinition columns = 9;
  repeated TableRelationship relationships = 10;
}

message ColumnDefinition {
  string column_name = 1;
  string display_name = 2;
  string data_type = 3;
  bool is_primary_key = 4;
  bool is_nullable = 5;
  bool is_searchable = 6;
  bool is_sortable = 7;
  bool is_filterable = 8;
  bool is_visible_in_list = 9;
  bool is_visible_in_form = 10;
  bool is_readonly = 11;
  string input_type = 12;
  int32 display_order = 13;
}

message TableRelationship {
  string source_column = 1;
  string target_table = 2;
  string target_column = 3;
  string relationship_type = 4;
  string display_name = 5;
}

message ListTableDefinitionsRequest {
  string category = 1;
  bool active_only = 2;
  int32 page = 3;
  int32 page_size = 4;
}

message ListTableDefinitionsResponse {
  repeated TableDefinitionResponse tables = 1;
  int32 total = 2;
}

message GetRecordRequest {
  string table_name = 1;
  string record_id = 2;
}

message RecordResponse {
  google.protobuf.Struct data = 1;
  repeated ValidationWarning warnings = 2;
}

message ListRecordsRequest {
  string table_name = 1;
  int32 page = 2;
  int32 page_size = 3;
  string sort = 4;
  string filter = 5;
  string search = 6;
}

message ListRecordsResponse {
  repeated google.protobuf.Struct records = 1;
  int32 total = 2;
}

message CreateRecordRequest {
  string table_name = 1;
  google.protobuf.Struct data = 2;
}

message UpdateRecordRequest {
  string table_name = 1;
  string record_id = 2;
  google.protobuf.Struct data = 3;
}

message DeleteRecordRequest {
  string table_name = 1;
  string record_id = 2;
}

message DeleteRecordResponse {
  bool success = 1;
}

message CheckConsistencyRequest {
  string table_name = 1;
  repeated string rule_ids = 2;
}

message CheckConsistencyResponse {
  repeated ConsistencyResult results = 1;
  int32 total_checked = 2;
  int32 error_count = 3;
  int32 warning_count = 4;
}

message ConsistencyResult {
  string rule_id = 1;
  string rule_name = 2;
  string severity = 3;
  bool passed = 4;
  string message = 5;
  repeated string affected_record_ids = 6;
}

message ValidationWarning {
  string rule_name = 1;
  string message = 2;
  string severity = 3;
}

message GetTableSchemaRequest {
  string table_name = 1;
}

message TableSchemaResponse {
  string json_schema = 1;
}
```

### Kafka イベント

**トピック:** `k1s0.system.mastermaintenance.data_changed.v1`

```json
{
  "event_id": "uuid",
  "event_type": "record.updated",
  "table_name": "departments",
  "record_id": "uuid",
  "operation": "UPDATE",
  "before": { "name": "旧部門名", "status": "active" },
  "after": { "name": "新部門名", "status": "active" },
  "changed_columns": ["name"],
  "changed_by": "user-uuid",
  "trace_id": "trace-uuid",
  "timestamp": "2026-01-15T09:00:00Z"
}
```

---

## バックエンド設計 (Rust)

### ディレクトリ構成

```
regions/system/server/rust/master-maintenance/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── lib.rs                           # ライブラリクレートルート
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── table_definition.rs      # TableDefinition エンティティ
│   │   │   ├── column_definition.rs     # ColumnDefinition エンティティ
│   │   │   ├── table_relationship.rs    # TableRelationship エンティティ
│   │   │   ├── consistency_rule.rs      # ConsistencyRule エンティティ
│   │   │   ├── rule_condition.rs        # RuleCondition エンティティ
│   │   │   ├── display_config.rs        # DisplayConfig エンティティ
│   │   │   ├── change_log.rs            # ChangeLog エンティティ
│   │   │   └── import_job.rs            # ImportJob エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── table_definition_repository.rs
│   │   │   ├── column_definition_repository.rs
│   │   │   ├── consistency_rule_repository.rs
│   │   │   ├── change_log_repository.rs
│   │   │   └── dynamic_record_repository.rs  # 動的 CRUD
│   │   ├── service/
│   │   │   ├── mod.rs
│   │   │   ├── metadata_service.rs      # メタデータ管理ロジック
│   │   │   ├── rule_engine_service.rs   # 整合性チェックエンジン
│   │   │   ├── query_builder_service.rs # 動的 SQL 生成
│   │   │   └── schema_generator_service.rs  # JSON Schema 生成
│   │   └── value_object/
│   │       ├── mod.rs
│   │       ├── data_type.rs             # DataType 列挙
│   │       ├── operator.rs              # Operator 列挙
│   │       └── rule_result.rs           # RuleResult 値オブジェクト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── manage_table_definitions.rs
│   │   ├── manage_column_definitions.rs
│   │   ├── crud_records.rs              # 動的 CRUD ユースケース
│   │   ├── check_consistency.rs         # 整合性チェック
│   │   ├── manage_rules.rs
│   │   ├── import_records.rs
│   │   ├── export_records.rs
│   │   └── get_audit_logs.rs
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── table_handler.rs         # テーブル定義 REST ハンドラー
│   │   │   ├── record_handler.rs        # データ CRUD REST ハンドラー
│   │   │   ├── rule_handler.rs          # ルール管理 REST ハンドラー
│   │   │   ├── import_export_handler.rs # インポート・エクスポート
│   │   │   ├── audit_handler.rs         # 監査ログ
│   │   │   ├── grpc_handler.rs          # tonic gRPC ハンドラー
│   │   │   └── error.rs                 # エラーレスポンス
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── response.rs
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── table_rbac.rs            # テーブル単位 RBAC
│   └── infrastructure/
│       ├── mod.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── app_config.rs
│       ├── persistence/
│       │   ├── mod.rs
│       │   ├── db.rs
│       │   ├── table_definition_repo_impl.rs
│       │   ├── column_definition_repo_impl.rs
│       │   ├── consistency_rule_repo_impl.rs
│       │   ├── change_log_repo_impl.rs
│       │   └── dynamic_record_repo_impl.rs
│       ├── rule_engine/
│       │   ├── mod.rs
│       │   ├── zen_engine_adapter.rs    # ZEN Engine ラッパー
│       │   ├── cross_table_evaluator.rs
│       │   ├── range_evaluator.rs
│       │   ├── uniqueness_evaluator.rs
│       │   └── conditional_evaluator.rs
│       └── messaging/
│           ├── mod.rs
│           └── kafka_producer.rs
├── migrations/
│   ├── 001_create_schema.sql
│   ├── 002_create_table_definitions.sql
│   ├── 003_create_column_definitions.sql
│   ├── 004_create_table_relationships.sql
│   ├── 005_create_consistency_rules.sql
│   ├── 006_create_rule_conditions.sql
│   ├── 007_create_display_configs.sql
│   ├── 008_create_change_logs.sql
│   └── 009_create_import_jobs.sql
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── build.rs
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
└── README.md
```

---

## 設定フィールド

| カテゴリ | フィールド | 説明 |
| --- | --- | --- |
| server | `rest_port` / `grpc_port` / `environment` | REST 8110 / gRPC 50051 |
| database | `host` / `port` / `name` / `schema` / `max_connections` | PostgreSQL `master_maintenance` スキーマ |
| kafka | `brokers` / `topic` | `k1s0.system.mastermaintenance.data_changed.v1` |
| auth | `jwks_url` / `issuer` / `audience` | JWT 認証設定 |
| rule_engine | `max_rules_per_table` / `evaluation_timeout_ms` / `cache_ttl_seconds` | ルールエンジン設定 |
| import | `max_file_size_mb` / `max_rows_per_import` / `batch_size` | インポート制限 |

---

## デプロイ

[system-server-deploy.md](../_common/deploy.md) に従い Helm Chart でデプロイする。

| パラメータ | 値 |
| --- | --- |
| replicas | 2 |
| resources.requests.cpu / memory | 200m / 256Mi |
| resources.limits.cpu / memory | 500m / 512Mi |
| readinessProbe.path | /readyz |
| livenessProbe.path | /healthz |

### Docker Compose (開発環境)

```yaml
master-maintenance-server:
  build:
    context: ./regions/system/server/rust/master-maintenance
    dockerfile: Dockerfile
  ports:
    - "8110:8110"
    - "9098:50051"
  environment:
    - DATABASE_URL=postgresql://k1s0:k1s0@postgres:5432/k1s0
    - KAFKA_BROKERS=kafka:9092
    - AUTH_JWKS_URL=http://keycloak:8080/realms/k1s0/protocol/openid-connect/certs
  depends_on:
    - postgres
    - kafka
    - keycloak
```

---

## 設計思想：メタデータ駆動アーキテクチャ

本システムは **Metadata-Driven Architecture** を採用する。すべての画面・フォーム・バリデーション・ビジネスルールをメタデータ（テーブルデータ）として定義し、ランタイムエンジンが動的に解釈・実行する。

**従来のアプローチとの比較：**

| 項目 | 従来（テーブルごとに個別実装） | 本システム（メタデータ駆動） |
| --- | --- | --- |
| 新規テーブル追加 | API・画面・バリデーションを個別開発 | メタデータ登録のみ（コード変更なし） |
| 整合性ルール変更 | コード修正 → テスト → デプロイ | ルールテーブルを UPDATE（即時反映） |
| 開発コスト | テーブル数に比例して増加 | 一定（メタデータ登録コストのみ） |
| 一貫性 | 開発者による差異が発生 | 全テーブルで統一された UX |

### 取り入れた最新事例・技術

| 技術 / 事例 | 活用箇所 | 参照 |
| --- | --- | --- |
| **Refine** (React meta-framework) | フロントエンド CRUD UI 自動生成 | refine.dev |
| **ZEN Engine** (gorules.io) | Rust ネイティブ ビジネスルールエンジン | gorules.io |
| **JSON Schema** | 動的フォーム定義の標準フォーマット | json-schema.org |
| **Evolutility** | モデル駆動 CRUD ビューの設計パターン | GitHub |
| **Rules Engine Pattern** | ルール自体をテーブルで管理するパターン | Medium |

---

## 動的 SQL 生成の安全性

メタデータから SQL を動的に生成する際、**SQL インジェクション防止**のために以下の制約を適用する。

```rust
/// 動的クエリビルダー
/// メタデータ定義に基づいて安全な SQL を生成する
pub struct DynamicQueryBuilder;

impl DynamicQueryBuilder {
    /// テーブル名・カラム名はメタデータ定義に存在するもののみ許可
    /// （ユーザー入力をそのまま SQL に埋め込まない）
    fn validate_identifier(name: &str, allowed: &[String]) -> Result<&str, Error> {
        if allowed.contains(&name.to_string()) {
            Ok(name)
        } else {
            Err(Error::InvalidIdentifier(name.to_string()))
        }
    }

    /// SELECT 文の生成
    pub fn build_select(
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        filters: &[Filter],
        sort: &[Sort],
        page: i32,
        page_size: i32,
    ) -> Result<(String, Vec<Value>), Error> {
        let allowed_columns: Vec<String> = columns.iter()
            .map(|c| c.column_name.clone())
            .collect();

        // テーブル名は table_definitions に登録済みのもののみ
        let qualified_table = format!(
            "\"{}\".\"{}\"",
            table_def.schema_name,
            table_def.name
        );

        let select_cols = columns.iter()
            .filter(|c| c.is_visible_in_list)
            .map(|c| format!("\"{}\"", c.column_name))
            .collect::<Vec<_>>()
            .join(", ");

        let mut sql = format!("SELECT {} FROM {}", select_cols, qualified_table);
        let mut params: Vec<Value> = Vec::new();
        let mut param_idx = 1;

        // WHERE 句（フィルタ）
        if !filters.is_empty() {
            let mut conditions = Vec::new();
            for filter in filters {
                Self::validate_identifier(&filter.column, &allowed_columns)?;
                conditions.push(format!(
                    "\"{}\" {} ${}",
                    filter.column, filter.operator.to_sql(), param_idx
                ));
                params.push(filter.value.clone());
                param_idx += 1;
            }
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }

        // ORDER BY
        if !sort.is_empty() {
            let sort_clauses: Vec<String> = sort.iter()
                .filter_map(|s| {
                    Self::validate_identifier(&s.column, &allowed_columns).ok()
                        .map(|col| format!("\"{}\" {}", col, s.direction.to_sql()))
                })
                .collect();
            if !sort_clauses.is_empty() {
                sql.push_str(&format!(" ORDER BY {}", sort_clauses.join(", ")));
            }
        }

        // LIMIT/OFFSET
        sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, (page - 1) * page_size));

        Ok((sql, params))
    }
}
```

---

## Cargo.toml

```toml
[package]
name = "k1s0-master-maintenance-server"
version = "0.1.0"
edition = "2021"

[lib]
name = "k1s0_master_maintenance_server"
path = "src/lib.rs"

[[bin]]
name = "k1s0-master-maintenance-server"
path = "src/main.rs"

[dependencies]
# Web framework
axum = { version = "0.7", features = ["macros", "multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate"] }

# Rule Engine
zen-engine = "0.20"

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"
regex = "1"

# CSV/Excel
csv = "1"
calamine = "0.26"

# Logging / Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Telemetry library
k1s0-telemetry = { path = "../../../library/rust/telemetry", features = ["full"] }

# Server common (error codes)
k1s0-server-common = { path = "../../../library/rust/server-common", features = ["axum"] }

# OpenAPI
utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

[build-dependencies]
tonic-build = "0.12"

[features]
db-tests = []

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
tower = { version = "0.5", features = ["util"] }
```

---

## フロントエンド設計 (React)

### コンポーネント構成

<img src="diagrams/master-maintenance-react-components.svg" width="1400" />

### 動的フォーム生成の仕組み

`FieldRenderer` がカラム定義の `data_type` + `input_type` から適切な Ant Design コンポーネントを解決する。

```typescript
// hooks/useFieldRenderer.ts
import { ColumnDefinition } from '../types/column';

type FieldConfig = {
  component: string;
  props: Record<string, unknown>;
};

export function resolveField(column: ColumnDefinition): FieldConfig {
  const base = {
    name: column.column_name,
    label: column.display_name,
    required: !column.is_nullable,
    disabled: column.is_readonly,
  };

  switch (column.input_type) {
    case 'text':
      return {
        component: 'Input',
        props: { ...base, maxLength: column.max_length },
      };
    case 'textarea':
      return {
        component: 'Input.TextArea',
        props: { ...base, rows: 4, maxLength: column.max_length },
      };
    case 'number':
      return {
        component: 'InputNumber',
        props: {
          ...base,
          min: column.min_value,
          max: column.max_value,
        },
      };
    case 'select':
      return {
        component: 'Select',
        props: {
          ...base,
          options: column.select_options,
        },
      };
    case 'checkbox':
      return { component: 'Checkbox', props: base };
    case 'date':
      return { component: 'DatePicker', props: base };
    case 'file':
      return { component: 'Upload', props: base };
    case 'json_editor':
      return { component: 'JSONEditor', props: base };
    default:
      return { component: 'Input', props: base };
  }
}
```

### Refine DataProvider 統合

```typescript
// providers/data-provider.ts
import { DataProvider } from '@refinedev/core';

export const masterMaintenanceDataProvider: DataProvider = {
  getList: async ({ resource, pagination, sorters, filters }) => {
    const { current, pageSize } = pagination ?? {};
    const params = new URLSearchParams();
    if (current) params.set('page', String(current));
    if (pageSize) params.set('page_size', String(pageSize));

    // resource = テーブル名（動的）
    const response = await fetch(
      `/api/v1/tables/${resource}/records?${params}`
    );
    const data = await response.json();

    return {
      data: data.records,
      total: data.total,
    };
  },

  getOne: async ({ resource, id }) => {
    const response = await fetch(
      `/api/v1/tables/${resource}/records/${id}`
    );
    return { data: await response.json() };
  },

  create: async ({ resource, variables }) => {
    const response = await fetch(`/api/v1/tables/${resource}/records`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(variables),
    });
    return { data: await response.json() };
  },

  update: async ({ resource, id, variables }) => {
    const response = await fetch(
      `/api/v1/tables/${resource}/records/${id}`,
      {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(variables),
      }
    );
    return { data: await response.json() };
  },

  deleteOne: async ({ resource, id }) => {
    await fetch(`/api/v1/tables/${resource}/records/${id}`, {
      method: 'DELETE',
    });
    return { data: { id } as any };
  },

  getApiUrl: () => '/api/v1',
};
```

---

## セキュリティ

### テーブル・カラム単位の RBAC

```
1. JWT から user_id + roles を取得
2. table_definitions の RBAC 設定を確認
   - allow_create / allow_update / allow_delete フラグ
   - テーブル単位の追加パーミッション
3. column_definitions の is_readonly フラグ
   - readonly カラムは UPDATE 時に除外
4. 操作が許可されていない場合は 403 Forbidden を返却
```

### 動的 SQL のセキュリティ対策

| 脅威 | 対策 |
| --- | --- |
| SQL インジェクション | テーブル名・カラム名はメタデータ定義に存在するもののみ許可。値はプリペアドステートメントのパラメータとしてバインド |
| 権限昇格 | テーブル単位の RBAC + JWT 検証。`sys_admin` のみ削除操作 |
| データ漏洩 | `is_visible_in_list` / `is_visible_in_form` で公開カラムを制御 |
| 大量データ取得 | `page_size` 上限 100。Export は非同期ジョブで実行 |

### 監査ログ

全 CRUD 操作で以下を自動記録：

- **who**: JWT から取得した user_id
- **what**: 対象テーブル・レコード・カラム
- **when**: タイムスタンプ
- **before/after**: JSONB 形式で変更前後のデータ
- **trace_id**: OpenTelemetry trace_id（分散トレーシング連携）

---

## 開発者ガイド

### 新規テーブル登録手順

**例：「部門マスタ」テーブルを登録する場合**

#### Step 1: テーブル定義登録

```bash
curl -X POST http://localhost:8110/api/v1/tables \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "departments",
    "schema_name": "business",
    "display_name": "部門マスタ",
    "description": "部門の一覧管理",
    "category": "組織",
    "allow_create": true,
    "allow_update": true,
    "allow_delete": false
  }'
```

#### Step 2: カラム定義一括登録

```bash
curl -X POST http://localhost:8110/api/v1/tables/departments/columns \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "columns": [
      {
        "column_name": "id",
        "display_name": "ID",
        "data_type": "uuid",
        "is_primary_key": true,
        "is_visible_in_form": false,
        "display_order": 0
      },
      {
        "column_name": "code",
        "display_name": "部門コード",
        "data_type": "text",
        "max_length": 10,
        "is_unique": true,
        "is_searchable": true,
        "is_filterable": true,
        "input_type": "text",
        "display_order": 1
      },
      {
        "column_name": "name",
        "display_name": "部門名",
        "data_type": "text",
        "max_length": 100,
        "is_nullable": false,
        "is_searchable": true,
        "input_type": "text",
        "display_order": 2
      },
      {
        "column_name": "parent_id",
        "display_name": "上位部門",
        "data_type": "uuid",
        "is_nullable": true,
        "input_type": "select",
        "display_order": 3
      },
      {
        "column_name": "is_active",
        "display_name": "有効",
        "data_type": "boolean",
        "default_value": "true",
        "is_filterable": true,
        "input_type": "checkbox",
        "display_order": 4
      }
    ]
  }'
```

#### Step 3: テーブル間関係定義

```bash
curl -X POST http://localhost:8110/api/v1/relationships \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "source_table": "departments",
    "source_column": "parent_id",
    "target_table": "departments",
    "target_column": "id",
    "relationship_type": "many_to_one",
    "display_name": "上位部門"
  }'
```

#### Step 4: 整合性ルール定義

```bash
curl -X POST http://localhost:8110/api/v1/rules \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "dept_code_format",
    "description": "部門コードは英大文字3桁+数字3桁の形式",
    "rule_type": "conditional",
    "severity": "error",
    "source_table": "departments",
    "evaluation_timing": "before_save",
    "error_message_template": "部門コード「{code}」の形式が不正です（例: ABC001）",
    "conditions": [
      {
        "condition_order": 1,
        "left_column": "code",
        "operator": "regex",
        "right_value": "^[A-Z]{3}[0-9]{3}$"
      }
    ]
  }'
```

以上で完了。React 画面でエンドユーザーが部門マスタの CRUD 操作を開始できる。

### 整合性ルール定義パターン集

#### パターン 1: クロステーブル参照整合性

```json
{
  "name": "employee_dept_exists",
  "rule_type": "cross_table",
  "source_table": "employees",
  "conditions": [
    {
      "left_column": "department_id",
      "operator": "exists",
      "right_table": "departments",
      "right_column": "id"
    }
  ]
}
```

#### パターン 2: 範囲チェック

```json
{
  "name": "price_range",
  "rule_type": "range",
  "source_table": "products",
  "conditions": [
    {
      "left_column": "unit_price",
      "operator": "gte",
      "right_value": "0"
    },
    {
      "left_column": "unit_price",
      "operator": "lte",
      "right_value": "999999",
      "logical_connector": "AND"
    }
  ]
}
```

#### パターン 3: 条件付きユニーク

```json
{
  "name": "unique_position_per_dept",
  "rule_type": "uniqueness",
  "source_table": "employees",
  "conditions": [
    {
      "left_column": "department_id",
      "operator": "eq",
      "right_value": "{current.department_id}"
    },
    {
      "left_column": "position",
      "operator": "eq",
      "right_value": "{current.position}",
      "logical_connector": "AND"
    }
  ]
}
```

#### パターン 4: ZEN Engine カスタムルール

```json
{
  "name": "sales_target_staffing",
  "rule_type": "custom",
  "source_table": "departments",
  "zen_rule_json": {
    "nodes": [
      {
        "id": "input",
        "type": "inputNode",
        "content": {
          "fields": [
            { "field": "sales_target", "type": "number" },
            { "field": "headcount", "type": "number" }
          ]
        }
      },
      {
        "id": "dt1",
        "type": "decisionTableNode",
        "content": {
          "rules": [
            { "sales_target": "> 10000000", "headcount": "< 3", "_result": "fail", "_message": "売上目標1千万超は最低3名必要" },
            { "sales_target": "> 50000000", "headcount": "< 10", "_result": "fail", "_message": "売上目標5千万超は最低10名必要" },
            { "sales_target": "", "headcount": "", "_result": "pass" }
          ]
        }
      },
      { "id": "output", "type": "outputNode" }
    ],
    "edges": [
      { "sourceId": "input", "targetId": "dt1" },
      { "sourceId": "dt1", "targetId": "output" }
    ]
  }
}
```
