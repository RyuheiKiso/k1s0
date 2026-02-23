"""Page-based pagination."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Generic, TypeVar

T = TypeVar("T")


@dataclass
class PageRequest:
    """Page request with 0-based page number."""

    page: int
    per_page: int


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
