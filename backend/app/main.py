"""FastAPI application 생성, 공통 경계 등록과 lifespan을 담당한다."""

from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI

from app.api.errors import register_exception_handlers
from app.api.middleware import request_id_middleware
from app.api.routers.health import router as health_router
from app.core.logging import configure_logging
from app.core.runtime import RuntimePaths
from app.core.settings import AppSettings, load_settings
from app.db.migrations import upgrade_database
from app.db.session import create_session_factory, create_sqlite_engine

API_PREFIX = "/api/v1"


def create_app(
    *,
    runtime_paths: RuntimePaths | None = None,
    settings: AppSettings | None = None,
    apply_migrations: bool = True,
) -> FastAPI:
    @asynccontextmanager
    async def lifespan(application: FastAPI) -> AsyncIterator[None]:
        paths = runtime_paths or RuntimePaths.discover()
        paths.ensure_directories()
        app_settings = settings or load_settings(paths)
        configure_logging(app_settings.logging, paths.application_log_file)

        if apply_migrations:
            upgrade_database(paths.database_file)

        engine = create_sqlite_engine(paths.database_file, app_settings.database)
        application.state.runtime_paths = paths
        application.state.settings = app_settings
        application.state.engine = engine
        application.state.session_factory = create_session_factory(engine)
        try:
            yield
        finally:
            engine.dispose()

    application = FastAPI(
        title="배움마루 API",
        summary="노인복지관 교육 접수와 운영을 위한 내부망 API",
        version="0.1.0",
        openapi_url=f"{API_PREFIX}/openapi.json",
        docs_url="/api/docs",
        redoc_url=None,
        lifespan=lifespan,
    )
    application.middleware("http")(request_id_middleware)
    register_exception_handlers(application)
    application.include_router(health_router, prefix=API_PREFIX)
    return application


app = create_app()
