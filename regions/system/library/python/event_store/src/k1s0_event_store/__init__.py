"""k1s0 event_store library."""

from .exceptions import StreamNotFoundError, VersionConflictError
from .memory import InMemoryEventStore
from .models import EventEnvelope
from .store import EventStore

__all__ = [
    "EventEnvelope",
    "EventStore",
    "InMemoryEventStore",
    "StreamNotFoundError",
    "VersionConflictError",
]
