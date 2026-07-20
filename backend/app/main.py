"""FastAPI application 생성, 공통 경계 등록과 lifespan을 담당한다."""

from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI

from app.api.dependencies import (
    RealtimeSessionVerifier,
    default_realtime_session_verifier,
)
from app.api.errors import register_exception_handlers
from app.api.middleware import request_id_middleware
from app.api.realtime import RealtimeHub
from app.api.routers.auth import router as auth_router
from app.api.routers.buildings import router as buildings_router
from app.api.routers.courses import router as courses_router
from app.api.routers.health import router as health_router
from app.api.routers.lottery import router as lottery_router
from app.api.routers.members import router as members_router
from app.api.routers.offerings import router as offerings_router
from app.api.routers.operators import router as operators_router
from app.api.routers.realtime import router as realtime_router
from app.api.routers.registrations import router as registrations_router
from app.api.routers.spaces import router as spaces_router
from app.core.logging import configure_logging
from app.core.runtime import RuntimePaths
from app.core.settings import AppSettings, load_settings
from app.db.database import initialize_database

API_PREFIX = "/api/v1"


def create_app(
    *,
    runtime_paths: RuntimePaths | None = None,
    settings: AppSettings | None = None,
    initialize_schema: bool = True,
    realtime_session_verifier: RealtimeSessionVerifier | None = None,
) -> FastAPI:
    @asynccontextmanager
    async def lifespan(application: FastAPI) -> AsyncIterator[None]:
        paths = runtime_paths or RuntimePaths.discover()
        paths.ensure_directories()
        app_settings = settings or load_settings(paths)
        configure_logging(app_settings.logging, paths.application_log_file)

        if initialize_schema:
            initialize_database(paths.database_file, app_settings.database)

        realtime_hub = RealtimeHub(app_settings.realtime)
        application.state.runtime_paths = paths
        application.state.settings = app_settings
        application.state.realtime_hub = realtime_hub
        application.state.resource_event_sink = realtime_hub.publish
        await realtime_hub.start()
        try:
            yield
        finally:
            await realtime_hub.stop()

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
    application.state.realtime_session_verifier = (
        realtime_session_verifier or default_realtime_session_verifier
    )
    register_exception_handlers(application)
    application.include_router(health_router, prefix=API_PREFIX)
    application.include_router(realtime_router, prefix=API_PREFIX)
    application.include_router(auth_router, prefix=API_PREFIX)
    application.include_router(operators_router, prefix=API_PREFIX)
    application.include_router(buildings_router, prefix=API_PREFIX)
    application.include_router(spaces_router, prefix=API_PREFIX)
    application.include_router(members_router, prefix=API_PREFIX)
    application.include_router(courses_router, prefix=API_PREFIX)
    application.include_router(offerings_router, prefix=API_PREFIX)
    application.include_router(registrations_router, prefix=API_PREFIX)
    application.include_router(lottery_router, prefix=API_PREFIX)
    return application


app = create_app()
