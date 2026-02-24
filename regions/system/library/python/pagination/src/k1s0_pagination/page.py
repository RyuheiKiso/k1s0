"""Page-based pagination."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Generic, TypeVar

T = TypeVar("T")

MIN_PER_PAGE = 1
MAX_PER_PAGE = 100


class PerPageValidationError(ValueError):
    """Raised when per_page is out of valid range."""

    def __init__(self, value: int) -> None:
        super().__init__(
            f"invalid per_page: {value} (must be between {MIN_PER_PAGE} and {MAX_PER_PAGE})"
        )
        self.value = value


def validate_per_page(per_page: int) -> int:
    """Validate that per_page is between 1 and 100."""
    if per_page < MIN_PER_PAGE or per_page > MAX_PER_PAGE:
        raise PerPageValidationError(per_page)
    return per_page


@dataclass
class PageRequest:
    """Page request with 0-based page number."""

    page: int
    per_page: int


@dataclass
class PaginationMeta:
    """Offset pagination metadata."""

    total: int
    page: int
    per_page: int
    total_pages: int


@dataclass
class PageResponse(Generic[T]):
    """Page response with total count."""

    items: list[T]
    total: int
    page: int
    per_page: int
    total_pages: int

    @classmethod
    def create(
        cls, items: list[T], total: int, req: PageRequest
    ) -> PageResponse[T]:
        total_pages = math.ceil(total / req.per_page) if req.per_page > 0 else 0
        return cls(
            items=items,
            total=total,
            page=req.page,
            per_page=req.per_page,
            total_pages=total_pages,
        )

    @property
    def meta(self) -> PaginationMeta:
        """Return the pagination metadata for this response."""
        return PaginationMeta(
            total=self.total,
            page=self.page,
            per_page=self.per_page,
            total_pages=self.total_pages,
        )
