# k1s0-schemaregistry

k1s0 Confluent Schema Registry クライアント Swift ライブラリ

Schema Registry へのスキーマ登録・取得・互換性チェックを提供します。

## 使い方

```swift
import K1s0SchemaRegistry

let config = SchemaRegistryConfig(url: "http://schema-registry:8081")
let client = URLSessionSchemaRegistryClient(config: config)
let schemaId = try await client.registerSchema(
    subject: SchemaRegistryConfig.subjectName(for: "k1s0.service.orders.order-created.v1"),
    schema: mySchemaJSON,
    schemaType: .avro
)
```

## 開発

```bash
swift build
swift test
```
