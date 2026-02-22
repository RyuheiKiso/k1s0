"""OutboxError / OutboxErrorCodes のユニットテスト"""

from k1s0_outbox.exceptions import OutboxError, OutboxErrorCodes


def test_outbox_error_str() -> None:
    """OutboxError の str 表現が 'CODE: message' 形式であること。"""
    err = OutboxError(code="SAVE_FAILED", message="save failed")
    assert str(err) == "SAVE_FAILED: save failed"


def test_outbox_error_code_attribute() -> None:
    """OutboxError の code 属性が正しく設定されること。"""
    err = OutboxError(code="FETCH_FAILED", message="fetch error")
    assert err.code == "FETCH_FAILED"


def test_outbox_error_with_cause() -> None:
    """cause を指定すると __cause__ が設定されること。"""
    cause = ValueError("original error")
    err = OutboxError(code="UPDATE_FAILED", message="update error", cause=cause)
    assert err.__cause__ is cause


def test_outbox_error_without_cause() -> None:
    """cause なしでも正常に生成できること。"""
    err = OutboxError(code="PUBLISH_FAILED", message="publish error")
    assert err.__cause__ is None


def test_outbox_error_codes_constants() -> None:
    """OutboxErrorCodes の定数が正しい値を持つこと。"""
    assert OutboxErrorCodes.SAVE_FAILED == "SAVE_FAILED"
    assert OutboxErrorCodes.FETCH_FAILED == "FETCH_FAILED"
    assert OutboxErrorCodes.UPDATE_FAILED == "UPDATE_FAILED"
    assert OutboxErrorCodes.PUBLISH_FAILED == "PUBLISH_FAILED"


def test_outbox_error_is_exception() -> None:
    """OutboxError が Exception のサブクラスであること。"""
    err = OutboxError(code="SAVE_FAILED", message="test")
    assert isinstance(err, Exception)
