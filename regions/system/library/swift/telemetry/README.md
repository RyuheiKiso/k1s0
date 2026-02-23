# k1s0-telemetry

k1s0 テレメトリ Swift ライブラリ

構造化ログ、メトリクス収集、トレーシングを提供します。

## 使い方

```swift
import K1s0Telemetry

TelemetrySetup.initialize(config: TelemetryConfig(
    serviceName: "order-service",
    version: "1.0.0",
    tier: "service",
    environment: "prod",
    traceEndpoint: "http://jaeger:4317",
    sampleRate: 0.1
))

let logger = Logger(subsystem: "com.k1s0.order-service", category: "orders")
logger.info("注文処理を開始します")

await TelemetrySetup.metrics.incrementHttpRequests(method: "POST", path: "/orders", status: 201)
```

## 開発

```bash
swift build
swift test
```
