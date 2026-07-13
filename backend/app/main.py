from __future__ import annotations

import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI

from app.composition import compose_api_router, compose_application


def create_app(runtime_dir: str | Path | None = None) -> FastAPI:
    @asynccontextmanager
    async def lifespan(application: FastAPI) -> AsyncIterator[None]:
        container = compose_application(runtime_dir)
        application.state.container = container
        logger = logging.getLogger("baeum_maru.app")
        logger.info("application started", extra={"event": "application.started"})

        try:
            yield
        finally:
            logger.info("application stopped", extra={"event": "application.stopped"})

    application = FastAPI(
        title="Baeum Maru API",
        version="0.1.0",
        lifespan=lifespan,
    )
    application.include_router(compose_api_router(), prefix="/api/v1")
    return application


app = create_app()
