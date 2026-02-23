# k1s0-outbox

k1s0 トランザクショナルアウトボックスパターン Swift ライブラリ

DB と Kafka の原子性を保証するアウトボックスパターンの実装を提供します。

## 使い方

```swift
import K1s0Outbox

var message = OutboxMessage(
    topic: "k1s0.service.orders.order-created.v1",
    partitionKey: "order-1",
    payload: try JSONEncoder().encode(myEvent)
)
// DBに保存（トランザクション内）
try await store.save(message)
// 定期的にプロセッサーを起動
let processed = try await processor.processBatch()
```

## 開発

```bash
swift build
swift test
```
