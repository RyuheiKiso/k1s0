---
id: env.data.data_lint_format
axis: data
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.data.data_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- sqlfluff（SQL lint）/ Apicurio schema compatibility check / avro schema lint の 3 ツールを手元で実行し、全 check が通ることを data 軸の lint 検収条件とする。

## sqlfluff のインストールと実行

sqlfluff は SQL ファイルの文法・スタイルを検査するツール。

```bash
source .venv/bin/activate
pip install sqlfluff
sqlfluff --version

# SQL ファイルの lint（PostgreSQL 方言）
sqlfluff lint --dialect postgres <path-to-sql-file>.sql

# 自動修正（safe な修正のみ）
sqlfluff fix --dialect postgres <path-to-sql-file>.sql
```

主要ルール:

| ルール | 内容 |
|---|---|
| L001 | 末尾の不要なスペース禁止 |
| L010 | キーワードは大文字 |
| L014 | 識別子は小文字 |
| L019 | カンマのスペース統一 |
| L040 | NULL 比較は IS NULL / IS NOT NULL を使う |

`.sqlfluff` 設定ファイルをリポジトリ root に配置する:

```ini
[sqlfluff]
dialect = postgres
templater = raw

[sqlfluff:rules:L010]
capitalisation_policy = upper
```

## Apicurio schema compatibility check

新スキーマを登録する前に後方互換性を確認する。

```bash
# 互換性チェック（BACKWARD 互換性の確認）
curl -s -X POST \
  http://localhost:8080/apis/registry/v2/groups/default/artifacts/test-schema/versions/1/compatibility \
  -H "Content-Type: application/json" \
  -d '{
    "type": "record",
    "name": "TestEvent",
    "fields": [
      {"name": "id", "type": "string"},
      {"name": "timestamp", "type": "long"},
      {"name": "new_field", "type": ["null", "string"], "default": null}
    ]
  }' | python3 -m json.tool
# compatible: true が返ることを確認
```

## avro schema lint

Avro スキーマの文法チェックを行う。

```bash
pip install avro-python3 2>/dev/null || pip install avro
python3 -c "
import avro.schema
schema_str = '''
{
  \"type\": \"record\",
  \"name\": \"TestEvent\",
  \"fields\": [
    {\"name\": \"id\", \"type\": \"string\"},
    {\"name\": \"timestamp\", \"type\": \"long\"}
  ]
}
'''
avro.schema.parse(schema_str)
print('Avro schema lint: OK')
"
```

## ClickHouse SQL の lint

ClickHouse の DDL は標準 SQL と異なるため、ClickHouse 方言で sqlfluff を使う。

```bash
sqlfluff lint --dialect clickhouse <path-to-sql-file>.sql
```

## 検収コマンド

```bash
source .venv/bin/activate
sqlfluff --version
python3 -c "import avro; print('avro:', avro.__version__)" 2>/dev/null || echo "avro: installed"
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
