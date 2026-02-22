# k1s0-messaging

k1s0 messaging ライブラリ — Kafka ベースのイベントプロデューサー・コンシューマーを提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_messaging import KafkaEventProducer, EventEnvelope

with KafkaEventProducer(brokers=["kafka:9092"]) as producer:
    envelope = EventEnvelope(topic="user-events", payload=b'{"type":"created"}')
    producer.publish(envelope)
```

テスト用:
```python
from k1s0_messaging import NoOpEventProducer
producer = NoOpEventProducer()
producer.publish(envelope)
assert len(producer.published) == 1
```

## 開発

```bash
uv run pytest
```
