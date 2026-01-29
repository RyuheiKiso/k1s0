"""Generic async repository base class."""

from __future__ import annotations

from typing import Generic, TypeVar

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

T = TypeVar("T")


class RepositoryBase(Generic[T]):
    """Base repository providing common CRUD operations.

    Subclasses should set the ``model`` class attribute to the SQLAlchemy
    mapped class they manage.

    Usage::

        class UserRepository(RepositoryBase[User]):
            model = User
    """

    model: type[T]

    def __init__(self, session: AsyncSession) -> None:
        self._session = session

    async def get_by_id(self, entity_id: object) -> T | None:
        """Retrieve an entity by its primary key.

        Args:
            entity_id: The primary key value.

        Returns:
            The entity instance, or None if not found.
        """
        return await self._session.get(self.model, entity_id)

    async def list_all(self, *, limit: int = 100, offset: int = 0) -> list[T]:
        """Retrieve a paginated list of entities.

        Args:
            limit: Maximum number of entities to return.
            offset: Number of entities to skip.

        Returns:
            A list of entity instances.
        """
        stmt = select(self.model).limit(limit).offset(offset)
        result = await self._session.execute(stmt)
        return list(result.scalars().all())

    async def add(self, entity: T) -> T:
        """Add a new entity to the session.

        Args:
            entity: The entity to persist.

        Returns:
            The added entity.
        """
        self._session.add(entity)
        await self._session.flush()
        return entity

    async def delete(self, entity: T) -> None:
        """Mark an entity for deletion.

        Args:
            entity: The entity to remove.
        """
        await self._session.delete(entity)
        await self._session.flush()
