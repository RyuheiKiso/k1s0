# k1s0-dlq

k1s0 Dead Letter Queue クライアント Swift ライブラリ

DLQ の HTTP REST API クライアントを提供します。

## 使い方

```swift
import K1s0Dlq

let client = DlqClient(endpoint: "http://dlq-service:8080")
let messages = try await client.listMessages(topic: "k1s0.service.orders.order-created.v1")
try await client.retryMessage(id: messages.messages.first!.id)
```

## 開発

```bash
swift build
swift test
```
