"""k1s0-validation: Input validation built on Pydantic."""

from __future__ import annotations

from k1s0_validation.extensions import validate_kebab_case, validate_non_empty, validate_uuid_str
from k1s0_validation.validator import K1s0BaseModel

__all__ = [
    "K1s0BaseModel",
    "validate_kebab_case",
    "validate_non_empty",
    "validate_uuid_str",
]
