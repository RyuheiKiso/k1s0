# k1s0-kafka

k1s0 Kafka 設定・ヘルスチェック Swift ライブラリ

Kafka クライアント設定の管理とヘルスチェックを提供します。

## 使い方

```swift
import K1s0Kafka

let config = KafkaConfig(brokers: ["kafka:9092"])
try config.validate()

let checker = KafkaHealthChecker(config: config)
let status = await checker.check()
```

## 開発

```bash
swift build
swift test
```
