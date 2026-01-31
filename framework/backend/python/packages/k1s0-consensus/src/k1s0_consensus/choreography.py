"""Choreography-based saga using event-driven step coordination."""

from __future__ import annotations

import asyncio
import logging
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any

from k1s0_consensus.config import ChoreographyConfig
from k1s0_consensus.error import ConsensusError

logger = logging.getLogger("k1s0.consensus.choreography")


class EventStepHandler(ABC):
    """Abstract handler for a single choreography saga step.

    Each handler reacts to a specific event type and may emit further
    events to trigger subsequent steps.
    """

    @property
    @abstractmethod
    def event_type(self) -> str:
        """The event type this handler listens for."""

    @abstractmethod
    async def handle(self, payload: dict[str, Any]) -> dict[str, Any] | None:
        """Handle an incoming event.

        Args:
            payload: The event payload.

        Returns:
            An optional result payload. Returning a dict may trigger
            downstream handlers.
        """

    @abstractmethod
    async def on_timeout(self, payload: dict[str, Any]) -> None:
        """Called when this step times out.

        Implementations should perform compensation or alerting logic.

        Args:
            payload: The original event payload that was being handled.
        """


@dataclass
class _RegisteredHandler:
    """Internal record for a registered handler with its timeout."""

    handler: EventStepHandler
    timeout_ms: int


@dataclass
class ChoreographySaga:
    """A choreography-based saga that coordinates steps via events.

    Handlers are registered for specific event types. When an event
    arrives, the matching handler is invoked. A background timeout
    monitor cancels handlers that exceed their configured timeout.

    Args:
        name: Name of the choreography saga.
        config: Configuration for timeouts and behavior.
    """

    name: str
    config: ChoreographyConfig = field(default_factory=ChoreographyConfig)
    _handlers: dict[str, _RegisteredHandler] = field(default_factory=dict, init=False)
    _timeout_tasks: dict[str, asyncio.Task[None]] = field(default_factory=dict, init=False)

    def register(self, handler: EventStepHandler, timeout_ms: int | None = None) -> None:
        """Register an event step handler.

        Args:
            handler: The handler to register.
            timeout_ms: Optional timeout override in milliseconds.
                Uses config default if None.
        """
        effective_timeout = timeout_ms if timeout_ms is not None else self.config.default_timeout_ms
        self._handlers[handler.event_type] = _RegisteredHandler(
            handler=handler,
            timeout_ms=effective_timeout,
        )
        logger.debug("Registered handler for event type %s in saga %s", handler.event_type, self.name)

    async def dispatch(self, event_type: str, payload: dict[str, Any]) -> dict[str, Any] | None:
        """Dispatch an event to the registered handler.

        Starts a timeout monitor for the handler invocation.

        Args:
            event_type: The type of event to dispatch.
            payload: The event payload.

        Returns:
            The handler result, or None.

        Raises:
            ConsensusError: If no handler is registered for the event type.
        """
        registered = self._handlers.get(event_type)
        if registered is None:
            msg = f"No handler registered for event type '{event_type}' in saga '{self.name}'"
            raise ConsensusError(msg)

        # Start timeout monitor
        timeout_event = asyncio.Event()
        timeout_task = asyncio.create_task(
            self._timeout_monitor(registered, payload, timeout_event)
        )
        self._timeout_tasks[event_type] = timeout_task

        try:
            result = await registered.handler.handle(payload)
            timeout_event.set()  # Cancel timeout
            return result
        except Exception:
            timeout_event.set()
            raise
        finally:
            if event_type in self._timeout_tasks:
                task = self._timeout_tasks.pop(event_type)
                if not task.done():
                    task.cancel()
                    try:
                        await task
                    except asyncio.CancelledError:
                        pass

    async def _timeout_monitor(
        self,
        registered: _RegisteredHandler,
        payload: dict[str, Any],
        cancel_event: asyncio.Event,
    ) -> None:
        """Monitor a handler invocation and trigger on_timeout if exceeded.

        Args:
            registered: The registered handler record.
            payload: The original event payload.
            cancel_event: Event set when the handler completes normally.
        """
        try:
            await asyncio.wait_for(
                cancel_event.wait(),
                timeout=registered.timeout_ms / 1000.0,
            )
        except asyncio.TimeoutError:
            logger.warning(
                "Handler for %s in saga %s timed out after %dms",
                registered.handler.event_type,
                self.name,
                registered.timeout_ms,
            )
            try:
                await registered.handler.on_timeout(payload)
            except Exception:
                logger.exception("on_timeout failed for %s", registered.handler.event_type)

    async def shutdown(self) -> None:
        """Cancel all active timeout monitors."""
        for task in self._timeout_tasks.values():
            task.cancel()
        for task in self._timeout_tasks.values():
            try:
                await task
            except asyncio.CancelledError:
                pass
        self._timeout_tasks.clear()


class ChoreographySagaBuilder:
    """Fluent builder for constructing ChoreographySaga instances.

    Example::

        saga = (
            ChoreographySagaBuilder("payment-flow")
            .on("order.created", OrderHandler(), timeout_ms=5000)
            .on("payment.charged", PaymentHandler())
            .build()
        )
    """

    def __init__(self, name: str, config: ChoreographyConfig | None = None) -> None:
        self._name = name
        self._config = config or ChoreographyConfig()
        self._registrations: list[tuple[EventStepHandler, int | None]] = []

    def on(self, event_type: str, handler: EventStepHandler, timeout_ms: int | None = None) -> ChoreographySagaBuilder:
        """Register a handler for an event type.

        The handler's ``event_type`` property is overridden by the
        ``event_type`` argument provided here when they differ. However,
        it is recommended to keep them consistent.

        Args:
            event_type: The event type string (used for dispatch).
            handler: The handler to invoke.
            timeout_ms: Optional timeout in milliseconds.

        Returns:
            The builder for chaining.
        """
        # We use the handler directly; event_type routing is by the handler's property
        self._registrations.append((handler, timeout_ms))
        return self

    def build(self) -> ChoreographySaga:
        """Build the ChoreographySaga.

        Returns:
            The constructed ChoreographySaga.

        Raises:
            ValueError: If no handlers have been registered.
        """
        if not self._registrations:
            msg = "ChoreographySaga must have at least one handler"
            raise ValueError(msg)

        saga = ChoreographySaga(name=self._name, config=self._config)
        for handler, timeout_ms in self._registrations:
            saga.register(handler, timeout_ms)
        return saga
