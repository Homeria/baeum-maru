"""관계자 CRUD와 접속 코드 발급·폐기 요청을 받는 router."""

from typing import Annotated

from fastapi import APIRouter, Depends, Query, status

import app.services.auth_service as auth_service
import app.services.operator_service as operator_service
from app.api.dependencies import get_current_operator, get_settings
from app.core.settings import AppSettings
from app.schemas.operators import (
    AccessCodeIssueRequest,
    AccessCodeIssueResponse,
    AccessCodeResponse,
    OperatorCreate,
    OperatorResponse,
    OperatorUpdate,
)

router = APIRouter(tags=["operators"], dependencies=[Depends(get_current_operator)])


@router.post(
    "/operators",
    response_model=OperatorResponse,
    status_code=status.HTTP_201_CREATED,
    summary="관계자 등록",
)
def create_operator(payload: OperatorCreate) -> OperatorResponse:
    operator = operator_service.create_operator(
        display_name=payload.display_name, role=payload.role
    )
    return OperatorResponse.model_validate(operator)


@router.get("/operators", response_model=list[OperatorResponse], summary="관계자 목록")
def list_operators(
    include_inactive: bool = Query(default=False),
) -> list[OperatorResponse]:
    operators = operator_service.list_operators(include_inactive=include_inactive)
    return [OperatorResponse.model_validate(item) for item in operators]


@router.get("/operators/{operator_id}", response_model=OperatorResponse, summary="관계자 조회")
def get_operator(operator_id: int) -> OperatorResponse:
    return OperatorResponse.model_validate(operator_service.get_operator(operator_id))


@router.patch("/operators/{operator_id}", response_model=OperatorResponse, summary="관계자 수정")
def update_operator(operator_id: int, payload: OperatorUpdate) -> OperatorResponse:
    operator = operator_service.update_operator(
        operator_id,
        display_name=payload.display_name,
        role=payload.role,
        is_active=payload.is_active,
    )
    return OperatorResponse.model_validate(operator)


@router.delete(
    "/operators/{operator_id}", status_code=status.HTTP_204_NO_CONTENT, summary="관계자 삭제"
)
def delete_operator(operator_id: int) -> None:
    operator_service.delete_operator(operator_id)


@router.post(
    "/operators/{operator_id}/access-codes",
    response_model=AccessCodeIssueResponse,
    status_code=status.HTTP_201_CREATED,
    summary="접속 코드 발급(평문은 이때 한 번만 반환)",
)
def issue_access_code(
    operator_id: int,
    payload: AccessCodeIssueRequest,
    settings: Annotated[AppSettings, Depends(get_settings)],
) -> AccessCodeIssueResponse:
    ttl = payload.ttl_minutes or settings.auth.access_code_ttl_minutes
    record = auth_service.issue_access_code(operator_id, ttl_minutes=ttl)
    return AccessCodeIssueResponse.model_validate(record)


@router.get(
    "/operators/{operator_id}/access-codes",
    response_model=list[AccessCodeResponse],
    summary="접속 코드 목록",
)
def list_access_codes(operator_id: int) -> list[AccessCodeResponse]:
    codes = auth_service.list_access_codes(operator_id)
    return [AccessCodeResponse.model_validate(item) for item in codes]


@router.post(
    "/operators/{operator_id}/access-codes/{code_id}/revoke",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="접속 코드 폐기",
)
def revoke_access_code(operator_id: int, code_id: int) -> None:
    auth_service.revoke_access_code(operator_id, code_id)
