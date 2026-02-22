# k1s0-dlq-client

k1s0 dlq_client ライブラリ — Dead Letter Queue サーバーへの REST クライアントを提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_dlq_client import HttpDlqClient, DlqConfig
import uuid

config = DlqConfig(base_url="http://dlq-server:8080", api_key="my-key")
client = HttpDlqClient(config)

# メッセージ一覧取得
response = await client.list_messages("events", page=1, page_size=20)
for msg in response.messages:
    print(msg.id, msg.status)

# リトライ
await client.retry_message(uuid.UUID("..."))
```

## 開発

```bash
uv run pytest
```
