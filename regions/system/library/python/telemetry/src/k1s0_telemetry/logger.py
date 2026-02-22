"""structlog ベースのロガー設定"""

from __future__ import annotations

import logging
import sys

import structlog


def new_logger(level: str = "INFO", format: str = "json") -> structlog.stdlib.BoundLogger:
    """設定済みの structlog ロガーを返す。

    Args:
        level: ログレベル ("DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL")
        format: 出力形式 ("json" or "text")

    Returns:
        設定済みの structlog.stdlib.BoundLogger
    """
    log_level = getattr(logging, level.upper(), logging.INFO)
    logging.basicConfig(
        format="%(message)s",
        stream=sys.stdout,
        level=log_level,
    )

    processors: list[structlog.types.Processor]
    if format == "json":
        processors = [
            structlog.contextvars.merge_contextvars,
            structlog.processors.add_log_level,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.processors.StackInfoRenderer(),
            structlog.processors.JSONRenderer(),
        ]
    else:
        processors = [
            structlog.contextvars.merge_contextvars,
            structlog.processors.add_log_level,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.dev.ConsoleRenderer(),
        ]

    structlog.configure(
        processors=processors,
        wrapper_class=structlog.stdlib.BoundLogger,
        context_class=dict,
        logger_factory=structlog.stdlib.LoggerFactory(),
        cache_logger_on_first_use=True,
    )

    return structlog.stdlib.get_logger()
