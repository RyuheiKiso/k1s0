# k1s0-correlation

k1s0 correlation ライブラリ — HTTPリクエスト間の相関IDとトレースIDを管理します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_correlation import (
    CorrelationContext,
    extract_from_headers,
    inject_into_headers,
    set_correlation_context,
    get_correlation_context,
)

# HTTPヘッダーから抽出
ctx = extract_from_headers(request.headers)
token = set_correlation_context(ctx)

# 次のリクエストへ伝播
outgoing_headers: dict[str, str] = {}
inject_into_headers(ctx, outgoing_headers)
```

## 開発

```bash
uv run pytest
```
