from pathlib import Path

from fastapi import APIRouter

from app.container import ApplicationContainer, SystemModule
from app.core.bootstrap import bootstrap_runtime
from app.modules.system.api import router as system_router
from app.modules.system.application import GetHealthHandler


def compose_application(runtime_dir: str | Path | None = None) -> ApplicationContainer:
    runtime = bootstrap_runtime(runtime_dir)
    system = SystemModule(get_health=GetHealthHandler())
    return ApplicationContainer(runtime=runtime, system=system)


def compose_api_router() -> APIRouter:
    router = APIRouter()
    router.include_router(system_router)
    return router
