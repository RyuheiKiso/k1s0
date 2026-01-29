"""Base model for k1s0 validation."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict


class K1s0BaseModel(BaseModel):
    """Base Pydantic model for k1s0 services.

    Provides strict validation defaults:
    - Extra fields are forbidden
    - Strings are stripped of leading/trailing whitespace
    - Validation happens on assignment
    """

    model_config = ConfigDict(
        extra="forbid",
        str_strip_whitespace=True,
        validate_assignment=True,
        frozen=False,
    )
