"""Unit of Work pattern with async context manager."""

from __future__ import annotations

import logging
from types import TracebackType

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker

logger = logging.getLogger("k1s0.db.uow")


class UnitOfWork:
    """Async Unit of Work wrapping a SQLAlchemy session.

    Usage::

        async with UnitOfWork(session_factory) as uow:
            # use uow.session for queries
            await uow.commit()

    On exit without commit, changes are rolled back automatically.
    """

    def __init__(self, session_factory: async_sessionmaker[AsyncSession]) -> None:
        self._session_factory = session_factory
        self._session: AsyncSession | None = None

    @property
    def session(self) -> AsyncSession:
        """The active database session."""
        if self._session is None:
            msg = "UnitOfWork must be used as an async context manager"
            raise RuntimeError(msg)
        return self._session

    async def __aenter__(self) -> UnitOfWork:
        self._session = self._session_factory()
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        if self._session is None:
            return
        try:
            if exc_type is not None:
                await self._session.rollback()
                logger.debug("UnitOfWork rolled back due to exception: %s", exc_type.__name__)
        finally:
            await self._session.close()
            self._session = None

    async def commit(self) -> None:
        """Commit the current transaction."""
        await self.session.commit()

    async def rollback(self) -> None:
        """Explicitly roll back the current transaction."""
        await self.session.rollback()
