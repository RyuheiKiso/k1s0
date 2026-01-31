from fastapi import APIRouter, HTTPException
from sqlalchemy import select

from database.connection import get_session
from models.item import Item, ItemCreate

router = APIRouter()


@router.get("/")
async def list_items():
    async with get_session() as session:
        result = await session.execute(select(Item))
        items = result.scalars().all()
        return items


@router.post("/", status_code=201)
async def create_item(input_data: ItemCreate):
    async with get_session() as session:
        item = Item(name=input_data.name, description=input_data.description)
        session.add(item)
        await session.commit()
        await session.refresh(item)
        return item


@router.get("/{item_id}")
async def get_item(item_id: int):
    async with get_session() as session:
        item = await session.get(Item, item_id)
        if item is None:
            raise HTTPException(status_code=404, detail="Item not found")
        return item


@router.delete("/{item_id}", status_code=204)
async def delete_item(item_id: int):
    async with get_session() as session:
        item = await session.get(Item, item_id)
        if item is None:
            raise HTTPException(status_code=404, detail="Item not found")
        await session.delete(item)
        await session.commit()
