"""Tests for k1s0 validation."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from k1s0_validation.extensions import validate_kebab_case, validate_non_empty, validate_uuid_str
from k1s0_validation.validator import K1s0BaseModel


class SampleModel(K1s0BaseModel):
    name: str
    age: int


class TestK1s0BaseModel:
    def test_valid_model(self) -> None:
        m = SampleModel(name="alice", age=30)
        assert m.name == "alice"
        assert m.age == 30

    def test_strips_whitespace(self) -> None:
        m = SampleModel(name="  alice  ", age=30)
        assert m.name == "alice"

    def test_extra_fields_forbidden(self) -> None:
        with pytest.raises(ValidationError):
            SampleModel(name="alice", age=30, extra="nope")  # type: ignore[call-arg]

    def test_validate_on_assignment(self) -> None:
        m = SampleModel(name="alice", age=30)
        with pytest.raises(ValidationError):
            m.age = "not_an_int"  # type: ignore[assignment]


class TestExtensions:
    def test_validate_non_empty(self) -> None:
        assert validate_non_empty("hello") == "hello"
        assert validate_non_empty("  hello  ") == "hello"

    def test_validate_non_empty_raises(self) -> None:
        with pytest.raises(ValueError, match="must not be empty"):
            validate_non_empty("")
        with pytest.raises(ValueError):
            validate_non_empty("   ")

    def test_validate_kebab_case(self) -> None:
        assert validate_kebab_case("my-service") == "my-service"
        assert validate_kebab_case("service") == "service"
        assert validate_kebab_case("a1-b2") == "a1-b2"

    def test_validate_kebab_case_invalid(self) -> None:
        with pytest.raises(ValueError, match="kebab-case"):
            validate_kebab_case("MyService")
        with pytest.raises(ValueError):
            validate_kebab_case("my_service")

    def test_validate_uuid_str(self) -> None:
        result = validate_uuid_str("550e8400-e29b-41d4-a716-446655440000")
        assert result == "550e8400-e29b-41d4-a716-446655440000"

    def test_validate_uuid_str_invalid(self) -> None:
        with pytest.raises(ValueError, match="UUID"):
            validate_uuid_str("not-a-uuid")
