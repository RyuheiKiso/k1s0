# k1s0-messaging

k1s0 メッセージング抽象化 Swift ライブラリ

Kafka メッセージングの抽象化インターフェースを提供します。

## 使い方

```swift
import K1s0Messaging

let meta = EventMetadata(eventType: "order.created", source: "order-service")
    .withTraceId("trace-id")
let envelope = try EventEnvelope.json(
    topic: "k1s0.service.orders.order-created.v1",
    key: "order-1",
    payload: myEvent
)
```

## 開発

```bash
swift build
swift test
```
