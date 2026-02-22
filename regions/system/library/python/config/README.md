# k1s0-config

k1s0 config ライブラリ — YAML 設定ファイルの読み込み・マージ・バリデーションを提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from pathlib import Path
from k1s0_config import load

config = load(Path("config/base.yaml"), Path("config/production.yaml"))
print(config.app.name)
print(config.server.port)
```

## 開発

```bash
uv run pytest
```
