# K1s0.System.Messaging

Kafka イベント発行・購読の抽象化ライブラリ (C#)。

## 機能

- **IEventProducer / KafkaEventProducer**: Confluent.Kafka ベースのイベント発行
- **IEventConsumer / KafkaEventConsumer**: Confluent.Kafka ベースのイベント購読 (手動コミット)
- **NoOpEventProducer**: テスト用の何もしない実装
- **EventEnvelope / EventMetadata**: イベントメッセージの標準化された構造
- **DI 拡張**: `AddK1s0Messaging` で簡単にサービス登録

## 使用方法

```csharp
using K1s0.System.Messaging;

// DI 登録 (Producer のみ)
services.AddK1s0Messaging(new MessagingConfig(
    Brokers: ["localhost:9092"]));

// DI 登録 (Producer + Consumer)
services.AddK1s0Messaging(
    new MessagingConfig(
        Brokers: ["localhost:9092"],
        ConsumerConfig: new ConsumerConfig(GroupId: "my-group")),
    "topic-a", "topic-b");

// イベント発行
var metadata = EventMetadata.New("user.created", "auth-service", "trace-1", "corr-1");
var payload = System.Text.Encoding.UTF8.GetBytes("{\"user_id\":\"123\"}");
var envelope = new EventEnvelope("user-events", "user-123", payload, metadata);
await producer.PublishAsync(envelope);

// イベント受信 (手動コミット)
var message = await consumer.ReceiveAsync();
// process message...
await consumer.CommitAsync(message);
```

## テスト

```bash
dotnet test tests/
```

テスト用には `NoOpEventProducer` または NSubstitute による `IEventProducer` / `IEventConsumer` のモックを使用してください。
