import os
from contextlib import asynccontextmanager

from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker

database_url = os.getenv("DATABASE_URL", "postgresql+asyncpg://user:pass@localhost:5432/mydb")
api_secret = os.getenv("API_SECRET", "default-secret")

engine = create_async_engine(database_url, echo=True)
async_session = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)


@asynccontextmanager
async def get_session():
    async with async_session() as session:
        try:
            yield session
        except Exception:
            await session.rollback()
            raise
