"""서버 프로세스의 생존 여부를 확인하는 health endpoint."""

from typing import Literal

from fastapi import APIRouter
from pydantic import BaseModel

router = APIRouter(tags=["system"])


class HealthResponse(BaseModel):
    status: Literal["ok"] = "ok"
    service: Literal["baeum-maru"] = "baeum-maru"


@router.get("/health", response_model=HealthResponse, summary="서버 생존 확인")
def health() -> HealthResponse:
    return HealthResponse()
