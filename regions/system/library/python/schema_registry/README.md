# k1s0-schema-registry

k1s0 schema_registry ライブラリ — Confluent Schema Registry との連携を提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_schema_registry import HttpSchemaRegistryClient, SchemaRegistryConfig, SchemaType

config = SchemaRegistryConfig(url="http://schema-registry:8081")
client = HttpSchemaRegistryClient(config)

schema_id = client.register_schema("user-value", '{"type":"record","name":"User","fields":[]}')
schema = client.get_schema_by_id(schema_id)
print(schema.schema)
```

## 開発

```bash
uv run pytest
```
