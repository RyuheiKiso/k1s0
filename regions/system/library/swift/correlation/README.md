# k1s0-correlation

k1s0 相関ID追跡 Swift ライブラリ

分散トレーシング用の相関ID・トレースIDを生成・伝播します。

## 使い方

```swift
import K1s0Correlation

let ctx = CorrelationContext()
let headers = CorrelationHeaders.toHeaders(ctx)
// HTTP リクエストにヘッダーを付与
```

## 開発

```bash
swift build
swift test
```
