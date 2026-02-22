# k1s0-outbox

k1s0 outbox ライブラリ — Transactional Outbox パターンによる信頼性の高いイベント配信を提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_outbox import OutboxProcessor, OutboxConfig, RetryConfig
from k1s0_outbox.in_memory_store import InMemoryOutboxStore

store = InMemoryOutboxStore()
config = OutboxConfig(polling_interval_seconds=1.0, retry=RetryConfig(max_retries=3))
processor = OutboxProcessor(store, config)

await processor.start(my_event_producer)
# ... アプリ処理 ...
await processor.stop()
```

## 開発

```bash
uv run pytest
```
