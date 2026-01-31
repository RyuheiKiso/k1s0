import os

import uvicorn
from fastapi import FastAPI

from routers.items import router as items_router

app = FastAPI(title="My Service")

app.include_router(items_router, prefix="/items", tags=["items"])


@app.get("/health")
async def health_check():
    return {"status": "ok"}


if __name__ == "__main__":
    port = int(os.environ.get("PORT", "8000"))
    database_url = os.environ["DATABASE_URL"]
    print(f"Database URL: {database_url}")
    uvicorn.run(app, host="0.0.0.0", port=port)
