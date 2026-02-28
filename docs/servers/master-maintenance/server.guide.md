# system-master-maintenance-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

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
| **Refine** (React meta-framework) | フロントエンド CRUD UI 自動生成 | [refine.dev](https://refine.dev/) |
| **ZEN Engine** (gorules.io) | Rust ネイティブ ビジネスルールエンジン | [gorules.io](https://gorules.io/) |
| **JSON Schema** | 動的フォーム定義の標準フォーマット | [json-schema.org](https://json-schema.org/) |
| **Evolutility** | モデル駆動 CRUD ビューの設計パターン | [GitHub](https://github.com/evoluteur/evolutility-ui-react) |
| **Rules Engine Pattern** | ルール自体をテーブルで管理するパターン | [Medium](https://medium.com/@herihermawan/the-ultimate-multifunctional-database-table-design-rules-engine-pattern-d55460f048c4) |

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

→ **以上で完了**。React 画面でエンドユーザーが部門マスタの CRUD 操作を開始できる。

### 整合性ルール定義パターン集

#### パターン 1: クロステーブル参照整合性

「社員テーブルの department_id が部門テーブルに存在すること」

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

「単価は 0 以上 999,999 以下」

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

「同一部門内で役職名がユニーク」

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

「売上目標に対する人員配置の妥当性チェック」

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

---

## デプロイ

### Docker Compose (開発環境)

```yaml
master-maintenance-server:
  build:
    context: ./regions/system/server/rust/master-maintenance
    dockerfile: Dockerfile
  ports:
    - "8110:8110"
    - "9098:9090"
  environment:
    - DATABASE_URL=postgresql://k1s0:k1s0@postgres:5432/k1s0
    - KAFKA_BROKERS=kafka:9092
    - AUTH_JWKS_URL=http://keycloak:8080/realms/k1s0/protocol/openid-connect/certs
  depends_on:
    - postgres
    - kafka
    - keycloak
```

### Kubernetes (本番環境)

[system-server-deploy.md](../_common/deploy.md) に従い、以下の Helm Values を使用する。

| パラメータ | 値 |
| --- | --- |
| replicas | 2 |
| resources.requests.cpu | 200m |
| resources.requests.memory | 256Mi |
| resources.limits.cpu | 500m |
| resources.limits.memory | 512Mi |
| readinessProbe.path | /readyz |
| livenessProbe.path | /healthz |
