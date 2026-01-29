"""Async SQLAlchemy session factory from YAML configuration."""

from __future__ import annotations

from sqlalchemy.ext.asyncio import AsyncEngine, AsyncSession, async_sessionmaker, create_async_engine

from k1s0_config.config import K1s0Config


def create_async_engine_from_config(config: K1s0Config) -> AsyncEngine:
    """Create an async SQLAlchemy engine from k1s0 configuration.

    Reads database connection parameters from the "database" section:
    - database.host
    - database.port
    - database.name
    - database.user
    - database.password (resolved from _file suffix)
    - database.pool_size (optional, default 5)

    Args:
        config: k1s0 configuration instance.

    Returns:
        An AsyncEngine connected to the configured database.
    """
    host = config.get_str("database.host", "localhost")
    port = config.get_int("database.port", 5432)
    name = config.get_str("database.name", "k1s0")
    user = config.get_str("database.user", "postgres")
    password = config.get_str("database.password", "")
    pool_size = config.get_int("database.pool_size", 5)

    url = f"postgresql+asyncpg://{user}:{password}@{host}:{port}/{name}"

    return create_async_engine(
        url,
        pool_size=pool_size,
        echo=config.get_bool("database.echo", False),
    )


def create_session_factory(engine: AsyncEngine) -> async_sessionmaker[AsyncSession]:
    """Create an async session factory bound to the given engine.

    Args:
        engine: The AsyncEngine to bind sessions to.

    Returns:
        An async_sessionmaker that produces AsyncSession instances.
    """
    return async_sessionmaker(
        bind=engine,
        class_=AsyncSession,
        expire_on_commit=False,
    )
