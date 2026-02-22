# K1s0.System.Saga

Saga パターンクライアントライブラリ。分散トランザクションの開始・状態取得・キャンセルを管理する。

## 機能

- HTTP REST クライアント (`HttpSagaClient`)
- gRPC クライアント (`GrpcSagaClient`) - proto ファイル生成後に実装予定
- プロトコル切り替え (`SagaConfig.Protocol`)
- DI 拡張メソッド

## 使い方

### HTTP モード

```csharp
services.AddK1s0Saga(new SagaConfig(
    RestBaseUrl: "http://saga-server:8080",
    GrpcEndpoint: null,
    Protocol: SagaProtocol.Http,
    TimeoutSeconds: 30));
```

### gRPC モード (proto 生成後)

```csharp
services.AddK1s0Saga(new SagaConfig(
    RestBaseUrl: "http://saga-server:8080",
    GrpcEndpoint: "http://saga-server:50051",
    Protocol: SagaProtocol.Grpc,
    TimeoutSeconds: 30));
```

### クライアント利用

```csharp
public class OrderService(ISagaClient sagaClient)
{
    public async Task<string> StartOrderFulfillment(string orderId)
    {
        var response = await sagaClient.StartSagaAsync(new StartSagaRequest(
            WorkflowName: "order-fulfillment",
            Payload: $$$"""{"orderId":"{{{orderId}}}"}""",
            CorrelationId: Guid.NewGuid().ToString()));

        return response.SagaId;
    }
}
```

## API エンドポイント

| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/v1/sagas` | Saga 開始 |
| GET | `/api/v1/sagas/{id}` | Saga 状態取得 |
| POST | `/api/v1/sagas/{id}/cancel` | Saga キャンセル |

## テスト

```bash
dotnet test tests/
```
