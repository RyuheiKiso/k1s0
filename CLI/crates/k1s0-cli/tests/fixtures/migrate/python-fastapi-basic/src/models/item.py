from typing import Optional

from pydantic import BaseModel
from sqlalchemy import Column, Integer, String
from sqlalchemy.orm import DeclarativeBase


class Base(DeclarativeBase):
    pass


class Item(Base):
    __tablename__ = "items"

    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(String, nullable=False)
    description = Column(String, nullable=True)


class ItemCreate(BaseModel):
    name: str
    description: Optional[str] = None
