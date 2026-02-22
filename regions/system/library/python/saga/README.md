# k1s0-saga

k1s0 saga ライブラリ — Saga パターンによる分散トランザクション管理クライアントを提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_saga import HttpSagaClient, SagaConfig, StartSagaRequest

config = SagaConfig(rest_url="http://saga-server:8080")
client = HttpSagaClient(config)

response = await client.start_saga(
    StartSagaRequest(
        workflow_name="order-fulfillment",
        payload={"order_id": "123", "user_id": "456"},
        correlation_id="corr-abc",
    )
)
print(response.saga_id, response.status)
```

## 開発

```bash
uv run pytest
```
