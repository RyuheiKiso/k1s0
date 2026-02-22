"""MessagingError のユニットテスト"""

from k1s0_messaging.exceptions import MessagingError, MessagingErrorCodes


def test_messaging_error_str_includes_code() -> None:
    """MessagingError の文字列表現にコードが含まれること。"""
    error = MessagingError(code="PUBLISH_FAILED", message="something went wrong")
    assert str(error) == "PUBLISH_FAILED: something went wrong"


def test_messaging_error_with_cause() -> None:
    """cause が設定されること。"""
    cause = RuntimeError("root cause")
    error = MessagingError(code="RECEIVE_FAILED", message="failed", cause=cause)
    assert error.__cause__ is cause


def test_messaging_error_without_cause() -> None:
    """cause なしでも作成できること。"""
    error = MessagingError(code="CONNECTION_FAILED", message="no connection")
    assert error.code == "CONNECTION_FAILED"
    assert str(error) == "CONNECTION_FAILED: no connection"


def test_messaging_error_codes_constants() -> None:
    """エラーコード定数が正しい値を持つこと。"""
    assert MessagingErrorCodes.PUBLISH_FAILED == "PUBLISH_FAILED"
    assert MessagingErrorCodes.RECEIVE_FAILED == "RECEIVE_FAILED"
    assert MessagingErrorCodes.CONNECTION_FAILED == "CONNECTION_FAILED"
    assert MessagingErrorCodes.SERIALIZATION_ERROR == "SERIALIZATION_ERROR"


def test_messaging_error_is_exception() -> None:
    """MessagingError が Exception のサブクラスであること。"""
    error = MessagingError(code="PUBLISH_FAILED", message="test")
    assert isinstance(error, Exception)
