"""Domain event abstract base class."""

from __future__ import annotations

from abc import ABC, abstractmethod


class DomainEvent(ABC):
    """Abstract base class for all domain events.

    Subclasses must implement event_type, aggregate_id, and aggregate_type
    to identify the event and the aggregate it belongs to.
    """

    @abstractmethod
    def event_type(self) -> str:
        """Return the unique type identifier for this event."""

    @abstractmethod
    def aggregate_id(self) -> str:
        """Return the ID of the aggregate that produced this event."""

    @abstractmethod
    def aggregate_type(self) -> str:
        """Return the type of the aggregate that produced this event."""
