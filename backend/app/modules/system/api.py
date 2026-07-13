from fastapi import APIRouter

from app.api.dependencies import ContainerDependency
from app.modules.system.application import GetHealthQuery
from app.modules.system.schemas import HealthResponse

router = APIRouter(tags=["system"])


@router.get("/health", response_model=HealthResponse)
def get_health(container: ContainerDependency) -> HealthResponse:
    status = container.system.get_health.fetch(GetHealthQuery())
    return HealthResponse.model_validate(status)
