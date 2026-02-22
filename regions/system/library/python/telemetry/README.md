# k1s0-telemetry

k1s0 telemetry ライブラリ — OpenTelemetry + structlog による可観測性基盤を提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_telemetry import init_telemetry, new_logger, TelemetryConfig, TraceConfig

config = TelemetryConfig(
    service_name="my-service",
    trace=TraceConfig(enabled=True, endpoint="http://otel-collector:4317", sample_rate=0.1),
)
init_telemetry(config)
logger = new_logger(level="INFO", format="json")
logger.info("Service started", service=config.service_name)
```

## 開発

```bash
uv run pytest
```
