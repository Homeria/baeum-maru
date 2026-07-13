from __future__ import annotations

import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI

from app.api.router import api_router
from app.core.bootstrap import bootstrap_runtime


def create_app(runtime_dir: str | Path | None = None) -> FastAPI:
    @asynccontextmanager
    async def lifespan(application: FastAPI) -> AsyncIterator[None]:
        runtime = bootstrap_runtime(runtime_dir)
        application.state.runtime = runtime
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
    application.include_router(api_router, prefix="/api/v1")
    return application


app = create_app()
