"""Tests for K1s0Exception and subclasses."""

from __future__ import annotations

import pytest

from k1s0_error.error_code import ErrorCode
from k1s0_error.exception import (
    ConflictException,
    ForbiddenException,
    K1s0Exception,
    NotFoundException,
    UnauthorizedException,
    ValidationException,
)


class TestK1s0Exception:
    def test_base_exception_defaults(self) -> None:
        exc = K1s0Exception("svc.cat.reason", detail="something broke")
        assert exc.http_status == 500
        assert str(exc.error_code) == "svc.cat.reason"
        assert exc.detail == "something broke"
        assert exc.trace_id is None

    def test_base_exception_with_trace_id(self) -> None:
        exc = K1s0Exception("svc.cat.reason", detail="err", trace_id="abc123")
        assert exc.trace_id == "abc123"

    def test_accepts_error_code_object(self) -> None:
        code = ErrorCode("svc.cat.reason")
        exc = K1s0Exception(code, detail="err")
        assert exc.error_code is code

    def test_invalid_error_code_raises(self) -> None:
        with pytest.raises(ValueError):
            K1s0Exception("invalid", detail="err")


class TestSubclasses:
    def test_not_found(self) -> None:
        exc = NotFoundException("user.not_found", detail="User 123 not found")
        assert exc.http_status == 404

    def test_validation(self) -> None:
        exc = ValidationException("user.invalid_email", detail="Bad email")
        assert exc.http_status == 400

    def test_conflict(self) -> None:
        exc = ConflictException("user.duplicate", detail="Already exists")
        assert exc.http_status == 409

    def test_unauthorized(self) -> None:
        exc = UnauthorizedException("auth.invalid_token", detail="Expired")
        assert exc.http_status == 401

    def test_forbidden(self) -> None:
        exc = ForbiddenException("auth.no_permission", detail="Denied")
        assert exc.http_status == 403

    def test_all_are_k1s0_exception(self) -> None:
        exceptions = [
            NotFoundException("svc.not_found", detail="x"),
            ValidationException("svc.invalid", detail="x"),
            ConflictException("svc.conflict", detail="x"),
            UnauthorizedException("svc.unauth", detail="x"),
            ForbiddenException("svc.forbidden", detail="x"),
        ]
        for exc in exceptions:
            assert isinstance(exc, K1s0Exception)


class TestErrorCode:
    def test_valid_two_segments(self) -> None:
        code = ErrorCode("auth.invalid_credentials")
        assert code.service == "auth"
        assert code.category == "invalid_credentials"
        assert code.reason is None

    def test_valid_three_segments(self) -> None:
        code = ErrorCode("user.profile.not_found")
        assert code.service == "user"
        assert code.category == "profile"
        assert code.reason == "not_found"

    def test_invalid_single_segment(self) -> None:
        with pytest.raises(ValueError):
            ErrorCode("invalid")

    def test_invalid_uppercase(self) -> None:
        with pytest.raises(ValueError):
            ErrorCode("Auth.Invalid")

    def test_equality(self) -> None:
        assert ErrorCode("svc.cat") == ErrorCode("svc.cat")
        assert ErrorCode("svc.cat") != ErrorCode("svc.other")

    def test_hash(self) -> None:
        s = {ErrorCode("svc.cat"), ErrorCode("svc.cat")}
        assert len(s) == 1
