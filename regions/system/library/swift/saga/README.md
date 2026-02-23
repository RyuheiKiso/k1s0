# k1s0-saga

k1s0 Saga パターンクライアント Swift ライブラリ

Saga ワークフローの開始・状態取得・キャンセルを提供します。

## 使い方

```swift
import K1s0Saga

let client = SagaClient(endpoint: "http://saga-server:8080")
let response = try await client.startSaga(StartSagaRequest(
    workflowName: "order-saga",
    payload: ["orderId": "order-1"],
    initiatedBy: "order-service"
))
let state = try await client.getSaga(id: response.sagaId)
```

## 開発

```bash
swift build
swift test
```
