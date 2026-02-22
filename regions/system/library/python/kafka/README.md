# k1s0-kafka

k1s0 kafka ライブラリ — Kafka 接続設定・ヘルスチェック・トピック管理を提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_kafka import KafkaConfigBuilder, KafkaHealthCheck

config = (
    KafkaConfigBuilder()
    .brokers("kafka:9092")
    .consumer_group("my-service")
    .build()
)

checker = KafkaHealthCheck(config.brokers)
result = checker.check()
print(result.status)
```

## 開発

```bash
uv run pytest
```
