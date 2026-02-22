"""ロガー設定のユニットテスト"""

from k1s0_telemetry.logger import new_logger


def test_new_logger_json_format() -> None:
    """JSON フォーマットのロガーが作成できること。"""
    logger = new_logger(level="INFO", format="json")
    assert logger is not None


def test_new_logger_text_format() -> None:
    """テキストフォーマットのロガーが作成できること。"""
    logger = new_logger(level="DEBUG", format="text")
    assert logger is not None


def test_new_logger_default_params() -> None:
    """デフォルトパラメータでロガーが作成できること。"""
    logger = new_logger()
    assert logger is not None


def test_new_logger_returns_bound_logger() -> None:
    """structlog.stdlib.BoundLogger が返ること。"""
    logger = new_logger()
    # structlog の BoundLogger は bind メソッドを持つ
    bound = logger.bind(key="value")
    assert bound is not None
