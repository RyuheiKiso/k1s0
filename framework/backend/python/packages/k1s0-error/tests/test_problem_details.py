"""Tests for ProblemDetails."""

from __future__ import annotations

from k1s0_error.exception import K1s0Exception, NotFoundException
from k1s0_error.problem_details import ProblemDetails


class TestProblemDetails:
    def test_from_exception(self) -> None:
        exc = NotFoundException("user.not_found", detail="User 123 not found", trace_id="t1")
        pd = ProblemDetails.from_exception(exc)
        assert pd.status == 404
        assert pd.title == "Not Found"
        assert pd.detail == "User 123 not found"
        assert pd.error_code == "user.not_found"
        assert pd.trace_id == "t1"

    def test_from_base_exception(self) -> None:
        exc = K1s0Exception("svc.err", detail="fail")
        pd = ProblemDetails.from_exception(exc)
        assert pd.status == 500
        assert pd.title == "Internal Server Error"

    def test_to_dict_without_trace(self) -> None:
        pd = ProblemDetails(status=400, title="Bad Request", detail="bad", error_code="svc.bad")
        d = pd.to_dict()
        assert d == {
            "status": 400,
            "title": "Bad Request",
            "detail": "bad",
            "error_code": "svc.bad",
        }
        assert "trace_id" not in d

    def test_to_dict_with_trace(self) -> None:
        pd = ProblemDetails(status=404, title="Not Found", detail="x", error_code="svc.nf", trace_id="t1")
        assert pd.to_dict()["trace_id"] == "t1"

    def test_frozen(self) -> None:
        pd = ProblemDetails(status=404, title="Not Found", detail="x", error_code="svc.nf")
        try:
            pd.status = 500  # type: ignore[misc]
            raise AssertionError("Should be frozen")
        except AttributeError:
            pass
