> **ステータス: 未実装（設計のみ）**

# Migration スキーマ進化管理

## 概要

`k1s0-migration` の `schema-evolution` Feature で有効化される拡張モジュール群。DOWN SQL 自動生成、破壊的変更検出、スキーマ差分比較、宣言的スキーマ定義 (TOML→SQL) を提供する。

## Feature Flags

```toml
[dependencies]
k1s0-migration = { path = "...", features = ["schema-evolution"] }
```

| Feature | 追加依存 | 説明 |
|---------|---------|------|
| `schema-evolution` | sqlparser, toml | reverse, analyzer, diff, declarative モジュールを有効化 |

## API

### reverse -- DOWN SQL 自動生成

UP SQL を解析し、対応する DOWN SQL を自動生成する。依存関係を考慮して逆順で出力。

```rust
pub fn generate_down_sql(up_sql: &str) -> Result<String, MigrationError>;
```

対応するステートメント:

| UP SQL | 生成される DOWN SQL |
|--------|-------------------|
| `CREATE TABLE t` | `DROP TABLE IF EXISTS t CASCADE` |
| `CREATE INDEX idx ON t` | `DROP INDEX IF EXISTS idx` |
| `ALTER TABLE t ADD COLUMN c` | `ALTER TABLE t DROP COLUMN c` |
| `ALTER TABLE t ADD CONSTRAINT c` | `ALTER TABLE t DROP CONSTRAINT c` |

### analyzer -- 破壊的変更検出

SQL から後方互換性を壊す変更を検出し、`BreakingChange` として返す。

```rust
pub fn detect_breaking_changes(sql: &str) -> Vec<BreakingChange>;

pub enum BreakingChange {
    ColumnDropped { table: String, column: String },
    ColumnTypeChanged { table: String, column: String, from: String, to: String },
    TableDropped { table: String },
    NotNullAdded { table: String, column: String },
    ColumnRenamed { table: String, from: String, to: String },
}
```

### schema -- スキーマ表現型

スキーマの構造を表現するデータ型。diff モジュールの入出力に使用。

```rust
pub struct Schema { pub tables: Vec<Table> }
pub struct Table { pub name: String, pub columns: Vec<Column>, pub indexes: Vec<Index>, pub constraints: Vec<Constraint> }
pub struct Column { pub name: String, pub data_type: String, pub nullable: bool, pub default: Option<String> }
pub struct Index { pub name: String, pub table: String, pub columns: Vec<String>, pub unique: bool }
pub enum Constraint { PrimaryKey { .. }, ForeignKey { .. }, Unique { .. }, Check { .. } }
```

### diff -- スキーマ差分比較

2つの `Schema` を比較し、差分を `SchemaDiff` のリストとして返す。

```rust
pub fn diff_schemas(old: &Schema, new: &Schema) -> Vec<SchemaDiff>;

pub enum SchemaDiff {
    TableAdded(Table),
    TableDropped(String),
    ColumnAdded { table: String, column: Column },
    ColumnDropped { table: String, column: String },
    ColumnChanged { table: String, column: String, from: Column, to: Column },
}
```

### declarative -- TOML 宣言的スキーマ定義

TOML 形式のテーブル定義から `CREATE TABLE` SQL を生成する。

```rust
pub fn toml_to_create_sql(toml_str: &str) -> Result<String, MigrationError>;
```

TOML フォーマット:

```toml
[table]
name = "users"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "email"
type = "TEXT"
nullable = true
unique = true
default = "'unknown'"
references = "other_table(id)"
```

カラムオプション:

| フィールド | 型 | デフォルト | 説明 |
|-----------|-----|----------|------|
| `name` | String | (必須) | カラム名 |
| `type` | String | (必須) | SQL型 |
| `primary_key` | bool | false | 主キー |
| `nullable` | bool | true | NULL許可 |
| `unique` | bool | false | ユニーク制約 |
| `default` | String? | None | デフォルト値 |
| `references` | String? | None | 外部キー参照 |

## 使用例

```rust
use k1s0_migration::reverse::generate_down_sql;
use k1s0_migration::analyzer::detect_breaking_changes;
use k1s0_migration::diff::diff_schemas;
use k1s0_migration::declarative::toml_to_create_sql;

// DOWN SQL 自動生成
let up = "CREATE TABLE users (id UUID PRIMARY KEY, name TEXT NOT NULL);";
let down = generate_down_sql(up)?;
// => "DROP TABLE IF EXISTS users CASCADE;"

// 破壊的変更検出
let changes = detect_breaking_changes("ALTER TABLE users DROP COLUMN email;");
for change in &changes {
    eprintln!("WARNING: {}", change);  // "Column users.email dropped"
}

// スキーマ差分
let diffs = diff_schemas(&old_schema, &new_schema);
for diff in &diffs {
    println!("{:?}", diff);
}

// TOML → CREATE TABLE
let sql = toml_to_create_sql(&std::fs::read_to_string("schema/users.toml")?)?;
// => "CREATE TABLE users (\n  id UUID,\n  name TEXT NOT NULL,\n  PRIMARY KEY (id)\n);"
```

## 設計判断

| 判断 | 理由 |
|------|------|
| sqlparser による SQL 解析 | 正規表現では扱えない複雑な SQL 構文に対応 |
| DOWN SQL の逆順出力 | 依存関係（FK等）を考慮した安全なロールバック |
| BreakingChange を enum で表現 | パターンマッチで網羅的な処理を強制 |
| Schema 型を独立定義 | sqlparser の AST に依存せず、diff/declarative 間で共通利用 |
| TOML による宣言的定義 | SQL を直接書くより可読性が高く、バリデーションも容易 |
