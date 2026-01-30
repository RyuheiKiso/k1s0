"""Cache error hierarchy."""

from __future__ import annotations


class CacheError(Exception):
    """Base exception for all cache operations."""


class ConnectionError(CacheError):  # noqa: A001
    """Raised when a connection to the cache backend fails."""


class SerializationError(CacheError):
    """Raised when serialization or deserialization of a cache value fails."""


class OperationError(CacheError):
    """Raised when a cache operation fails unexpectedly."""
